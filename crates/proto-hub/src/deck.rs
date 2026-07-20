//! Deckセットリストの型・ロード＆検証（D11）。schemas/deck.schema.json相当。
//! Actionはproto_keymapの型をそのまま再利用する（正は1箇所）。

use crate::error::{LOAD_JSON_SYNTAX, LOAD_SCHEMA_INVALID, LOAD_VK_UNKNOWN};
use proto_keymap::{is_known_vk, Action};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckError {
    pub code: &'static str,
    pub cause: String,
}

impl DeckError {
    fn new(code: &'static str, cause: impl Into<String>) -> Self {
        Self {
            code,
            cause: cause.into(),
        }
    }
}

impl std::fmt::Display for DeckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.cause)
    }
}

impl std::error::Error for DeckError {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    pub cols: u8,
    pub rows: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Slot {
    #[serde(rename = "slotId")]
    pub slot_id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub id: u32,
    pub slots: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeckSetlist {
    #[serde(rename = "deckId")]
    pub deck_id: String,
    #[serde(default)]
    pub description: String,
    pub grid: Grid,
    pub pages: Vec<Page>,
}

impl DeckSetlist {
    pub fn find_slot(&self, slot_id: &str) -> Option<&Slot> {
        self.pages
            .iter()
            .flat_map(|page| page.slots.iter())
            .find(|slot| slot.slot_id == slot_id)
    }

    /// このデッキに現れる全アクション（許可リスト構築・keymapId参照検証に使う）。
    pub fn actions(&self) -> impl Iterator<Item = &Action> {
        self.pages.iter().flat_map(|page| page.slots.iter().map(|slot| &slot.action))
    }
}

pub fn load_deck_from_path(path: impl AsRef<std::path::Path>) -> Result<DeckSetlist, DeckError> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).map_err(|error| {
        DeckError::new(
            LOAD_JSON_SYNTAX,
            format!("{}: failed to read file: {error}", path.display()),
        )
    })?;
    load_deck_str(&path.display().to_string(), &text)
}

pub fn load_deck_str(source: &str, text: &str) -> Result<DeckSetlist, DeckError> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|error| DeckError::new(LOAD_JSON_SYNTAX, format!("{source}: {error}")))?;

    let deck: DeckSetlist = serde_json::from_value(value)
        .map_err(|error| DeckError::new(LOAD_SCHEMA_INVALID, format!("{source}: {error}")))?;

    // slotId重複禁止（複数ページ間も含む）
    let mut seen = std::collections::BTreeSet::new();
    for slot in deck.pages.iter().flat_map(|page| page.slots.iter()) {
        if !seen.insert(slot.slot_id.as_str()) {
            return Err(DeckError::new(
                LOAD_SCHEMA_INVALID,
                format!("{source}: duplicate slotId '{}'", slot.slot_id),
            ));
        }
    }

    // vk辞書検証（Key/Chordのみ。KeymapSwitch先の存在確認はmain.rs側で全ロード後に行う）
    for slot in deck.pages.iter().flat_map(|page| page.slots.iter()) {
        match &slot.action {
            Action::Key { vk } => {
                if !is_known_vk(vk) {
                    return Err(DeckError::new(
                        LOAD_VK_UNKNOWN,
                        format!(
                            "{source}: slot '{}': unknown vk '{vk}'",
                            slot.slot_id
                        ),
                    ));
                }
            }
            Action::Chord { keys } => {
                for vk in keys {
                    if !is_known_vk(vk) {
                        return Err(DeckError::new(
                            LOAD_VK_UNKNOWN,
                            format!(
                                "{source}: slot '{}': unknown vk '{vk}' in chord",
                                slot.slot_id
                            ),
                        ));
                    }
                }
            }
            Action::Mo { .. } | Action::Tg { .. } | Action::Trans => {
                return Err(DeckError::new(
                    LOAD_SCHEMA_INVALID,
                    format!(
                        "{source}: slot '{}': mo/tg/trans are keyboard-layer actions and are not valid on the Deck",
                        slot.slot_id
                    ),
                ));
            }
            // D20: textはDeckでも有効（vk辞書を経由しないため個別チェックは不要）。
            Action::None | Action::KeymapSwitch { .. } | Action::KeymapReset | Action::Text { .. } => {}
        }
    }

    Ok(deck)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_text() -> &'static str {
        r#"{
            "deckId": "t",
            "grid": { "cols": 2, "rows": 1 },
            "pages": [ { "id": 1, "slots": [
                { "slotId": "S01", "label": "Mute", "action": { "t": "key", "vk": "MUTE" } },
                { "slotId": "S02", "label": "Save", "action": { "t": "chord", "keys": ["CTRL", "S"] } }
            ] } ]
        }"#
    }

    #[test]
    fn loads_valid_deck() {
        let deck = load_deck_str("test", sample_text()).unwrap();
        assert_eq!(deck.deck_id, "t");
        assert!(deck.find_slot("S01").is_some());
    }

    #[test]
    fn rejects_duplicate_slot_id() {
        let text = r#"{
            "deckId": "t",
            "grid": { "cols": 1, "rows": 1 },
            "pages": [
                { "id": 1, "slots": [ { "slotId": "S01", "label": "a", "action": { "t": "none" } } ] },
                { "id": 2, "slots": [ { "slotId": "S01", "label": "b", "action": { "t": "none" } } ] }
            ]
        }"#;
        let error = load_deck_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SCHEMA_INVALID);
    }

    #[test]
    fn rejects_unknown_vk() {
        let text = r#"{
            "deckId": "t",
            "grid": { "cols": 1, "rows": 1 },
            "pages": [ { "id": 1, "slots": [ { "slotId": "S01", "label": "a", "action": { "t": "key", "vk": "NOPE" } } ] } ]
        }"#;
        let error = load_deck_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_VK_UNKNOWN);
    }

    #[test]
    fn rejects_mo_action_on_deck() {
        let text = r#"{
            "deckId": "t",
            "grid": { "cols": 1, "rows": 1 },
            "pages": [ { "id": 1, "slots": [ { "slotId": "S01", "label": "a", "action": { "t": "mo", "layer": 1 } } ] } ]
        }"#;
        let error = load_deck_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SCHEMA_INVALID);
    }

    #[test]
    fn real_deck_default_json_loads_successfully() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../decks/deck_default.json");
        let deck = load_deck_from_path(path).expect("decks/deck_default.json must load");
        assert_eq!(deck.deck_id, "default");
    }
}
