//! surfaces/*.json のロード＆検証（D28・トラックボール面T11）。schemas/surface.schema.json相当。
//!
//! 設計書: brief/keydeck_trackball_design_v0.6.md §3.1・§4・T11。
//! `surfaces/trackball.json` を読み、`SurfaceRegistry { id -> SurfaceDef { binding, clamp } }` を
//! 構築する。`startup::load_startup_data` から呼ばれ、既存のkeymap/deck検証と同じエラー集約経路
//! （1件でも失敗したら起動拒否・部分適用しない）に乗る。
//!
//! `binding.t` の許可リストは本ファイル内の`ALLOWED_BINDING_TYPES`という固定リストのみ
//! （D28の要件。ここを緩めてはならない）。現状は`"mouse.move"`の1種のみ。

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

// ============================================================================
// D9 エラーコード（本trackball機能でproto-hubが生成する範囲。正はここ1箇所）
// ============================================================================

pub const SURFACE_UNKNOWN_ID: &str = "SURFACE_UNKNOWN_ID";
pub const SURFACE_STATE_RANGE: &str = "SURFACE_STATE_RANGE";
pub const LOAD_SURFACE_SCHEMA_INVALID: &str = "LOAD_SURFACE_SCHEMA_INVALID";
pub const LOAD_SURFACE_BINDING_UNKNOWN: &str = "LOAD_SURFACE_BINDING_UNKNOWN";
/// T18（brief/keydeck_trackball_gestures_v0.7.md §2.3）: `surface.gesture`で未知のgestureId
/// が来た場合。
pub const SURFACE_GESTURE_UNKNOWN_ID: &str = "SURFACE_GESTURE_UNKNOWN_ID";
/// T18: `hold1`等ButtonHold系ジェスチャーで`edge`が省略された場合（発火しない）。
pub const SURFACE_GESTURE_EDGE_REQUIRED: &str = "SURFACE_GESTURE_EDGE_REQUIRED";
/// T17（§2.2）: `gestures`マップの値（`t`/`button`/`vk`不正、gestureId重複）が不正な場合。
pub const LOAD_SURFACE_GESTURE_INVALID: &str = "LOAD_SURFACE_GESTURE_INVALID";

/// D28: `binding.t`の許可リスト。コード内固定リストであり、JSON側からは拡張できない。
/// T15/T17（brief/keydeck_trackball_gestures_v0.7.md §3）: continuousスクロール用に
/// "mouse.scroll"を追加（"mouse.move"のみだった状態から拡張）。
const ALLOWED_BINDING_TYPES: &[&str] = &["mouse.move", "mouse.scroll"];

/// クランプの許容範囲（省略時200。1〜1000の範囲外は拒否。§3.1）。
const CLAMP_DEFAULT: i64 = 200;
const CLAMP_MIN: i64 = 1;
const CLAMP_MAX: i64 = 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceError {
    pub code: &'static str,
    pub cause: String,
}

impl SurfaceError {
    fn new(code: &'static str, cause: impl Into<String>) -> Self {
        Self {
            code,
            cause: cause.into(),
        }
    }
}

impl std::fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.cause)
    }
}

impl std::error::Error for SurfaceError {}

/// T17（brief/keydeck_trackball_gestures_v0.7.md §2.2）: discreteジェスチャーで使う
/// マウスボタン識別。`proto_keymap::MouseButtonKind`を再利用しない（surface.rsの宣言は
/// 「どのボタンか」だけを持ち、down/upはHub側が`edge`から組み立てるため独立した小さい型
/// で十分。§2.2のコメントどおり）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickButton {
    Left,
    Right,
}

/// T17: `surfaces/trackball.json`の`gestures`マップの値。`proto_keymap::Action`は
/// 再利用しない（§2.2参照）。`ButtonHold`のみedge必須で、Hub側（ws.rs）が
/// `edge`から`proto_keymap::Action::MouseButton{button, down}`を組み立てる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GestureAction {
    Click { button: ClickButton },
    DoubleClick { button: ClickButton },
    /// edge必須。Down→press、Up→release（ws.rsの`handle_surface_gesture`が組み立てる）。
    ButtonHold { button: ClickButton },
    /// `is_known_vk()`で既存vk辞書と同じ検証を通す。
    Key { vk: String },
    /// 複数キー同時押し（Ctrl+V など）。キーマップの`chord`と同じ形。
    /// 「貼り付け」「ブラウザの戻る」のように、1キーでは表せない操作に要る。
    Chord { keys: Vec<String> },
}

/// ロード済みの1面ぶんの定義。`binding_t`は許可リスト検証済みの文字列
/// （"mouse.move"または"mouse.scroll"）、`clamp`は1..=1000に収まることを検証済み。
/// `gestures`はT17で追加（省略時は空マップ＝ジェスチャー無しの面として従来どおり動く）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceDef {
    pub binding_t: String,
    pub clamp: i64,
    pub gestures: BTreeMap<String, GestureAction>,
}

/// `surfaceId -> SurfaceDef`。クライアントはこのレジストリに載っているidしか名乗れない
/// （T12でHubがWSメッセージのsurfaceIdをここで引く）。
#[derive(Debug, Clone, Default)]
pub struct SurfaceRegistry {
    surfaces: BTreeMap<String, SurfaceDef>,
}

impl SurfaceRegistry {
    /// T11-4: `surfaces/`ディレクトリまたは`trackball.json`が存在しない場合に使う
    /// 空レジストリ（既存ユーザーの起動を壊さないため、エラーではなく正常起動扱い）。
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get(&self, id: &str) -> Option<&SurfaceDef> {
        self.surfaces.get(id)
    }

    pub fn len(&self) -> usize {
        self.surfaces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }
}

// ============================================================================
// ディスク上フォーマット（§3.1）
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SurfaceFileRoot {
    surfaces: Vec<SurfaceFileEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SurfaceFileEntry {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    binding: BindingWire,
    #[serde(default)]
    clamp: Option<i64>,
    /// T17（§2.2）: 任意フィールド。省略時は空マップ＝ジェスチャー無しの面。
    #[serde(default)]
    gestures: GesturesWire,
}

/// T17: `gestures`フィールド用のラッパー。標準の`BTreeMap<K,V>`のDeserializeは
/// JSON側に同名キーが複数回現れても後勝ちで黙って上書きする（duplicate検出不可）ため、
/// `visit_map`を自前実装し、挿入時に既存キーへ衝突したらエラーにする
/// （§2.2「同一面内でgestureId重複 → LOAD_SURFACE_GESTURE_INVALID」を実際に検出するため）。
#[derive(Debug, Default)]
struct GesturesWire(BTreeMap<String, GestureWire>);

impl<'de> Deserialize<'de> for GesturesWire {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct GesturesVisitor;

        impl<'de> serde::de::Visitor<'de> for GesturesVisitor {
            type Value = GesturesWire;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "a map of gestureId -> gesture definition")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut result: BTreeMap<String, GestureWire> = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, GestureWire>()? {
                    if result.insert(key.clone(), value).is_some() {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate gestureId '{key}'"
                        )));
                    }
                }
                Ok(GesturesWire(result))
            }
        }

        deserializer.deserialize_map(GesturesVisitor)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingWire {
    t: String,
}

/// T17（§2.2）: `gestures`マップの1エントリのディスク上フォーマット。
/// `button`は"mouse.click"/"mouse.dblclick"/"mouse.button.hold"のときのみ必須、
/// `vk`は"key"のときのみ必須（どちらも`Option`で受けてロード時検証する）。
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GestureWire {
    t: String,
    #[serde(default)]
    button: Option<String>,
    #[serde(default)]
    vk: Option<String>,
    #[serde(default)]
    keys: Option<Vec<String>>,
}

// ============================================================================
// ロード＆検証
// ============================================================================

/// `surfaces_dir`直下の`trackball.json`を読む（T11-1）。ディレクトリ自体、または
/// `trackball.json`が存在しない場合は空レジストリで正常起動する（T11-4）。
pub fn load_surface_registry(surfaces_dir: &Path) -> Result<SurfaceRegistry, SurfaceError> {
    let path = surfaces_dir.join("trackball.json");
    if !path.is_file() {
        return Ok(SurfaceRegistry::empty());
    }

    let text = std::fs::read_to_string(&path).map_err(|error| {
        SurfaceError::new(
            LOAD_SURFACE_SCHEMA_INVALID,
            format!("{}: failed to read file: {error}", path.display()),
        )
    })?;
    load_surface_registry_str(&path.display().to_string(), &text)
}

/// メモリ上の文字列からレジストリを構築する（単体テスト用。ディスクI/O無し）。
pub fn load_surface_registry_str(source: &str, text: &str) -> Result<SurfaceRegistry, SurfaceError> {
    let root: SurfaceFileRoot = serde_json::from_str(text).map_err(|error| {
        // T17: GesturesWireのvisit_mapが投げた「gestureId重複」だけは、他のJSON構文/形状
        // エラーとは別コード（LOAD_SURFACE_GESTURE_INVALID）で報告する（§2.2の要件どおり）。
        let message = error.to_string();
        if message.contains("duplicate gestureId") {
            SurfaceError::new(LOAD_SURFACE_GESTURE_INVALID, format!("{source}: {message}"))
        } else {
            SurfaceError::new(LOAD_SURFACE_SCHEMA_INVALID, format!("{source}: {error}"))
        }
    })?;

    let mut surfaces: BTreeMap<String, SurfaceDef> = BTreeMap::new();
    for entry in root.surfaces {
        // §3.1: type は現状 "trackball" のみ許可。
        if entry.kind != "trackball" {
            return Err(SurfaceError::new(
                LOAD_SURFACE_SCHEMA_INVALID,
                format!(
                    "{source}: surface '{}': unknown type '{}' (only \"trackball\" is allowed)",
                    entry.id, entry.kind
                ),
            ));
        }

        // T11-3/D28: binding.t はコード内固定の許可リストのみ。
        if !ALLOWED_BINDING_TYPES.contains(&entry.binding.t.as_str()) {
            return Err(SurfaceError::new(
                LOAD_SURFACE_BINDING_UNKNOWN,
                format!(
                    "{source}: surface '{}': unknown binding.t '{}'",
                    entry.id, entry.binding.t
                ),
            ));
        }

        // §3.1: clamp は省略時200、1〜1000の範囲外は拒否。
        let clamp = entry.clamp.unwrap_or(CLAMP_DEFAULT);
        if !(CLAMP_MIN..=CLAMP_MAX).contains(&clamp) {
            return Err(SurfaceError::new(
                LOAD_SURFACE_SCHEMA_INVALID,
                format!(
                    "{source}: surface '{}': clamp {clamp} out of range {CLAMP_MIN}..={CLAMP_MAX}",
                    entry.id
                ),
            ));
        }

        // T17（§2.2）: gesturesマップのロード＆検証。gestureId重複はGesturesWireの
        // カスタムDeserialize（visit_map）が既にJSONパース時点で検出済みのため、
        // ここに到達する時点でentry.gestures.0にキー重複は無い。
        let mut gestures: BTreeMap<String, GestureAction> = BTreeMap::new();
        for (gesture_id, wire) in entry.gestures.0 {
            let action = parse_gesture_wire(source, &entry.id, &gesture_id, &wire)?;
            gestures.insert(gesture_id, action);
        }

        let def = SurfaceDef {
            binding_t: entry.binding.t,
            clamp,
            gestures,
        };
        if surfaces.insert(entry.id.clone(), def).is_some() {
            return Err(SurfaceError::new(
                LOAD_SURFACE_SCHEMA_INVALID,
                format!("{source}: duplicate surface id '{}'", entry.id),
            ));
        }
    }

    Ok(SurfaceRegistry { surfaces })
}

/// T17（§2.2）ロード時検証:
/// - `t`が`"mouse.click"|"mouse.dblclick"|"mouse.button.hold"|"key"`以外 → LOAD_SURFACE_GESTURE_INVALID
/// - `button`が`"left"|"right"`以外 → LOAD_SURFACE_GESTURE_INVALID
/// - `t:"key"`の`vk`が`proto_keymap::is_known_vk()`を通らない → LOAD_SURFACE_GESTURE_INVALID
///   （`LOAD_VK_UNKNOWN`は再利用しない。surface.rs系のエラーは`LOAD_SURFACE_*`で揃える）
fn parse_gesture_wire(
    source: &str,
    surface_id: &str,
    gesture_id: &str,
    wire: &GestureWire,
) -> Result<GestureAction, SurfaceError> {
    fn parse_button(
        source: &str,
        surface_id: &str,
        gesture_id: &str,
        wire: &GestureWire,
    ) -> Result<ClickButton, SurfaceError> {
        match wire.button.as_deref() {
            Some("left") => Ok(ClickButton::Left),
            Some("right") => Ok(ClickButton::Right),
            other => Err(SurfaceError::new(
                LOAD_SURFACE_GESTURE_INVALID,
                format!(
                    "{source}: surface '{surface_id}': gesture '{gesture_id}': invalid button {other:?} (must be \"left\" or \"right\")"
                ),
            )),
        }
    }

    match wire.t.as_str() {
        "mouse.click" => Ok(GestureAction::Click {
            button: parse_button(source, surface_id, gesture_id, wire)?,
        }),
        "mouse.dblclick" => Ok(GestureAction::DoubleClick {
            button: parse_button(source, surface_id, gesture_id, wire)?,
        }),
        "mouse.button.hold" => Ok(GestureAction::ButtonHold {
            button: parse_button(source, surface_id, gesture_id, wire)?,
        }),
        "key" => {
            let Some(vk) = wire.vk.as_deref() else {
                return Err(SurfaceError::new(
                    LOAD_SURFACE_GESTURE_INVALID,
                    format!(
                        "{source}: surface '{surface_id}': gesture '{gesture_id}': t=\"key\" requires 'vk'"
                    ),
                ));
            };
            if !proto_keymap::is_known_vk(vk) {
                return Err(SurfaceError::new(
                    LOAD_SURFACE_GESTURE_INVALID,
                    format!(
                        "{source}: surface '{surface_id}': gesture '{gesture_id}': unknown vk '{vk}'"
                    ),
                ));
            }
            Ok(GestureAction::Key { vk: vk.to_string() })
        }
        "chord" => {
            let keys = wire.keys.clone().unwrap_or_default();
            if keys.is_empty() {
                return Err(SurfaceError::new(
                    LOAD_SURFACE_GESTURE_INVALID,
                    format!(
                        "{source}: surface '{surface_id}': gesture '{gesture_id}': t=\"chord\" requires a non-empty 'keys'"
                    ),
                ));
            }
            for vk in &keys {
                if !proto_keymap::is_known_vk(vk) {
                    return Err(SurfaceError::new(
                        LOAD_SURFACE_GESTURE_INVALID,
                        format!(
                            "{source}: surface '{surface_id}': gesture '{gesture_id}': unknown vk '{vk}' in chord"
                        ),
                    ));
                }
            }
            Ok(GestureAction::Chord { keys })
        }
        other => Err(SurfaceError::new(
            LOAD_SURFACE_GESTURE_INVALID,
            format!(
                "{source}: surface '{surface_id}': gesture '{gesture_id}': unknown t '{other}'"
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // G-11a: 正常なJSONでレジストリが構築される。
    #[test]
    fn g11a_valid_json_builds_registry() {
        let text = r#"{
            "surfaces": [
                { "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" }, "clamp": 200 }
            ]
        }"#;
        let registry = load_surface_registry_str("test", text).expect("valid surfaces.json must load");
        assert_eq!(registry.len(), 1);
        let def = registry.get("tb01").expect("tb01 must be registered");
        assert_eq!(def.binding_t, "mouse.move");
        assert_eq!(def.clamp, 200);
    }

    // clampを省略した場合は既定値200になる。
    #[test]
    fn clamp_defaults_to_200_when_omitted() {
        let text = r#"{
            "surfaces": [
                { "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" } }
            ]
        }"#;
        let registry = load_surface_registry_str("test", text).unwrap();
        assert_eq!(registry.get("tb01").unwrap().clamp, 200);
    }

    // G-11b: binding.tが未知の値だとLOAD_SURFACE_BINDING_UNKNOWNで起動拒否。
    #[test]
    fn g11b_unknown_binding_type_is_rejected() {
        let text = r#"{
            "surfaces": [
                { "id": "tb01", "type": "trackball", "binding": { "t": "gamepad.axis" }, "clamp": 200 }
            ]
        }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_BINDING_UNKNOWN);
        assert!(error.cause.contains("gamepad.axis"));
    }

    // G-11c: 形が壊れている（必須フィールド欠落）とLOAD_SURFACE_SCHEMA_INVALIDで起動拒否。
    #[test]
    fn g11c_malformed_shape_is_rejected_as_schema_invalid() {
        let text = r#"{ "surfaces": [ { "id": "tb01" } ] }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_SCHEMA_INVALID);
    }

    // G-11c派生: JSON構文自体が壊れている場合もLOAD_SURFACE_SCHEMA_INVALID
    // （surfacesのエラーコード表にはJSON構文専用コードが無く、構文/形状とも同一コードに集約する）。
    #[test]
    fn invalid_json_syntax_is_rejected_as_schema_invalid() {
        let error = load_surface_registry_str("test", "{ not json").unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_SCHEMA_INVALID);
    }

    // typeが"trackball"以外だとLOAD_SURFACE_SCHEMA_INVALID。
    #[test]
    fn unknown_surface_type_is_rejected_as_schema_invalid() {
        let text = r#"{
            "surfaces": [
                { "id": "tb01", "type": "gamepad", "binding": { "t": "mouse.move" }, "clamp": 200 }
            ]
        }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_SCHEMA_INVALID);
    }

    // clampが範囲外(1..=1000)だとLOAD_SURFACE_SCHEMA_INVALID。
    #[test]
    fn clamp_out_of_range_is_rejected() {
        let text = r#"{
            "surfaces": [
                { "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" }, "clamp": 5000 }
            ]
        }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_SCHEMA_INVALID);
    }

    // 重複id → LOAD_SURFACE_SCHEMA_INVALID。
    #[test]
    fn duplicate_surface_id_is_rejected() {
        let text = r#"{
            "surfaces": [
                { "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" } },
                { "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" } }
            ]
        }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_SCHEMA_INVALID);
    }

    // ── T17: gesturesマップのロード＆検証（G-17a/G-17b） ──────────────────

    // gesturesを持つ面が正常にロードされ、各GestureActionが正しく組み立てられる。
    #[test]
    fn g17a_valid_gestures_load_successfully() {
        let text = r#"{
            "surfaces": [
                {
                    "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" }, "clamp": 200,
                    "gestures": {
                        "tap1":  { "t": "mouse.click",     "button": "left" },
                        "dtap1": { "t": "mouse.dblclick",  "button": "left" },
                        "tap2":  { "t": "mouse.click",     "button": "right" },
                        "tap3":  { "t": "key",             "vk": "ESC" },
                        "hold1": { "t": "mouse.button.hold", "button": "left" }
                    }
                }
            ]
        }"#;
        let registry = load_surface_registry_str("test", text).expect("valid gestures must load");
        let def = registry.get("tb01").expect("tb01 must be registered");
        assert_eq!(def.gestures.len(), 5);
        assert_eq!(def.gestures.get("tap1"), Some(&GestureAction::Click { button: ClickButton::Left }));
        assert_eq!(
            def.gestures.get("dtap1"),
            Some(&GestureAction::DoubleClick { button: ClickButton::Left })
        );
        assert_eq!(def.gestures.get("tap2"), Some(&GestureAction::Click { button: ClickButton::Right }));
        assert_eq!(def.gestures.get("tap3"), Some(&GestureAction::Key { vk: "ESC".to_string() }));
        assert_eq!(
            def.gestures.get("hold1"),
            Some(&GestureAction::ButtonHold { button: ClickButton::Left })
        );
    }

    // gesturesを省略した面は空マップになる（従来どおり動く）。
    #[test]
    fn gestures_field_defaults_to_empty_map_when_omitted() {
        let text = r#"{
            "surfaces": [
                { "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" } }
            ]
        }"#;
        let registry = load_surface_registry_str("test", text).unwrap();
        assert!(registry.get("tb01").unwrap().gestures.is_empty());
    }

    // G-17a: 未知のt → LOAD_SURFACE_GESTURE_INVALID。
    #[test]
    fn g17a_unknown_gesture_t_is_rejected() {
        let text = r#"{
            "surfaces": [
                {
                    "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" },
                    "gestures": { "tap1": { "t": "mouse.triple", "button": "left" } }
                }
            ]
        }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_GESTURE_INVALID);
    }

    // G-17a: 未知のbutton → LOAD_SURFACE_GESTURE_INVALID。
    #[test]
    fn g17a_unknown_gesture_button_is_rejected() {
        let text = r#"{
            "surfaces": [
                {
                    "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" },
                    "gestures": { "tap1": { "t": "mouse.click", "button": "middle" } }
                }
            ]
        }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_GESTURE_INVALID);
    }

    // G-17a: mouse.clickでbutton省略 → LOAD_SURFACE_GESTURE_INVALID。
    #[test]
    fn g17a_missing_gesture_button_is_rejected() {
        let text = r#"{
            "surfaces": [
                {
                    "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" },
                    "gestures": { "tap1": { "t": "mouse.click" } }
                }
            ]
        }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_GESTURE_INVALID);
    }

    // G-17a: 未知のvk → LOAD_SURFACE_GESTURE_INVALID（LOAD_VK_UNKNOWNは再利用しない）。
    #[test]
    fn g17a_unknown_gesture_vk_is_rejected() {
        let text = r#"{
            "surfaces": [
                {
                    "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" },
                    "gestures": { "tap3": { "t": "key", "vk": "NOT_A_KEY" } }
                }
            ]
        }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_GESTURE_INVALID);
    }

    // G-17a: t="key"でvk省略 → LOAD_SURFACE_GESTURE_INVALID。
    #[test]
    fn g17a_missing_gesture_vk_is_rejected() {
        let text = r#"{
            "surfaces": [
                {
                    "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" },
                    "gestures": { "tap3": { "t": "key" } }
                }
            ]
        }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_GESTURE_INVALID);
    }

    // G-17a: 同一面内でgestureId重複 → LOAD_SURFACE_GESTURE_INVALID。
    #[test]
    fn g17a_duplicate_gesture_id_is_rejected() {
        let text = r#"{
            "surfaces": [
                {
                    "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" },
                    "gestures": {
                        "tap1": { "t": "mouse.click", "button": "left" },
                        "tap1": { "t": "mouse.click", "button": "right" }
                    }
                }
            ]
        }"#;
        let error = load_surface_registry_str("test", text).unwrap_err();
        assert_eq!(error.code, LOAD_SURFACE_GESTURE_INVALID);
        assert!(error.cause.contains("tap1"));
    }

    // G-17b: mouse.scrollがALLOWED_BINDING_TYPESに入り正常ロードされる。
    #[test]
    fn g17b_mouse_scroll_binding_type_is_allowed() {
        let text = r#"{
            "surfaces": [
                { "id": "tb01-scroll", "type": "trackball", "binding": { "t": "mouse.scroll" }, "clamp": 100 }
            ]
        }"#;
        let registry = load_surface_registry_str("test", text).expect("mouse.scroll binding must load");
        let def = registry.get("tb01-scroll").expect("tb01-scroll must be registered");
        assert_eq!(def.binding_t, "mouse.scroll");
        assert_eq!(def.clamp, 100);
    }

    // G-11d: ファイル不在でも起動は成功し、空レジストリになる（既存機能は無影響）。
    #[test]
    fn g11d_missing_file_yields_empty_registry() {
        let dir = std::env::temp_dir().join(format!(
            "keydeck_surface_test_missing_{}",
            std::process::id()
        ));
        // 意図的に作成しない（存在しないディレクトリを渡すケースも兼ねる）。
        let registry = load_surface_registry(&dir).expect("missing surfaces dir must not be an error");
        assert!(registry.is_empty());
    }

    // 実ファイルロード確認: surfaces/trackball.json（新規作成分）が正しく読み込めること。
    #[test]
    fn real_surfaces_trackball_json_loads_successfully() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../surfaces");
        let registry = load_surface_registry(std::path::Path::new(path))
            .expect("surfaces/trackball.json must load");
        assert!(!registry.is_empty(), "surfaces/trackball.json must define at least tb01");
        let def = registry.get("tb01").expect("tb01 must be registered");
        assert_eq!(def.binding_t, "mouse.move");
    }
}
