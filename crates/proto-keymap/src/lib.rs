//! proto-keymap — キーマップ型・JSONロード＆検証・レイヤー解決エンジン（T1）
//!
//! 設計書: brief/keydeck_design_v0.2.md の D3/D4/D9/D10。このコメント群は実装指示の一部。
//! チェックポイントID（T1-1..T1-4）は各実装箇所のコメントとテストモジュールに残す。
//!
//! ── 実装するもの（元の骨組みコメントをそのまま維持）─────────────
//!
//! 1. アクション型（D4）
//!    enum Action { Key{vk}, Chord{keys}, Mo{layer}, Tg{layer}, Trans, None,
//!                  KeymapSwitch{id}, KeymapReset }
//!    - vkはD4の固定辞書のみ。辞書は本crateに const で持つ（正は1箇所）。
//!    - serdeのtag="t"でJSONの {"t":"key","vk":"A"} 形式に対応させる。
//!
//! 2. キーマップ構造
//!    Keymap { keymap_id, halves{left,right: rows(Vec<Vec<KeyId>>)}, layers: Vec<Layer> }
//!    Layer { id: u8, keys: Map<KeyId, KeyDef{label, action}> }
//!
//! 3. ロード＆検証  load_keymap(path) -> Result<Keymap, KeymapError>
//!    検証順とエラーコード（D9。cause に「どのファイル・どのkeyId・何が悪いか」を必ず入れる）:
//!    [T1-1] JSON構文       → KeymapError::JsonSyntax   (code=LOAD_JSON_SYNTAX)
//!    [T1-2] スキーマ形状   → KeymapError::SchemaInvalid(code=LOAD_SCHEMA_INVALID)
//!    [T1-3] vk辞書外       → KeymapError::VkUnknown    (code=LOAD_VK_UNKNOWN)
//!           mo/tgの参照先レイヤー不在 → LayerRefInvalid(code=LOAD_LAYER_REF_INVALID)
//!    ※ Layer0必須。Layer0に trans を置くのも SchemaInvalid（最下層に透過先が無いため）。
//!
//! 4. レイヤー状態＋解決エンジン（D3）
//!    LayerState { momentary: BTreeSet<u8>, toggled: BTreeSet<u8> }  // Hubが保持する
//!    - key_down/key_up(keyId) を受けて状態遷移し、発火すべき Action を返す:
//!      resolve(keymap, state, key_id, edge) -> Resolved
//!      enum Resolved { Fire(Action),        // key/chord/keymap.* を down で発火
//!                      LayerChanged,        // mo/tg による状態変化（Hubはlayer.stateを配信）
//!                      Ignored,             // key/chord の up など
//!                      UnknownKey }         // code=KEY_UNKNOWN_ID で呼び出し側がerror返却
//!    - 有効レイヤー = {0} ∪ momentary ∪ toggled のうち番号最大を優先。
//!      そのレイヤーで該当keyIdが未定義 or Trans なら次に大きい有効レイヤーへフォールスルー。
//!      最後まで無ければ Resolved::Fire ではなく code=KEY_RESOLVE_NONE 相当（None扱い・無音でなくログ）。
//!      → 実装注記: 4変数（Fire/LayerChanged/Ignored/UnknownKey）だけでは「解決先が最後まで無い」を
//!        Ignored（正常な無処理）と区別できないため、5番目の変数 `NoResolution` を追加した。
//!        呼び出し側（proto-hub T3）は NoResolution を code=KEY_RESOLVE_NONE として整形する。
//!    - MOのdown/upは対で処理。upで該当レイヤーをmomentaryから除く（多重押しは重複無視でよい）。
//!    - 決定性: 同じ入力列は常に同じ結果（G5）。乱数・時刻を混ぜない。
//!
//! ── 単体テスト（12件以上。T1-4）────────────────────────────
//!  1. layer0の単キー解決 / 2. chord解決 / 3. MO down中はlayer1が勝つ
//!  4. MO up で layer0 に戻る / 5. TG でトグルON→OFF / 6. trans フォールスルー
//!  7. 有効レイヤー複数時は番号最大優先 / 8. 未知keyId → UnknownKey
//!  9. 辞書外vk → LOAD_VK_UNKNOWN / 10. mo参照先レイヤー不在 → LOAD_LAYER_REF_INVALID
//! 11. JSON構文エラー → LOAD_JSON_SYNTAX / 12. 同一入力列2回で結果一致（決定性）
//! （keymap_default.json / keymap_writing01.json の実ファイルロード成功もテストに含める）

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

// ============================================================================
// D9 エラーコード（proto-keymapが生成する範囲。他はproto-hub側で定義）
// ============================================================================

pub const LOAD_JSON_SYNTAX: &str = "LOAD_JSON_SYNTAX";
pub const LOAD_SCHEMA_INVALID: &str = "LOAD_SCHEMA_INVALID";
pub const LOAD_VK_UNKNOWN: &str = "LOAD_VK_UNKNOWN";
pub const LOAD_LAYER_REF_INVALID: &str = "LOAD_LAYER_REF_INVALID";
pub const KEY_UNKNOWN_ID: &str = "KEY_UNKNOWN_ID";
pub const KEY_RESOLVE_NONE: &str = "KEY_RESOLVE_NONE";

/// キーマップロード時のエラー。code はD9固定文字列、cause は
/// 「どのファイル・どのレイヤー・どのkeyId・何が悪いか」を含む人間可読文。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapError {
    pub code: &'static str,
    pub cause: String,
}

impl KeymapError {
    fn new(code: &'static str, cause: impl Into<String>) -> Self {
        Self {
            code,
            cause: cause.into(),
        }
    }
}

impl fmt::Display for KeymapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.cause)
    }
}

impl std::error::Error for KeymapError {}

// ============================================================================
// vk辞書（D4）。正はここ1箇所。proto-adapter-winのVKコード表もこの集合を網羅すること。
// ============================================================================

pub const VK_DICTIONARY: &[&str] = &[
    // A-Z
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z",
    // 0-9
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
    // F1-F24
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13", "F14",
    "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24",
    // 制御・空白系
    "ENTER", "ESC", "TAB", "SPACE", "BKSP", "DEL",
    // 矢印
    "UP", "DOWN", "LEFT", "RIGHT",
    // 修飾
    "CTRL", "SHIFT", "ALT", "WIN",
    // 記号
    "COMMA", "PERIOD", "SLASH", "SEMICOLON", "QUOTE", "MINUS", "EQUALS", "LBRACKET", "RBRACKET",
    "BACKSLASH", "GRAVE",
    // メディア
    "VOL_UP", "VOL_DOWN", "MUTE", "MEDIA_PLAY", "MEDIA_NEXT", "MEDIA_PREV",
];

pub fn is_known_vk(vk: &str) -> bool {
    VK_DICTIONARY.contains(&vk)
}

// ============================================================================
// アクション型（D4）
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", deny_unknown_fields)]
pub enum Action {
    #[serde(rename = "key")]
    Key { vk: String },
    #[serde(rename = "chord")]
    Chord { keys: Vec<String> },
    #[serde(rename = "mo")]
    Mo { layer: u8 },
    #[serde(rename = "tg")]
    Tg { layer: u8 },
    #[serde(rename = "trans")]
    Trans,
    #[serde(rename = "none")]
    None,
    #[serde(rename = "keymap.switch")]
    KeymapSwitch { id: String },
    #[serde(rename = "keymap.reset")]
    KeymapReset,
}

// ============================================================================
// キーマップ構造
// ============================================================================

pub type KeyId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyDef {
    pub label: String,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub id: u8,
    pub keys: BTreeMap<KeyId, KeyDef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Half {
    pub rows: Vec<Vec<KeyId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Halves {
    pub left: Half,
    pub right: Half,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keymap {
    #[serde(rename = "keymapId")]
    pub keymap_id: String,
    #[serde(default)]
    pub description: String,
    pub halves: Halves,
    pub layers: Vec<Layer>,
}

impl Keymap {
    pub fn layer(&self, id: u8) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }
}

// ============================================================================
// ロード＆検証
// ============================================================================

/// ファイルパスからロード。読み込み失敗もLOAD_JSON_SYNTAX扱い（呼び出し側は
/// 「ファイルが読めない」も「JSONが壊れている」も同じ起動拒否経路で処理してよいため）。
pub fn load_keymap_from_path(path: impl AsRef<Path>) -> Result<Keymap, KeymapError> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).map_err(|error| {
        KeymapError::new(
            LOAD_JSON_SYNTAX,
            format!("{}: failed to read file: {error}", path.display()),
        )
    })?;
    load_keymap_str(&path.display().to_string(), &text)
}

/// テスト・再ロード用に文字列から直接ロードする経路。`source` はcauseに載る識別子
/// （通常はファイルパス）。
pub fn load_keymap_str(source: &str, text: &str) -> Result<Keymap, KeymapError> {
    // [T1-1] JSON構文検証
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|error| KeymapError::new(LOAD_JSON_SYNTAX, format!("{source}: {error}")))?;

    // [T1-2] スキーマ形状検証（serdeのdeny_unknown_fields + 必須フィールド + タグ付きenumで実施）
    let keymap: Keymap = serde_json::from_value(value)
        .map_err(|error| KeymapError::new(LOAD_SCHEMA_INVALID, format!("{source}: {error}")))?;

    // Layer0必須
    if keymap.layer(0).is_none() {
        return Err(KeymapError::new(
            LOAD_SCHEMA_INVALID,
            format!("{source}: layer 0 is required but was not found"),
        ));
    }

    // Layer0にtransは禁止（最下層に透過先が無いため）
    if let Some(layer0) = keymap.layer(0) {
        for (key_id, def) in &layer0.keys {
            if matches!(def.action, Action::Trans) {
                return Err(KeymapError::new(
                    LOAD_SCHEMA_INVALID,
                    format!(
                        "{source}: layer 0 key '{key_id}' must not be trans (no lower layer to fall through to)"
                    ),
                ));
            }
        }
    }

    // [T1-3] vk辞書外チェック＋mo/tg参照先レイヤー存在チェック
    let layer_ids: BTreeSet<u8> = keymap.layers.iter().map(|layer| layer.id).collect();
    for layer in &keymap.layers {
        for (key_id, def) in &layer.keys {
            match &def.action {
                Action::Key { vk } => {
                    if !is_known_vk(vk) {
                        return Err(KeymapError::new(
                            LOAD_VK_UNKNOWN,
                            format!(
                                "{source}: layer {} key '{key_id}': unknown vk '{vk}'",
                                layer.id
                            ),
                        ));
                    }
                }
                Action::Chord { keys } => {
                    for vk in keys {
                        if !is_known_vk(vk) {
                            return Err(KeymapError::new(
                                LOAD_VK_UNKNOWN,
                                format!(
                                    "{source}: layer {} key '{key_id}': unknown vk '{vk}' in chord",
                                    layer.id
                                ),
                            ));
                        }
                    }
                }
                Action::Mo { layer: target } | Action::Tg { layer: target } => {
                    if !layer_ids.contains(target) {
                        return Err(KeymapError::new(
                            LOAD_LAYER_REF_INVALID,
                            format!(
                                "{source}: layer {} key '{key_id}': references undefined layer {target}",
                                layer.id
                            ),
                        ));
                    }
                }
                Action::Trans | Action::None | Action::KeymapSwitch { .. } | Action::KeymapReset => {}
            }
        }
    }

    Ok(keymap)
}

// ============================================================================
// レイヤー状態＋解決エンジン（D3）
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Down,
    Up,
}

/// Hubがkeyboard単位で一元保持する状態。momentary=MO押下中の集合、toggled=TGでONの集合。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LayerState {
    momentary: BTreeSet<u8>,
    toggled: BTreeSet<u8>,
}

impl LayerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn momentary(&self) -> &BTreeSet<u8> {
        &self.momentary
    }

    pub fn toggled(&self) -> &BTreeSet<u8> {
        &self.toggled
    }

    /// keymap.switch / keymap.reset 時にHubが呼ぶ（D10: 切替時はレイヤー状態を0にリセット）。
    pub fn reset(&mut self) {
        self.momentary.clear();
        self.toggled.clear();
    }

    /// 有効レイヤー = {0} ∪ momentary ∪ toggled。
    fn active_layers(&self) -> BTreeSet<u8> {
        let mut set = BTreeSet::new();
        set.insert(0);
        set.extend(self.momentary.iter().copied());
        set.extend(self.toggled.iter().copied());
        set
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolved {
    /// key/chord/keymap.switch/keymap.reset をdownで発火。
    Fire(Action),
    /// mo/tgによる状態変化。呼び出し側はlayer.stateを全クライアントへ配信する。
    LayerChanged,
    /// key/chordのup、あるいはtgのupなど、正常だが何もしない場合。
    Ignored,
    /// keyIdがこのキーマップのどの層にも定義されていない。呼び出し側はKEY_UNKNOWN_ID。
    UnknownKey,
    /// keyIdは存在するが、有効レイヤーのどこにも非trans定義が無く解決先が無い。
    /// 呼び出し側はKEY_RESOLVE_NONEとして記録する（無音の見落としを防ぐため）。
    NoResolution,
}

/// [T1-4] 決定的なレイヤー解決。keymap/stateのみに依存し乱数・時刻を使わないため、
/// 同一の呼び出し列は常に同一の結果列を生む（G5）。
pub fn resolve(keymap: &Keymap, state: &mut LayerState, key_id: &str, edge: Edge) -> Resolved {
    let exists_anywhere = keymap
        .layers
        .iter()
        .any(|layer| layer.keys.contains_key(key_id));
    if !exists_anywhere {
        return Resolved::UnknownKey;
    }

    // 有効レイヤーを番号の大きい順に走査し、非transの定義に当たったら採用（フォールスルー）。
    let active = state.active_layers();
    let mut found: Option<&Action> = None;
    for layer_id in active.iter().rev() {
        if let Some(layer) = keymap.layer(*layer_id) {
            if let Some(def) = layer.keys.get(key_id) {
                if !matches!(def.action, Action::Trans) {
                    found = Some(&def.action);
                    break;
                }
            }
        }
    }

    let Some(action) = found else {
        return Resolved::NoResolution;
    };

    match action {
        Action::Mo { layer } => {
            let layer = *layer;
            match edge {
                Edge::Down => {
                    state.momentary.insert(layer);
                    Resolved::LayerChanged
                }
                Edge::Up => {
                    if state.momentary.remove(&layer) {
                        Resolved::LayerChanged
                    } else {
                        Resolved::Ignored
                    }
                }
            }
        }
        Action::Tg { layer } => {
            let layer = *layer;
            match edge {
                Edge::Down => {
                    if !state.toggled.remove(&layer) {
                        state.toggled.insert(layer);
                    }
                    Resolved::LayerChanged
                }
                Edge::Up => Resolved::Ignored,
            }
        }
        Action::Key { .. }
        | Action::Chord { .. }
        | Action::KeymapSwitch { .. }
        | Action::KeymapReset => match edge {
            Edge::Down => Resolved::Fire(action.clone()),
            Edge::Up => Resolved::Ignored,
        },
        Action::None => Resolved::Ignored,
        Action::Trans => unreachable!("Trans is filtered out during the active-layer walk"),
    }
}

// ============================================================================
// 単体テスト（T1-4。12件以上）
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn layer_with(id: u8, entries: &[(&str, &str, Action)]) -> Layer {
        let mut keys = BTreeMap::new();
        for (key_id, label, action) in entries {
            keys.insert(
                key_id.to_string(),
                KeyDef {
                    label: label.to_string(),
                    action: action.clone(),
                },
            );
        }
        Layer { id, keys }
    }

    fn half(ids: &[&str]) -> Half {
        Half {
            rows: vec![ids.iter().map(|s| s.to_string()).collect()],
        }
    }

    /// resolve()単体テスト用の小さな固定キーマップ:
    /// layer0: K1=key A, K2=MO(1)
    /// layer1: K1=key B (layer0を上書き), K3=key C, K4=TG(2)
    /// layer2: K1=trans（layer1へフォールスルー）
    fn fixture_keymap() -> Keymap {
        Keymap {
            keymap_id: "fixture".into(),
            description: String::new(),
            halves: Halves {
                left: half(&["K1", "K2", "K4"]),
                right: half(&["K3"]),
            },
            layers: vec![
                layer_with(
                    0,
                    &[
                        ("K1", "A", Action::Key { vk: "A".into() }),
                        ("K2", "MO(1)", Action::Mo { layer: 1 }),
                    ],
                ),
                layer_with(
                    1,
                    &[
                        ("K1", "B", Action::Key { vk: "B".into() }),
                        ("K3", "C", Action::Key { vk: "C".into() }),
                        ("K4", "TG(2)", Action::Tg { layer: 2 }),
                    ],
                ),
                layer_with(2, &[("K1", "trans", Action::Trans)]),
            ],
        }
    }

    // 1. layer0の単キー解決
    #[test]
    fn t1_4_layer0_single_key_resolves_to_fire() {
        let keymap = fixture_keymap();
        let mut state = LayerState::new();
        let resolved = resolve(&keymap, &mut state, "K1", Edge::Down);
        assert_eq!(resolved, Resolved::Fire(Action::Key { vk: "A".into() }));
    }

    // 2. chord解決
    #[test]
    fn t1_4_chord_resolves_to_fire() {
        let mut keymap = fixture_keymap();
        keymap.layers[0].keys.insert(
            "K5".into(),
            KeyDef {
                label: "save".into(),
                action: Action::Chord {
                    keys: vec!["CTRL".into(), "S".into()],
                },
            },
        );
        let mut state = LayerState::new();
        let resolved = resolve(&keymap, &mut state, "K5", Edge::Down);
        assert_eq!(
            resolved,
            Resolved::Fire(Action::Chord {
                keys: vec!["CTRL".into(), "S".into()]
            })
        );
    }

    // 3. MO down中はlayer1が勝つ（番号最大優先）
    #[test]
    fn t1_4_mo_down_makes_layer1_win_over_layer0() {
        let keymap = fixture_keymap();
        let mut state = LayerState::new();

        let mo = resolve(&keymap, &mut state, "K2", Edge::Down);
        assert_eq!(mo, Resolved::LayerChanged);
        assert!(state.momentary().contains(&1));

        let resolved = resolve(&keymap, &mut state, "K1", Edge::Down);
        assert_eq!(resolved, Resolved::Fire(Action::Key { vk: "B".into() }));
    }

    // 4. MO up で layer0 に戻る
    #[test]
    fn t1_4_mo_up_returns_to_layer0() {
        let keymap = fixture_keymap();
        let mut state = LayerState::new();

        resolve(&keymap, &mut state, "K2", Edge::Down);
        let up = resolve(&keymap, &mut state, "K2", Edge::Up);
        assert_eq!(up, Resolved::LayerChanged);
        assert!(!state.momentary().contains(&1));

        let resolved = resolve(&keymap, &mut state, "K1", Edge::Down);
        assert_eq!(resolved, Resolved::Fire(Action::Key { vk: "A".into() }));
    }

    // 5. TG でトグルON→OFF
    #[test]
    fn t1_4_tg_toggles_on_then_off() {
        let keymap = fixture_keymap();
        let mut state = LayerState::new();
        // layer1を有効化しないとK4(TG)に到達しないので先にMOで一時的に有効化して押す、
        // という迂遠さを避けるため、layer0にもTGキーを複製したテスト専用キーマップにする。
        let mut keymap = keymap;
        keymap.layers[0].keys.insert(
            "K6".into(),
            KeyDef {
                label: "TG(2)".into(),
                action: Action::Tg { layer: 2 },
            },
        );
        let _ = &keymap;

        let on = resolve(&keymap, &mut state, "K6", Edge::Down);
        assert_eq!(on, Resolved::LayerChanged);
        assert!(state.toggled().contains(&2));

        let up = resolve(&keymap, &mut state, "K6", Edge::Up);
        assert_eq!(up, Resolved::Ignored, "TG up is a no-op");
        assert!(state.toggled().contains(&2), "TG stays on until the next down");

        let off = resolve(&keymap, &mut state, "K6", Edge::Down);
        assert_eq!(off, Resolved::LayerChanged);
        assert!(!state.toggled().contains(&2));
    }

    // 6. trans フォールスルー
    #[test]
    fn t1_4_trans_falls_through_to_lower_layer() {
        let keymap = fixture_keymap();
        let mut state = LayerState::new();
        // layer1とlayer2を両方有効化してK1を押すと、layer2ではtransなので
        // 次に大きい有効レイヤー(layer1)のBへフォールスルーする。
        state.momentary.insert(1);
        state.toggled.insert(2);

        let resolved = resolve(&keymap, &mut state, "K1", Edge::Down);
        assert_eq!(resolved, Resolved::Fire(Action::Key { vk: "B".into() }));
    }

    // 6b. trans が有効レイヤーを飛び越して、より下の有効レイヤーまで届くケース
    // （layer1は有効でないので、layer2のtransはlayer1を素通りしてlayer0のAに落ちる）
    #[test]
    fn t1_4_trans_skips_inactive_layers_down_to_next_active_one() {
        let keymap = fixture_keymap();
        let mut state = LayerState::new();
        state.toggled.insert(2);

        let resolved = resolve(&keymap, &mut state, "K1", Edge::Down);
        assert_eq!(resolved, Resolved::Fire(Action::Key { vk: "A".into() }));
    }

    // 7. 有効レイヤー複数時は番号最大優先
    #[test]
    fn t1_4_highest_active_layer_wins_when_multiple_active() {
        let keymap = fixture_keymap();
        let mut state = LayerState::new();
        state.momentary.insert(1);
        state.toggled.insert(2);
        // layer2はK1=transなのでlayer1のBへフォールスルーするが、layer2自体が
        // 最優先で走査されることを確認するため、layer2にK3の非trans定義を追加して検証する。
        let mut keymap = keymap;
        keymap
            .layers
            .iter_mut()
            .find(|l| l.id == 2)
            .unwrap()
            .keys
            .insert(
                "K3".into(),
                KeyDef {
                    label: "layer2-C".into(),
                    action: Action::Key { vk: "Z".into() },
                },
            );

        let resolved = resolve(&keymap, &mut state, "K3", Edge::Down);
        assert_eq!(
            resolved,
            Resolved::Fire(Action::Key { vk: "Z".into() }),
            "layer2 (highest active) must win over layer1's definition of K3"
        );
    }

    // 8. 未知keyId → UnknownKey
    #[test]
    fn t1_4_unknown_key_id_is_reported() {
        let keymap = fixture_keymap();
        let mut state = LayerState::new();
        let resolved = resolve(&keymap, &mut state, "NOPE", Edge::Down);
        assert_eq!(resolved, Resolved::UnknownKey);
    }

    // 8b. 有効レイヤーに解決先が無い場合はNoResolution（KEY_RESOLVE_NONE相当）
    #[test]
    fn t1_4_no_resolution_when_key_not_defined_in_any_active_layer() {
        let keymap = fixture_keymap();
        let mut state = LayerState::new();
        // K3はlayer1のみに定義されている。layer0だけが有効な状態で押すとNoResolution。
        let resolved = resolve(&keymap, &mut state, "K3", Edge::Down);
        assert_eq!(resolved, Resolved::NoResolution);
    }

    // 9. 辞書外vk → LOAD_VK_UNKNOWN
    #[test]
    fn t1_4_unknown_vk_is_rejected_at_load() {
        let text = r#"{
            "keymapId": "bad_vk",
            "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
            "layers": [ { "id": 0, "keys": { "K1": { "label": "x", "action": { "t": "key", "vk": "NOT_A_KEY" } } } } ]
        }"#;
        let error = load_keymap_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_VK_UNKNOWN);
        assert!(error.cause.contains("K1"));
        assert!(error.cause.contains("NOT_A_KEY"));
    }

    // 10. mo参照先レイヤー不在 → LOAD_LAYER_REF_INVALID
    #[test]
    fn t1_4_mo_referencing_missing_layer_is_rejected_at_load() {
        let text = r#"{
            "keymapId": "bad_ref",
            "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
            "layers": [ { "id": 0, "keys": { "K1": { "label": "x", "action": { "t": "mo", "layer": 9 } } } } ]
        }"#;
        let error = load_keymap_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_LAYER_REF_INVALID);
        assert!(error.cause.contains("K1"));
        assert!(error.cause.contains('9'));
    }

    // 11. JSON構文エラー → LOAD_JSON_SYNTAX
    #[test]
    fn t1_4_json_syntax_error_is_rejected_at_load() {
        let error = load_keymap_str("test", "{ this is not json").unwrap_err();
        assert_eq!(error.code, LOAD_JSON_SYNTAX);
    }

    // 11b. スキーマ形状違反（必須フィールド欠落）→ LOAD_SCHEMA_INVALID
    #[test]
    fn t1_4_missing_required_field_is_rejected_as_schema_invalid() {
        let text = r#"{ "keymapId": "no_halves", "layers": [] }"#;
        let error = load_keymap_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SCHEMA_INVALID);
    }

    // 11c. layer0欠落 → LOAD_SCHEMA_INVALID
    #[test]
    fn t1_4_missing_layer0_is_rejected_as_schema_invalid() {
        let text = r#"{
            "keymapId": "no_layer0",
            "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
            "layers": [ { "id": 1, "keys": {} } ]
        }"#;
        let error = load_keymap_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SCHEMA_INVALID);
    }

    // 11d. layer0にtrans → LOAD_SCHEMA_INVALID
    #[test]
    fn t1_4_trans_on_layer0_is_rejected_as_schema_invalid() {
        let text = r#"{
            "keymapId": "trans_on_zero",
            "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
            "layers": [ { "id": 0, "keys": { "K1": { "label": "x", "action": { "t": "trans" } } } } ]
        }"#;
        let error = load_keymap_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SCHEMA_INVALID);
    }

    // 12. 同一入力列2回で結果一致（決定性。G5）
    #[test]
    fn t1_4_same_input_sequence_is_deterministic() {
        let keymap = fixture_keymap();

        let run = |keymap: &Keymap| -> Vec<Resolved> {
            let mut state = LayerState::new();
            let sequence = [
                ("K2", Edge::Down),
                ("K1", Edge::Down),
                ("K1", Edge::Up),
                ("K2", Edge::Up),
                ("K1", Edge::Down),
            ];
            sequence
                .iter()
                .map(|(key_id, edge)| resolve(keymap, &mut state, key_id, *edge))
                .collect()
        };

        let first = run(&keymap);
        let second = run(&keymap);
        assert_eq!(first, second);
        assert_eq!(
            first,
            vec![
                Resolved::LayerChanged,
                Resolved::Fire(Action::Key { vk: "B".into() }),
                Resolved::Ignored,
                Resolved::LayerChanged,
                Resolved::Fire(Action::Key { vk: "A".into() }),
            ]
        );
    }

    // 実ファイルロード成功確認: keymap_default.json / keymap_writing01.json
    #[test]
    fn real_keymap_default_json_loads_successfully() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../keymaps/keymap_default.json");
        let keymap = load_keymap_from_path(path).expect("keymap_default.json must load");
        assert_eq!(keymap.keymap_id, "default");
        assert!(keymap.layer(0).is_some());
    }

    #[test]
    fn real_keymap_writing01_json_loads_successfully() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../keymaps/keymap_writing01.json"
        );
        let keymap = load_keymap_from_path(path).expect("keymap_writing01.json must load");
        assert_eq!(keymap.keymap_id, "writing01");
        assert!(keymap.layer(0).is_some());
    }

    // vk辞書の網羅性チェック（T2のVKコード表と突き合わせる際の基準）
    #[test]
    fn vk_dictionary_has_no_duplicates() {
        let mut seen = BTreeSet::new();
        for vk in VK_DICTIONARY {
            assert!(seen.insert(*vk), "duplicate vk in dictionary: {vk}");
        }
    }
}
