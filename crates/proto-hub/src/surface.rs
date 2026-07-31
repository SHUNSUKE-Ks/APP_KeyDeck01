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

/// D28: `binding.t`の許可リスト。コード内固定リストであり、JSON側からは拡張できない。
/// 現状"mouse.move"のみ許可（brief/keydeck_trackball_design_v0.6.md §3.1）。
const ALLOWED_BINDING_TYPES: &[&str] = &["mouse.move"];

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

/// ロード済みの1面ぶんの定義。`binding_t`は許可リスト検証済みの文字列
/// （現状は常に"mouse.move"）、`clamp`は1..=1000に収まることを検証済み。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceDef {
    pub binding_t: String,
    pub clamp: i64,
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
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingWire {
    t: String,
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
    let root: SurfaceFileRoot = serde_json::from_str(text)
        .map_err(|error| SurfaceError::new(LOAD_SURFACE_SCHEMA_INVALID, format!("{source}: {error}")))?;

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

        let def = SurfaceDef {
            binding_t: entry.binding.t,
            clamp,
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
