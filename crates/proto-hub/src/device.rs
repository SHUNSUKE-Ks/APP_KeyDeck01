//! 端末スロット（P-008 段階A・2026-09-30 ユーザー裁定）。
//!
//! ## 何のためか
//!
//! Hub はこれまで端末を区別していなかった。繋がった WebSocket に通し番号を振るだけで、
//! `layout.switch` は `/layout` を開いている**全端末へ一斉に**届いていた。
//! iPad で「YouTube へ」を押すと、机の Android も YouTube の盤面に変わってしまう。
//!
//! ここでは `devices/devices.json`（**手で書く・Hub は読むだけ**）に最大3台の端末を決めておき、
//! 端末は URL の `device=<id>` で自分のスロットを名乗る。分割キーボードの `half=left` と同じ形。
//!
//! ## 守っていること
//!
//! - **ファイルが無ければ今の動きのまま**（端末の区別なし・一斉送信）。既存の使い方を壊さない
//! - 端末が名乗れるのは**登録済みの id だけ**。未知の id の接続は拒否する（`WS_DEVICE_UNKNOWN`）。
//!   id は「何のキーが出るか」に関わらない値なので、不変条件1（位置IDしか送れない）とは衝突しない
//! - 書き込み口は増やさない。devices.json は起動時に読むだけ
//! - `defaultLayout` は実在する盤面だけ（起動時に確かめる）

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

pub const LOAD_DEVICE_SCHEMA_INVALID: &str = "LOAD_DEVICE_SCHEMA_INVALID";

/// 登録できる端末の上限（iPad 1台＋Android 2台）。
pub const MAX_DEVICES: usize = 3;

/// `devices/` の中で読むファイル名。これ以外（`devices.example.json` など）は読まない。
pub const DEVICES_FILE: &str = "devices.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceError {
    pub code: &'static str,
    pub cause: String,
}

impl DeviceError {
    fn new(cause: impl Into<String>) -> Self {
        Self { code: LOAD_DEVICE_SCHEMA_INVALID, cause: cause.into() }
    }
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.cause)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceKind {
    Tablet,
    Phone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Landscape,
    Portrait,
}

/// 端末1台。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceSlot {
    pub id: String,
    /// 画面に出す名前（例「iPad 12.9（リビング）」）。
    pub label: String,
    pub kind: DeviceKind,
    pub orientation: Orientation,
    /// この端末に最初に出す盤面。省略すると Hub 全体の既定に従う。
    #[serde(rename = "defaultLayout", default, skip_serializing_if = "Option::is_none")]
    pub default_layout: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DevicesFile {
    /// 人向けの覚書。中身は使わない。
    #[serde(default)]
    #[allow(dead_code)]
    description: String,
    devices: Vec<DeviceSlot>,
}

/// 登録済みの端末（並びはファイルの順）。空＝端末の区別をしない（従来どおり）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeviceRegistry {
    slots: Vec<DeviceSlot>,
}

impl DeviceRegistry {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn get(&self, id: &str) -> Option<&DeviceSlot> {
        self.slots.iter().find(|slot| slot.id == id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.get(id).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = &DeviceSlot> {
        self.slots.iter()
    }
}

/// `[a-z0-9_]{1,32}`。URL に載り、ログと画面に出るので layoutId と同じ種類の厳しさにする。
pub fn device_id_is_safe(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 32
        && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// 文字列から読む（テストと `load_devices` の共通の入口）。
/// `layouts` は起動時に読み込んだ盤面。`defaultLayout` の実在確認に使う。
pub fn load_devices_str<L>(
    source: &str,
    text: &str,
    layouts: &BTreeMap<String, L>,
) -> Result<DeviceRegistry, DeviceError> {
    let file: DevicesFile = serde_json::from_str(text)
        .map_err(|error| DeviceError::new(format!("{source}: {error}")))?;
    if file.devices.len() > MAX_DEVICES {
        return Err(DeviceError::new(format!(
            "{source}: {} devices listed but at most {MAX_DEVICES} are allowed",
            file.devices.len()
        )));
    }
    let mut seen = BTreeSet::new();
    for slot in &file.devices {
        if !device_id_is_safe(&slot.id) {
            return Err(DeviceError::new(format!(
                "{source}: device id '{}' must match [a-z0-9_]{{1,32}}",
                slot.id
            )));
        }
        if !seen.insert(slot.id.clone()) {
            return Err(DeviceError::new(format!("{source}: device id '{}' is listed twice", slot.id)));
        }
        if slot.label.trim().is_empty() {
            return Err(DeviceError::new(format!("{source}: device '{}' has an empty label", slot.id)));
        }
        if let Some(layout_id) = &slot.default_layout {
            if !layouts.contains_key(layout_id) {
                return Err(DeviceError::new(format!(
                    "{source}: device '{}' has defaultLayout '{layout_id}' which is not a loaded layout",
                    slot.id
                )));
            }
        }
    }
    Ok(DeviceRegistry { slots: file.devices })
}

/// `devices/devices.json` を読む。**ファイルが無ければ空**（今の動きのまま）。
/// 読めない・形が違う・上限超え・未知の盤面は起動を止める（他の JSON と同じ扱い）。
pub fn load_devices<L>(dir: &Path, layouts: &BTreeMap<String, L>) -> Result<DeviceRegistry, DeviceError> {
    let path = dir.join(DEVICES_FILE);
    if !path.exists() {
        return Ok(DeviceRegistry::empty());
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|error| DeviceError::new(format!("{}: {error}", path.display())))?;
    load_devices_str(&path.display().to_string(), &text, layouts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layouts(ids: &[&str]) -> BTreeMap<String, ()> {
        ids.iter().map(|id| (id.to_string(), ())).collect()
    }

    const THREE: &str = r#"{
        "devices": [
            { "id": "ipad", "label": "iPad", "kind": "tablet", "orientation": "landscape", "defaultLayout": "note_story" },
            { "id": "android1", "label": "Android 1", "kind": "phone", "orientation": "portrait" },
            { "id": "android2", "label": "Android 2", "kind": "phone", "orientation": "landscape", "defaultLayout": "game" }
        ]
    }"#;

    #[test]
    fn loads_three_devices_in_file_order() {
        let reg = load_devices_str("t", THREE, &layouts(&["note_story", "game"])).expect("valid");
        assert_eq!(reg.len(), 3);
        assert_eq!(reg.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), ["ipad", "android1", "android2"]);
        assert_eq!(reg.get("ipad").unwrap().kind, DeviceKind::Tablet);
        assert_eq!(reg.get("android1").unwrap().default_layout, None);
        assert!(!reg.contains("android3"));
    }

    #[test]
    fn rejects_more_than_three_devices() {
        let four = r#"{ "devices": [
            { "id": "a", "label": "A", "kind": "phone", "orientation": "portrait" },
            { "id": "b", "label": "B", "kind": "phone", "orientation": "portrait" },
            { "id": "c", "label": "C", "kind": "phone", "orientation": "portrait" },
            { "id": "d", "label": "D", "kind": "phone", "orientation": "portrait" }
        ]}"#;
        let error = load_devices_str("t", four, &layouts(&[])).expect_err("4 devices must be rejected");
        assert_eq!(error.code, LOAD_DEVICE_SCHEMA_INVALID);
    }

    #[test]
    fn rejects_bad_ids_duplicates_empty_labels_and_unknown_layouts() {
        let cases = [
            r#"{ "devices": [ { "id": "iPad", "label": "x", "kind": "tablet", "orientation": "landscape" } ] }"#,
            r#"{ "devices": [ { "id": "../x", "label": "x", "kind": "tablet", "orientation": "landscape" } ] }"#,
            r#"{ "devices": [ { "id": "a", "label": "x", "kind": "phone", "orientation": "portrait" },
                              { "id": "a", "label": "y", "kind": "phone", "orientation": "portrait" } ] }"#,
            r#"{ "devices": [ { "id": "a", "label": "  ", "kind": "phone", "orientation": "portrait" } ] }"#,
            r#"{ "devices": [ { "id": "a", "label": "x", "kind": "phone", "orientation": "portrait", "defaultLayout": "nope" } ] }"#,
            r#"{ "devices": [ { "id": "a", "label": "x", "kind": "watch", "orientation": "portrait" } ] }"#,
            r#"{ "devices": [ { "id": "a", "label": "x", "kind": "phone", "orientation": "portrait", "token": "x" } ] }"#,
        ];
        for text in cases {
            assert!(load_devices_str("t", text, &layouts(&["game"])).is_err(), "must reject: {text}");
        }
    }

    #[test]
    fn missing_file_means_no_devices() {
        let dir = std::env::temp_dir().join(format!("keydeck_devices_missing_{}", std::process::id()));
        let reg = load_devices(&dir, &layouts(&[])).expect("missing file is fine");
        assert!(reg.is_empty());
    }

    #[test]
    fn device_id_rule() {
        for ok in ["ipad", "android_1", "a", &"a".repeat(32)] {
            assert!(device_id_is_safe(ok), "{ok}");
        }
        for bad in ["", "Ipad", "a-b", "a b", "..", "a/b", &"a".repeat(33)] {
            assert!(!device_id_is_safe(bad), "{bad:?}");
        }
    }

    /// 同梱の見本（devices.example.json）が、そのまま devices.json にしても読める形であること。
    #[test]
    fn example_file_is_valid() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let text = std::fs::read_to_string(root.join("devices").join("devices.example.json")).expect("example exists");
        let loaded = crate::startup::load_startup_data(
            &root.join(crate::KEYMAPS_DIR),
            &root.join(crate::DECKS_DIR),
            &root.join(crate::SURFACES_DIR),
            &root.join(crate::LAYOUTS_DIR),
            &root.join(crate::APPS_DIR),
        )
        .expect("real data loads");
        let reg = load_devices_str("example", &text, &loaded.layouts).expect("example must be valid");
        assert_eq!(reg.len(), 3);
    }
}
