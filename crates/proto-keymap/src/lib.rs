//! proto-keymap — キーマップ型・JSONロード＆検証・レイヤー解決エンジン（T1、T7でスキーマv2へ）
//!
//! 設計書: brief/keydeck_design_v0.3.md の D13（レイヤー別JSON/スキーマv2）、
//! brief/keydeck_design_v0.4.md の D20（text Action）・D24（グリッドboard）。
//! このコメント群は実装指示の一部。チェックポイントID（T1-1..T1-4、T7-*）は各実装箇所の
//! コメントとテストモジュールに残す。
//!
//! ── T7: スキーマv2（マニフェスト＋レイヤー別JSON）─────────────────
//!
//! ディスク上のフォーマットは2種類に分離される（正は1箇所ずつ）:
//!   1. マニフェスト `keymap_<id>.json` = `KeymapManifest`
//!      { keymapId, kind: "split"|"single", halves|board, layerFiles: [...] }
//!   2. レイヤーファイル `layers/<id>_layer<N>.json` = `LayerFile`
//!      { layer: n, keys: { keyId: {label, action} } }
//!
//! `load_keymap_from_path` はマニフェストを読み、layerFiles をマニフェストと同じ
//! ディレクトリ基準の相対パスとして解決してすべて読み込み、結合してから
//! 従来どおりの検証（Layer0必須・L0にtrans禁止・vk辞書・mo/tg参照先）を1回だけ行う。
//! 検証はファイル単位ではなく「全ファイル読込後に結合して」実施する（D13）。
//! 旧・単一JSONインライン形式（halves+layers埋め込み）の読込は廃止した。
//!
//! ── 元のT1実装（維持）─────────────────────────────────────────
//!
//! 1. アクション型（D4＋D20でtext追加）
//!    enum Action { Key{vk}, Chord{keys}, Mo{layer}, Tg{layer}, Trans, None,
//!                  KeymapSwitch{id}, KeymapReset, Text{string} }
//!    - vkはD4の固定辞書のみ。辞書は本crateに const で持つ（正は1箇所）。
//!    - serdeのtag="t"でJSONの {"t":"key","vk":"A"} 形式に対応させる。
//!    - Text{string}はvk辞書の対象外（adapter側がKEYEVENTF_UNICODEで直接注入するため）。
//!
//! 2. キーマップ構造（v2）
//!    Keymap { keymap_id, kind, halves: Option<Halves>, board: Option<Board>, layers: Vec<Layer> }
//!    Layer { id: u8, keys: Map<KeyId, KeyDef{label, action}> }
//!    Board（D24グリッド式） { cols, keys: Vec<BoardKey{id,row,col,colSpan?,rowSpan?}> }
//!
//! 3. ロード＆検証  load_keymap_from_path(path) -> Result<Keymap, KeymapError>
//!    検証順とエラーコード（D9。cause に「どのファイル・どのkeyId・何が悪いか」を必ず入れる）:
//!    [T1-1] JSON構文（マニフェスト＋各レイヤーファイル） → KeymapError::JsonSyntax (code=LOAD_JSON_SYNTAX)
//!    [T1-2] スキーマ形状（マニフェスト＋各レイヤーファイル＋kind/board整合性）→ LOAD_SCHEMA_INVALID
//!    [T1-3] vk辞書外       → KeymapError::VkUnknown    (code=LOAD_VK_UNKNOWN)
//!           mo/tgの参照先レイヤー不在 → LayerRefInvalid(code=LOAD_LAYER_REF_INVALID)
//!    ※ Layer0必須。Layer0に trans を置くのも SchemaInvalid（最下層に透過先が無いため）。
//!
//! 4. レイヤー状態＋解決エンジン（D3。変更なし）
//!    LayerState { momentary: BTreeSet<u8>, toggled: BTreeSet<u8> }  // Hubが保持する
//!    - key_down/key_up(keyId) を受けて状態遷移し、発火すべき Action を返す:
//!      resolve(keymap, state, key_id, edge) -> Resolved
//!      enum Resolved { Fire(Action), LayerChanged, Ignored, UnknownKey, NoResolution }
//!    - 有効レイヤー = {0} ∪ momentary ∪ toggled のうち番号最大優先・transフォールスルー。
//!    - 決定性: 同じ入力列は常に同じ結果（G5）。乱数・時刻を混ぜない。
//!
//! ── 単体テスト ────────────────────────────────────────────
//!  resolve()系（T1-4、12件以上）は変更なしで維持。
//!  T7で追加: マニフェスト＋レイヤーファイルの正常結合読込／レイヤーファイル欠損→LOAD_JSON_SYNTAX／
//!  結合後にのみ解決できる参照（=ファイル単位で検証していないことの証明）／グリッドboardの読込。

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

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
// アクション型（D4＋D20）
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
    /// D20: IME状態に依存しない直接文字入力。adapter側でKEYEVENTF_UNICODEにより
    /// サロゲートペア対応のdown/up対で送出する。vk辞書の対象外（文字列そのものが許可対象）。
    #[serde(rename = "text")]
    Text { string: String },
}

// ============================================================================
// キーマップ構造（v2: kind + halves(split) | board(single、D24グリッド式)）
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

/// D24: kind="single"の盤面は13列CSS Gridに合わせたグリッド式。
/// row/col/colSpan/rowSpanはそのままCSSの grid-row/grid-column に転記できる値
/// （span省略時は1）。配置の正はmockのgrid-column/grid-row指定。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Board {
    pub cols: u8,
    pub keys: Vec<BoardKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardKey {
    pub id: KeyId,
    pub row: u8,
    pub col: u8,
    #[serde(rename = "colSpan", default = "one_u8", skip_serializing_if = "is_one_u8")]
    pub col_span: u8,
    #[serde(rename = "rowSpan", default = "one_u8", skip_serializing_if = "is_one_u8")]
    pub row_span: u8,
}

fn one_u8() -> u8 {
    1
}

fn is_one_u8(value: &u8) -> bool {
    *value == 1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeymapKind {
    Split,
    Single,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keymap {
    #[serde(rename = "keymapId")]
    pub keymap_id: String,
    #[serde(default)]
    pub description: String,
    pub kind: KeymapKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub halves: Option<Halves>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<Board>,
    pub layers: Vec<Layer>,
}

impl Keymap {
    pub fn layer(&self, id: u8) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }
}

// ============================================================================
// ディスク上フォーマット（T7）: マニフェスト＋レイヤーファイル
// ============================================================================

/// マニフェスト `keymap_<id>.json` の形。halves/boardはkindに応じて片方だけ必須
/// （検証は load_keymap_with 内で行う。ここではserdeレベルの形状のみ）。
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeymapManifest {
    #[serde(rename = "keymapId")]
    keymap_id: String,
    #[serde(default)]
    description: String,
    kind: KeymapKind,
    #[serde(default)]
    halves: Option<Halves>,
    #[serde(default)]
    board: Option<Board>,
    #[serde(rename = "layerFiles")]
    layer_files: Vec<String>,
}

/// レイヤーファイル `layers/<id>_layer<N>.json` の形（schemas/layer.schema.json）。
/// マニフェスト内埋め込みの `Layer`（フィールド名 "id"）とは異なり、
/// ディスク上は D13 の指定どおりフィールド名 "layer" を使う。
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LayerFile {
    layer: u8,
    #[serde(default)]
    #[allow(dead_code)]
    description: String,
    keys: BTreeMap<KeyId, KeyDef>,
}

// ============================================================================
// ロード＆検証
// ============================================================================

/// ファイルパスからロード。マニフェストを読み、layerFilesをマニフェストと同じ
/// ディレクトリ基準の相対パスとして解決してすべて読み込み、結合してから検証する。
pub fn load_keymap_from_path(path: impl AsRef<Path>) -> Result<Keymap, KeymapError> {
    let path = path.as_ref();
    let manifest_text = std::fs::read_to_string(path).map_err(|error| {
        KeymapError::new(
            LOAD_JSON_SYNTAX,
            format!("{}: failed to read file: {error}", path.display()),
        )
    })?;
    let base_dir: PathBuf = path
        .parent()
        .map(|parent| parent.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let source = path.display().to_string();

    load_keymap_with(&source, &manifest_text, |relative| {
        let layer_path = base_dir.join(relative);
        std::fs::read_to_string(&layer_path).map_err(|error| {
            KeymapError::new(
                LOAD_JSON_SYNTAX,
                format!(
                    "{source} -> {relative} ({}): failed to read layer file: {error}",
                    layer_path.display()
                ),
            )
        })
    })
}

/// マニフェストのテキストと、相対パス→レイヤーファイルのテキストを返すローダー関数から
/// キーマップを構築する。`load_keymap_from_path` はディスクI/Oでこの関数を呼び出し、
/// 単体テストはメモリ上の文字列で同じ経路を検証できる（旧`load_keymap_str`の後継）。
pub fn load_keymap_with(
    source: &str,
    manifest_text: &str,
    mut layer_loader: impl FnMut(&str) -> Result<String, KeymapError>,
) -> Result<Keymap, KeymapError> {
    // [T1-1] マニフェストのJSON構文検証
    let manifest_value: serde_json::Value = serde_json::from_str(manifest_text)
        .map_err(|error| KeymapError::new(LOAD_JSON_SYNTAX, format!("{source}: {error}")))?;

    // [T1-2] マニフェストのスキーマ形状検証
    let manifest: KeymapManifest = serde_json::from_value(manifest_value).map_err(|error| {
        KeymapError::new(LOAD_SCHEMA_INVALID, format!("{source} (manifest): {error}"))
    })?;

    if manifest.layer_files.is_empty() {
        return Err(KeymapError::new(
            LOAD_SCHEMA_INVALID,
            format!("{source}: layerFiles must not be empty"),
        ));
    }

    // T7-1: kindごとにhalves/boardのどちらが必須かを検証する。
    match manifest.kind {
        KeymapKind::Split => {
            if manifest.halves.is_none() {
                return Err(KeymapError::new(
                    LOAD_SCHEMA_INVALID,
                    format!("{source}: kind=\"split\" requires 'halves'"),
                ));
            }
            if manifest.board.is_some() {
                return Err(KeymapError::new(
                    LOAD_SCHEMA_INVALID,
                    format!("{source}: kind=\"split\" must not have 'board'"),
                ));
            }
        }
        KeymapKind::Single => {
            if manifest.board.is_none() {
                return Err(KeymapError::new(
                    LOAD_SCHEMA_INVALID,
                    format!("{source}: kind=\"single\" requires 'board'"),
                ));
            }
            if manifest.halves.is_some() {
                return Err(KeymapError::new(
                    LOAD_SCHEMA_INVALID,
                    format!("{source}: kind=\"single\" must not have 'halves'"),
                ));
            }
        }
    }

    // T7-2: グリッドboardの基本整合性（D24）。
    if let Some(board) = &manifest.board {
        if board.cols == 0 {
            return Err(KeymapError::new(
                LOAD_SCHEMA_INVALID,
                format!("{source}: board.cols must be at least 1"),
            ));
        }
        let mut seen_ids = BTreeSet::new();
        for key in &board.keys {
            if !seen_ids.insert(key.id.clone()) {
                return Err(KeymapError::new(
                    LOAD_SCHEMA_INVALID,
                    format!("{source}: board has duplicate key id '{}'", key.id),
                ));
            }
            if key.row == 0 || key.col == 0 || key.row_span == 0 || key.col_span == 0 {
                return Err(KeymapError::new(
                    LOAD_SCHEMA_INVALID,
                    format!(
                        "{source}: board key '{}' has a zero row/col/rowSpan/colSpan",
                        key.id
                    ),
                ));
            }
            if key.col as u16 + key.col_span as u16 - 1 > board.cols as u16 {
                return Err(KeymapError::new(
                    LOAD_SCHEMA_INVALID,
                    format!(
                        "{source}: board key '{}' (col={}, colSpan={}) exceeds cols={}",
                        key.id, key.col, key.col_span, board.cols
                    ),
                ));
            }
        }
    }

    // レイヤーファイルの読込＋構文/形状検証。
    let mut layers: Vec<Layer> = Vec::with_capacity(manifest.layer_files.len());
    let mut seen_layer_ids = BTreeSet::new();
    for relative in &manifest.layer_files {
        let text = layer_loader(relative)?;

        // [T1-1] レイヤーファイルのJSON構文検証
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|error| KeymapError::new(LOAD_JSON_SYNTAX, format!("{source} -> {relative}: {error}")))?;

        // [T1-2] レイヤーファイルのスキーマ形状検証
        let layer_file: LayerFile = serde_json::from_value(value).map_err(|error| {
            KeymapError::new(LOAD_SCHEMA_INVALID, format!("{source} -> {relative}: {error}"))
        })?;

        if !seen_layer_ids.insert(layer_file.layer) {
            return Err(KeymapError::new(
                LOAD_SCHEMA_INVALID,
                format!(
                    "{source} -> {relative}: duplicate layer id {} (already provided by another layerFile)",
                    layer_file.layer
                ),
            ));
        }

        layers.push(Layer {
            id: layer_file.layer,
            keys: layer_file.keys,
        });
    }

    let keymap = Keymap {
        keymap_id: manifest.keymap_id,
        description: manifest.description,
        kind: manifest.kind,
        halves: manifest.halves,
        board: manifest.board,
        layers,
    };

    // 結合後の検証（D13: 全ファイル読込後に結合して従来どおり実施）。
    validate_merged(source, &keymap)?;

    Ok(keymap)
}

/// Layer0必須／L0にtrans禁止／vk辞書／mo・tg参照先。従来のload_keymap_str相当の検証を、
/// マニフェスト＋複数レイヤーファイルを結合した後のKeymapに対して1回だけ行う。
fn validate_merged(source: &str, keymap: &Keymap) -> Result<(), KeymapError> {
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
                Action::Trans
                | Action::None
                | Action::KeymapSwitch { .. }
                | Action::KeymapReset
                | Action::Text { .. } => {}
            }
        }
    }

    Ok(())
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
    /// key/chord/text/keymap.switch/keymap.reset をdownで発火。
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
        | Action::Text { .. }
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
// 単体テスト
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
            kind: KeymapKind::Split,
            halves: Some(Halves {
                left: half(&["K1", "K2", "K4"]),
                right: half(&["K3"]),
            }),
            board: None,
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

    /// テスト用: マニフェストテキスト＋メモリ上のレイヤーファイル一覧からキーマップを組み立てる。
    /// 旧`load_keymap_str`（単一インラインJSON）の後継で、ディスクI/O無しに
    /// load_keymap_withの経路（結合＋検証）を検証できる。
    fn load_test_keymap(manifest: &str, layer_files: &[(&str, &str)]) -> Result<Keymap, KeymapError> {
        load_keymap_with("test", manifest, |relative| {
            layer_files
                .iter()
                .find(|(name, _)| *name == relative)
                .map(|(_, contents)| contents.to_string())
                .ok_or_else(|| {
                    KeymapError::new(
                        LOAD_JSON_SYNTAX,
                        format!("test: layer file not found in fixture: {relative}"),
                    )
                })
        })
    }

    /// 一時ディレクトリにマニフェスト＋レイヤーファイルを書き出す（実ファイルI/Oが必要なテスト用）。
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let n = COUNTER.fetch_add(1, Ordering::SeqCst);
            let dir = std::env::temp_dir().join(format!(
                "keydeck_keymap_test_{tag}_{}_{n}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).expect("create temp dir for keymap test");
            Self(dir)
        }

        fn write(&self, name: &str, contents: &str) -> PathBuf {
            let path = self.0.join(name);
            std::fs::write(&path, contents).expect("write temp fixture file");
            path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
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

    // 8c. text Actionもkey/chordと同様にdownで発火・upは無視
    #[test]
    fn t1_4_text_action_resolves_to_fire_on_down_and_ignored_on_up() {
        let mut keymap = fixture_keymap();
        keymap.layers[0].keys.insert(
            "K7".into(),
            KeyDef {
                label: "(".into(),
                action: Action::Text { string: "(".into() },
            },
        );
        let mut state = LayerState::new();
        let down = resolve(&keymap, &mut state, "K7", Edge::Down);
        assert_eq!(down, Resolved::Fire(Action::Text { string: "(".into() }));
        let up = resolve(&keymap, &mut state, "K7", Edge::Up);
        assert_eq!(up, Resolved::Ignored);
    }

    // 9. 辞書外vk → LOAD_VK_UNKNOWN
    #[test]
    fn t1_4_unknown_vk_is_rejected_at_load() {
        let manifest = r#"{
            "keymapId": "bad_vk",
            "kind": "split",
            "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
            "layerFiles": ["layer0.json"]
        }"#;
        let layer0 = r#"{ "layer": 0, "keys": { "K1": { "label": "x", "action": { "t": "key", "vk": "NOT_A_KEY" } } } }"#;
        let error = load_test_keymap(manifest, &[("layer0.json", layer0)]).unwrap_err();
        assert_eq!(error.code, LOAD_VK_UNKNOWN);
        assert!(error.cause.contains("K1"));
        assert!(error.cause.contains("NOT_A_KEY"));
    }

    // 10. mo参照先レイヤー不在 → LOAD_LAYER_REF_INVALID
    #[test]
    fn t1_4_mo_referencing_missing_layer_is_rejected_at_load() {
        let manifest = r#"{
            "keymapId": "bad_ref",
            "kind": "split",
            "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
            "layerFiles": ["layer0.json"]
        }"#;
        let layer0 = r#"{ "layer": 0, "keys": { "K1": { "label": "x", "action": { "t": "mo", "layer": 9 } } } }"#;
        let error = load_test_keymap(manifest, &[("layer0.json", layer0)]).unwrap_err();
        assert_eq!(error.code, LOAD_LAYER_REF_INVALID);
        assert!(error.cause.contains("K1"));
        assert!(error.cause.contains('9'));
    }

    // 11. JSON構文エラー（マニフェスト側）→ LOAD_JSON_SYNTAX
    #[test]
    fn t1_4_json_syntax_error_is_rejected_at_load() {
        let error = load_test_keymap("{ this is not json", &[]).unwrap_err();
        assert_eq!(error.code, LOAD_JSON_SYNTAX);
    }

    // 11b. スキーマ形状違反（必須フィールド欠落）→ LOAD_SCHEMA_INVALID
    #[test]
    fn t1_4_missing_required_field_is_rejected_as_schema_invalid() {
        let error = load_test_keymap(r#"{ "keymapId": "no_kind", "layerFiles": [] }"#, &[]).unwrap_err();
        assert_eq!(error.code, LOAD_SCHEMA_INVALID);
    }

    // 11c. layer0欠落 → LOAD_SCHEMA_INVALID
    #[test]
    fn t1_4_missing_layer0_is_rejected_as_schema_invalid() {
        let manifest = r#"{
            "keymapId": "no_layer0",
            "kind": "split",
            "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
            "layerFiles": ["layer1.json"]
        }"#;
        let layer1 = r#"{ "layer": 1, "keys": {} }"#;
        let error = load_test_keymap(manifest, &[("layer1.json", layer1)]).unwrap_err();
        assert_eq!(error.code, LOAD_SCHEMA_INVALID);
    }

    // 11d. layer0にtrans → LOAD_SCHEMA_INVALID
    #[test]
    fn t1_4_trans_on_layer0_is_rejected_as_schema_invalid() {
        let manifest = r#"{
            "keymapId": "trans_on_zero",
            "kind": "split",
            "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
            "layerFiles": ["layer0.json"]
        }"#;
        let layer0 = r#"{ "layer": 0, "keys": { "K1": { "label": "x", "action": { "t": "trans" } } } }"#;
        let error = load_test_keymap(manifest, &[("layer0.json", layer0)]).unwrap_err();
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

    // 実ファイルロード成功確認: keymap_default.json / keymap_writing01.json（T7: マニフェスト形式）
    #[test]
    fn real_keymap_default_json_loads_successfully() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../keymaps/keymap_default.json");
        let keymap = load_keymap_from_path(path).expect("keymap_default.json must load");
        assert_eq!(keymap.keymap_id, "default");
        assert_eq!(keymap.kind, KeymapKind::Split);
        assert!(keymap.halves.is_some());
        assert!(keymap.board.is_none());
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

    #[test]
    fn real_keymap_ipad01_vol12_json_loads_successfully() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../keymaps/keymap_ipad01_vol12.json"
        );
        let keymap = load_keymap_from_path(path).expect("keymap_ipad01_vol12.json must load");
        assert_eq!(keymap.keymap_id, "ipad01_vol12");
        assert_eq!(keymap.kind, KeymapKind::Single);
        assert!(keymap.board.is_some());
        assert!(keymap.halves.is_none());
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

    // ── T7: マニフェスト＋レイヤーファイル読込のテスト ──────────────────

    // マニフェスト読込: 複数レイヤーファイルが正しく結合される。
    #[test]
    fn t7_manifest_and_layer_files_load_and_merge_successfully() {
        let dir = TempDir::new("manifest_ok");
        dir.write(
            "layer0.json",
            r#"{ "layer": 0, "keys": { "K1": { "label": "A", "action": { "t": "key", "vk": "A" } } } }"#,
        );
        dir.write(
            "layer1.json",
            r#"{ "layer": 1, "keys": { "K1": { "label": "B", "action": { "t": "key", "vk": "B" } } } }"#,
        );
        let manifest_path = dir.write(
            "keymap_test.json",
            r#"{
                "keymapId": "manifest_test",
                "kind": "split",
                "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
                "layerFiles": ["layer0.json", "layer1.json"]
            }"#,
        );

        let keymap = load_keymap_from_path(&manifest_path).expect("manifest+layers must load");
        assert_eq!(keymap.keymap_id, "manifest_test");
        assert_eq!(keymap.layers.len(), 2);
        assert!(keymap.layer(0).is_some());
        assert!(keymap.layer(1).is_some());
    }

    // レイヤーファイル欠損 → LOAD_JSON_SYNTAX（実ファイルI/O経路）
    #[test]
    fn t7_missing_layer_file_is_load_json_syntax_error() {
        let dir = TempDir::new("missing_layer");
        let manifest_path = dir.write(
            "keymap_test.json",
            r#"{
                "keymapId": "missing_layer",
                "kind": "split",
                "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
                "layerFiles": ["does_not_exist.json"]
            }"#,
        );

        let error = load_keymap_from_path(&manifest_path).unwrap_err();
        assert_eq!(error.code, LOAD_JSON_SYNTAX);
        assert!(error.cause.contains("does_not_exist.json"));
    }

    // 結合後検証: K2(layer0)のmo(1)はlayer1ファイルが読み込まれて初めて解決できる。
    // ファイル単位で検証していたら（layer0.json単体を見た時点では）layer1は未知に見えるはず。
    #[test]
    fn t7_validation_runs_after_merging_all_layer_files() {
        let dir = TempDir::new("merge_validate");
        dir.write(
            "layer0.json",
            r#"{ "layer": 0, "keys": {
                "K1": { "label": "A", "action": { "t": "key", "vk": "A" } },
                "K2": { "label": "MO(1)", "action": { "t": "mo", "layer": 1 } }
            } }"#,
        );
        dir.write(
            "layer1.json",
            r#"{ "layer": 1, "keys": { "K1": { "label": "B", "action": { "t": "key", "vk": "B" } } } }"#,
        );
        let manifest_path = dir.write(
            "keymap_test.json",
            r#"{
                "keymapId": "merge_validate",
                "kind": "split",
                "halves": { "left": { "rows": [["K1", "K2"]] }, "right": { "rows": [[]] } },
                "layerFiles": ["layer0.json", "layer1.json"]
            }"#,
        );

        let keymap = load_keymap_from_path(&manifest_path)
            .expect("mo(1) must resolve once every layer file has been merged");
        assert_eq!(keymap.layers.len(), 2);
    }

    // グリッドboard（D24）の読込: kind=singleのboardが正しくデシリアライズされ、
    // colSpan/rowSpan省略時は1になる。
    #[test]
    fn t7_grid_board_single_kind_loads_successfully() {
        let dir = TempDir::new("grid_board");
        dir.write(
            "layer0.json",
            r#"{ "layer": 0, "keys": {
                "K101": { "label": "1", "action": { "t": "key", "vk": "1" } },
                "K113": { "label": "Enter", "action": { "t": "key", "vk": "ENTER" } }
            } }"#,
        );
        let manifest_path = dir.write(
            "keymap_test.json",
            r#"{
                "keymapId": "grid_test",
                "kind": "single",
                "board": { "cols": 13, "keys": [
                    { "id": "K101", "row": 1, "col": 1 },
                    { "id": "K113", "row": 2, "col": 13, "rowSpan": 2 }
                ] },
                "layerFiles": ["layer0.json"]
            }"#,
        );

        let keymap = load_keymap_from_path(&manifest_path).expect("grid board keymap must load");
        assert_eq!(keymap.kind, KeymapKind::Single);
        let board = keymap.board.as_ref().expect("board must be present for kind=single");
        assert_eq!(board.cols, 13);
        let enter = board.keys.iter().find(|k| k.id == "K113").unwrap();
        assert_eq!(enter.row_span, 2, "explicit rowSpan must be preserved");
        assert_eq!(enter.col_span, 1, "colSpan defaults to 1 when omitted");
        let one = board.keys.iter().find(|k| k.id == "K101").unwrap();
        assert_eq!(one.row_span, 1, "rowSpan defaults to 1 when omitted");
    }

    // kind=singleでboard欠落 → LOAD_SCHEMA_INVALID
    #[test]
    fn t7_kind_single_without_board_is_schema_invalid() {
        let manifest = r#"{ "keymapId": "x", "kind": "single", "layerFiles": ["layer0.json"] }"#;
        let error = load_test_keymap(manifest, &[]).unwrap_err();
        assert_eq!(error.code, LOAD_SCHEMA_INVALID);
    }

    // kind=splitでboardが混在 → LOAD_SCHEMA_INVALID
    #[test]
    fn t7_kind_split_with_board_is_schema_invalid() {
        let manifest = r#"{
            "keymapId": "x",
            "kind": "split",
            "halves": { "left": { "rows": [["K1"]] }, "right": { "rows": [[]] } },
            "board": { "cols": 2, "keys": [{ "id": "K1", "row": 1, "col": 1 }] },
            "layerFiles": ["layer0.json"]
        }"#;
        let error = load_test_keymap(manifest, &[]).unwrap_err();
        assert_eq!(error.code, LOAD_SCHEMA_INVALID);
    }
}
