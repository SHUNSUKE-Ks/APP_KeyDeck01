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
    /// V2.1: タイルの地色（`#rrggbb`）。省略すると今までどおりの既定の色。
    /// 見た目だけの項目で、**何が起きるかには一切関わらない**。
    /// 形を縛るのは、ここが CSS へそのまま入るため（`url(...)` などを入れさせない）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub id: u32,
    pub slots: Vec<Slot>,
}

/// P-005 段階A: Deckの描き方。データ（label＋actionの並び）は同じで見た目だけが違う。
/// `grid`=正方形スロットの格子（Stream Deck）、`list`=横いっぱいの縦リスト（コピペリスト）。
/// 省略時は`grid`なので、既存のDeck JSONは書き換えなくてよい。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeckRender {
    #[default]
    Grid,
    List,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeckSetlist {
    #[serde(rename = "deckId")]
    pub deck_id: String,
    #[serde(default)]
    pub description: String,
    pub grid: Grid,
    /// P-005 段階A。省略時は`Grid`。
    #[serde(default)]
    pub render: DeckRender,
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

/// `#rrggbb` だけを通す。タイルの地色はCSSへそのまま入るので、ここが唯一の関門になる。
pub fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].bytes().all(|b| b.is_ascii_hexdigit())
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

    // P-005 段階A（§1.3の欠陥修正）: `grid.rows`はこれまで宣言されているだけで、
    // 描画にも検証にも一度も使われていなかった（行数はスロット数÷colsで暗黙に決まっていた）。
    // ここで「1ページのスロット数は cols×rows に収まること」を検証し、rowsを本物の
    // 意味のあるフィールドにする。溢れたスロットが黙って消える事故も同時に防ぐ。
    let capacity = deck.grid.cols as usize * deck.grid.rows as usize;
    for page in &deck.pages {
        if page.slots.len() > capacity {
            return Err(DeckError::new(
                LOAD_SCHEMA_INVALID,
                format!(
                    "{source}: page {} has {} slots but grid is {}x{} (capacity {capacity}); increase grid.cols/grid.rows or split the slots across pages",
                    page.id,
                    page.slots.len(),
                    deck.grid.cols,
                    deck.grid.rows
                ),
            ));
        }
    }

    // V2.1: タイルの地色。CSSへそのまま入るので、`#rrggbb` 以外は通さない
    for slot in deck.pages.iter().flat_map(|page| page.slots.iter()) {
        if let Some(color) = &slot.color {
            if !is_hex_color(color) {
                return Err(DeckError::new(
                    LOAD_SCHEMA_INVALID,
                    format!(
                        "{source}: slot '{}': color must be '#rrggbb' (got '{color}')",
                        slot.slot_id
                    ),
                ));
            }
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
            // T21: tg.fireもレイヤー状態を持つキーボード専用アクション。Deckには
            // レイヤーの概念が無いため、mo/tg/transと同じくロード時に拒否する。
            // P-005 段階C: key.hold/key.buttonはDeckに置けない。`deck.press`にはedgeが無く、
            // 「離す」機会が来ないため、押したキーが永久に押されっぱなしになる。
            Action::KeyHold { .. } | Action::KeyButton { .. } => {
                return Err(DeckError::new(
                    LOAD_SCHEMA_INVALID,
                    format!(
                        "{source}: slot '{}': key.hold/key.button need a release edge, but deck.press has none; use 'key' or 'chord' on a Deck",
                        slot.slot_id
                    ),
                ));
            }
            Action::Mo { .. } | Action::Tg { .. } | Action::Trans | Action::TgFire { .. } => {
                return Err(DeckError::new(
                    LOAD_SCHEMA_INVALID,
                    format!(
                        "{source}: slot '{}': mo/tg/tg.fire/trans are keyboard-layer actions and are not valid on the Deck",
                        slot.slot_id
                    ),
                ));
            }
            // D20: textはDeckでも有効（vk辞書を経由しないため個別チェックは不要）。
            // T10（SR-002）: proto_keymap::ActionへMouseMove追加に伴う網羅性維持のみの1行。
            // vk辞書を経由しない点はTextと同じ扱い。既存Deck JSONはmouse.moveを含まないため無挙動変化。
            // T15（brief/keydeck_trackball_gestures_v0.7.md §4の事前警告どおり、今回は
            // 最初からここを直す）: MouseClick/MouseDoubleClick/MouseButton/MouseScrollも
            // 同じ扱い（vk辞書を経由しない・既存Deck JSONには出現しないため無挙動変化）。
            // アプリ起動も押した瞬間に1回で終わるので、Deckに置いてよい。
            // 参照先のアプリが実在するかは startup 側でまとめて確認する
            // （ここからは apps/apps.json が見えないため）。
            Action::AppLaunch { .. } => {}
            // 表示するboardを切り替えるだけなので、Deckに置いても問題ない
            // （押した瞬間に1回で終わる。離す機会を必要としない）。
            // 中の fire は Key/Chord と同じ規則で vk を検証する。
            Action::LayoutSwitch { fire, .. } => {
                if let Some(inner) = fire.as_ref() {
                    match inner.as_ref() {
                        Action::Key { vk } => {
                            if !is_known_vk(vk) {
                                return Err(DeckError::new(
                                    LOAD_VK_UNKNOWN,
                                    format!(
                                        "{source}: slot '{}': unknown vk '{vk}' in layout.switch",
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
                                            "{source}: slot '{}': unknown vk '{vk}' in layout.switch chord",
                                            slot.slot_id
                                        ),
                                    ));
                                }
                            }
                        }
                        Action::Text { .. } => {}
                        other => {
                            return Err(DeckError::new(
                                LOAD_SCHEMA_INVALID,
                                format!(
                                    "{source}: slot '{}': layout.switch.fire must be key/chord/text, got {other:?}",
                                    slot.slot_id
                                ),
                            ));
                        }
                    }
                }
            }
            Action::None
            | Action::KeymapSwitch { .. }
            | Action::KeymapReset
            | Action::Text { .. }
            | Action::MouseMove { .. }
            | Action::MouseClick { .. }
            | Action::MouseDoubleClick { .. }
            | Action::MouseButton { .. }
            | Action::MouseScroll { .. }
            | Action::MouseWheel { .. } => {}
        }
    }

    Ok(deck)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// V2.1: タイルの地色。CSSへそのまま入る値なので、形の合わないものは読み込みで止める。
    /// ここが緩むと、Deck編集画面から端末の画面へ任意のCSSを流し込めてしまう。
    #[test]
    fn slot_color_accepts_only_hex_and_is_optional() {
        // 色の `"#...` が `r#"` を閉じてしまうので、囲みは `r##"` にする
        let with_color = r##"{
            "deckId": "t",
            "grid": { "cols": 1, "rows": 1 },
            "pages": [ { "id": 1, "slots": [
                { "slotId": "S01", "label": "A", "color": "#1b8a5a",
                  "action": { "t": "key", "vk": "A" } }
            ] } ]
        }"##;
        let deck = load_deck_str("test", with_color).expect("hex color must load");
        assert_eq!(deck.pages[0].slots[0].color.as_deref(), Some("#1b8a5a"));

        for bad in ["red", "#1b8a5", "#1b8a5az", "url(x)", "#12345g"] {
            let text = with_color.replace("#1b8a5a", bad);
            assert!(
                load_deck_str("test", &text).is_err(),
                "color '{bad}' must be rejected, but it loaded"
            );
        }

        // 色を書かない既存のDeckはそのまま読めて、書き出しても欄は増えない
        let plain = load_deck_str("test", sample_text()).expect("load");
        assert!(plain.pages[0].slots[0].color.is_none());
        let round = serde_json::to_string(&plain).expect("serialize");
        assert!(!round.contains("color"), "{round}");
    }

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

    /// P-005 段階A: `grid.rows`が本当に効いていること（capacity超過を拒否）。
    /// この欠陥は「rowsを2にしてもDeckが2段にならない」という形で実機に出ていた。
    #[test]
    fn rejects_more_slots_than_grid_capacity() {
        let text = r#"{
            "deckId": "t",
            "grid": { "cols": 2, "rows": 1 },
            "pages": [ { "id": 1, "slots": [
                { "slotId": "S01", "label": "a", "action": { "t": "none" } },
                { "slotId": "S02", "label": "b", "action": { "t": "none" } },
                { "slotId": "S03", "label": "c", "action": { "t": "none" } }
            ] } ]
        }"#;
        let error = load_deck_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SCHEMA_INVALID);
        assert!(error.cause.contains("capacity"), "cause must explain the capacity: {}", error.cause);
    }

    /// `render`は省略可能で、省略時はgrid（既存のDeck JSONを書き換えなくてよい）。
    #[test]
    fn render_defaults_to_grid_and_accepts_list() {
        let deck = load_deck_str("test", sample_text()).unwrap();
        assert_eq!(deck.render, DeckRender::Grid);

        let text = r#"{
            "deckId": "l",
            "grid": { "cols": 1, "rows": 1 },
            "render": "list",
            "pages": [ { "id": 1, "slots": [
                { "slotId": "S01", "label": "a", "action": { "t": "text", "string": "x" } }
            ] } ]
        }"#;
        assert_eq!(load_deck_str("test", text).unwrap().render, DeckRender::List);
    }

    #[test]
    fn real_deck_default_json_loads_successfully() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../decks/deck_default.json");
        let deck = load_deck_from_path(path).expect("decks/deck_default.json must load");
        assert_eq!(deck.deck_id, "default");
    }
}
