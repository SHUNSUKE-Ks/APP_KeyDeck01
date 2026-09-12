//! レイアウト（画面の区画割り）の型・ロード＆検証。P-005 段階B。
//!
//! ■ 何のためか
//! これまで「画面 = 1枚の手書きHTML」で、どの部品をどこへ置くかはHTML/CSSに直書きだった
//! （`/panel` の上下2分割がその例）。複合面を1枚増やすたびにHTMLを書き足す必要があった。
//! ここで導入する `layouts/layout_*.json` は、**画面をグリッドに区切り、各区画へ部品を
//! 1つだけ割り当てる**という形をJSONで持つ。`static/layout.html` 1枚が全レイアウトを描く。
//!
//! ■ 三層（ユーザー裁定・P-005 §2）
//!   - **section**（区画）… 画面上の1区画。**中に部品をちょうど1つ持つ**（入れ子にしない）
//!   - **Component**（部品）… keyboard / deck / trackball
//!   - **slot**（スロット）… Deck部品の中のボタン1つ（既存の意味のまま）
//!
//! ■ グリッドの形はD24（keymapの`board`）と同じ
//! `row`/`col`/`colSpan`/`rowSpan` をそのままCSS Gridへ転記できる。1段上に持ち上げただけ
//! なので、読む側の負担が増えない。
//!
//! ■ プロトコルは増やしていない
//! `surface.config` に `layouts` を1つ足すだけ。中身（keymap/deck）は既に配信済み。

use serde::{Deserialize, Serialize};

use crate::error::LOAD_JSON_SYNTAX;

pub const LOAD_LAYOUT_INVALID: &str = "LOAD_LAYOUT_INVALID";
pub const LOAD_LAYOUT_REF_UNKNOWN: &str = "LOAD_LAYOUT_REF_UNKNOWN";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutError {
    pub code: &'static str,
    pub cause: String,
}

impl LayoutError {
    fn new(code: &'static str, cause: impl Into<String>) -> Self {
        Self { code, cause: cause.into() }
    }
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.cause)
    }
}

impl std::error::Error for LayoutError {}

/// 部品の種類。増やすときはここと `static/layout.html` の描画分岐を同時に足す。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComponentKind {
    /// `ref` = keymapId。13列グリッドの一枚キーボードでも、3×3の十字キーでも同じ扱い
    /// （十字キーは「小さいboardを持つkeyboard」でしかなく、新しい部品種別ではない）。
    Keyboard,
    /// `ref` = deckId。`render`（grid/list）はDeck JSON側が持つ。
    Deck,
    /// `ref` = surfaceId（`surfaces/*.json`）。D28のstate駆動面をそのまま区画に埋める。
    Trackball,
    /// `ref` = keymapId（`jog` を持つもの）。回すと `CW`/`CCW` の2キーを押すだけの部品で、
    /// 送るものはキーマップが決める。新しい通信は増やさない（不変条件1）。
    Jog,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub kind: ComponentKind,
    /// 参照先のid。実在するかはstartup側（全データをロードした後）で検証する。
    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub id: String,
    pub row: u8,
    pub col: u8,
    #[serde(rename = "colSpan", default = "one", skip_serializing_if = "is_one")]
    pub col_span: u8,
    #[serde(rename = "rowSpan", default = "one", skip_serializing_if = "is_one")]
    pub row_span: u8,
    pub component: Component,
}

fn one() -> u8 {
    1
}

fn is_one(v: &u8) -> bool {
    *v == 1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutGrid {
    pub cols: u8,
    pub rows: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    #[serde(rename = "layoutId")]
    pub layout_id: String,
    #[serde(default)]
    pub description: String,
    pub grid: LayoutGrid,
    pub sections: Vec<Section>,
}

pub fn load_layout_from_path(path: impl AsRef<std::path::Path>) -> Result<Layout, LayoutError> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path).map_err(|error| {
        LayoutError::new(
            LOAD_JSON_SYNTAX,
            format!("{}: failed to read file: {error}", path.display()),
        )
    })?;
    load_layout_str(&path.display().to_string(), &text)
}

pub fn load_layout_str(source: &str, text: &str) -> Result<Layout, LayoutError> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|error| LayoutError::new(LOAD_JSON_SYNTAX, format!("{source}: {error}")))?;
    let layout: Layout = serde_json::from_value(value)
        .map_err(|error| LayoutError::new(LOAD_LAYOUT_INVALID, format!("{source}: {error}")))?;

    if layout.sections.is_empty() {
        return Err(LayoutError::new(
            LOAD_LAYOUT_INVALID,
            format!("{source}: layout '{}' has no sections", layout.layout_id),
        ));
    }

    // section idの重複禁止（`key.press`/`deck.press`がidで区画を引くため、
    // 重複するとどちらが当たるか不定になる）。
    let mut seen = std::collections::BTreeSet::new();
    for section in &layout.sections {
        if !seen.insert(section.id.as_str()) {
            return Err(LayoutError::new(
                LOAD_LAYOUT_INVALID,
                format!("{source}: duplicate section id '{}'", section.id),
            ));
        }
    }

    // グリッドからのはみ出しを拒否する。CSS Gridははみ出しても勝手に行を足して
    // 描いてしまうため、JSONの時点で止めないと「なぜか下にずれる」形で気づきにくく壊れる。
    for section in &layout.sections {
        if section.row == 0 || section.col == 0 || section.row_span == 0 || section.col_span == 0 {
            return Err(LayoutError::new(
                LOAD_LAYOUT_INVALID,
                format!(
                    "{source}: section '{}': row/col/rowSpan/colSpan are 1-based and must be >= 1",
                    section.id
                ),
            ));
        }
        let right = section.col as u16 + section.col_span as u16 - 1;
        let bottom = section.row as u16 + section.row_span as u16 - 1;
        if right > layout.grid.cols as u16 || bottom > layout.grid.rows as u16 {
            return Err(LayoutError::new(
                LOAD_LAYOUT_INVALID,
                format!(
                    "{source}: section '{}' spans to ({bottom},{right}) which is outside the {}x{} grid",
                    section.id, layout.grid.rows, layout.grid.cols
                ),
            ));
        }
    }

    // 重なりを拒否する。「パズルのように入れ替える」用途では、入れ替えミスで区画が
    // 重なるのが最も起きやすい事故で、しかも画面上は「片方が消えた」ようにしか見えない。
    let mut occupied: std::collections::BTreeMap<(u8, u8), &str> = std::collections::BTreeMap::new();
    for section in &layout.sections {
        for r in section.row..section.row + section.row_span {
            for c in section.col..section.col + section.col_span {
                if let Some(other) = occupied.insert((r, c), section.id.as_str()) {
                    return Err(LayoutError::new(
                        LOAD_LAYOUT_INVALID,
                        format!(
                            "{source}: sections '{other}' and '{}' both occupy cell (row {r}, col {c})",
                            section.id
                        ),
                    ));
                }
            }
        }
    }

    Ok(layout)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> &'static str {
        r#"{
            "layoutId": "t",
            "grid": { "cols": 4, "rows": 2 },
            "sections": [
                { "id": "A", "row": 1, "col": 1, "colSpan": 2, "component": { "kind": "deck", "ref": "default" } },
                { "id": "B", "row": 1, "col": 3, "colSpan": 2, "component": { "kind": "trackball", "ref": "tb01" } },
                { "id": "C", "row": 2, "col": 1, "colSpan": 4, "component": { "kind": "keyboard", "ref": "ipad01_vol12" } }
            ]
        }"#
    }

    #[test]
    fn loads_a_valid_layout() {
        let layout = load_layout_str("test", sample()).unwrap();
        assert_eq!(layout.sections.len(), 3);
        let b = layout.sections.iter().find(|s| s.id == "B").unwrap();
        assert_eq!(b.component.kind, ComponentKind::Trackball);
        // spanを省略した区画は1になる
        assert_eq!(layout.sections[0].row_span, 1);
    }

    #[test]
    fn rejects_duplicate_section_id() {
        let text = r#"{
            "layoutId": "t", "grid": { "cols": 2, "rows": 1 },
            "sections": [
                { "id": "A", "row": 1, "col": 1, "component": { "kind": "deck", "ref": "d" } },
                { "id": "A", "row": 1, "col": 2, "component": { "kind": "deck", "ref": "d" } }
            ] }"#;
        let error = load_layout_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_LAYOUT_INVALID);
        assert!(error.cause.contains("duplicate"));
    }

    /// はみ出しは「なぜか下にずれる」形でしか見えないので、JSONの時点で止める。
    #[test]
    fn rejects_section_outside_the_grid() {
        let text = r#"{
            "layoutId": "t", "grid": { "cols": 2, "rows": 1 },
            "sections": [
                { "id": "A", "row": 1, "col": 2, "colSpan": 2, "component": { "kind": "deck", "ref": "d" } }
            ] }"#;
        let error = load_layout_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_LAYOUT_INVALID);
        assert!(error.cause.contains("outside"));
    }

    /// 「パズルのように入れ替える」で最も起きやすい事故。画面上は片方が消えたようにしか
    /// 見えないので、ロード時に止めないと原因究明に時間を取られる。
    #[test]
    fn rejects_overlapping_sections() {
        let text = r#"{
            "layoutId": "t", "grid": { "cols": 3, "rows": 1 },
            "sections": [
                { "id": "A", "row": 1, "col": 1, "colSpan": 2, "component": { "kind": "deck", "ref": "d" } },
                { "id": "B", "row": 1, "col": 2, "colSpan": 2, "component": { "kind": "deck", "ref": "d" } }
            ] }"#;
        let error = load_layout_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_LAYOUT_INVALID);
        assert!(error.cause.contains("both occupy"), "cause: {}", error.cause);
    }

    #[test]
    fn rejects_empty_sections() {
        let text = r#"{ "layoutId": "t", "grid": { "cols": 1, "rows": 1 }, "sections": [] }"#;
        assert_eq!(load_layout_str("test", text).unwrap_err().code, LOAD_LAYOUT_INVALID);
    }

    #[test]
    fn real_layout_files_load_successfully() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../layouts");
        let mut checked = 0;
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let is_layout = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("layout_") && n.ends_with(".json"));
                if is_layout {
                    load_layout_from_path(&path)
                        .unwrap_or_else(|e| panic!("{} must load: {e}", path.display()));
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "expected at least one layouts/layout_*.json in the repository");
    }
}
