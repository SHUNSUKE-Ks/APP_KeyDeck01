//! 起動時ロード／`/api/reload`共通のディレクトリスキャン＋検証（設計書v0.5 B1/B2）。
//!
//! B1: `keymaps/keymap_*.json` を全ロードする（固定3ファイルのハードコードを廃止）。
//! 新フォーマットはこのディレクトリへファイルを置くだけで発見される。
//! B2: `/api/reload`（ws.rs）は本モジュールの`load_startup_data`をmain.rsの起動処理と
//! 全く同じ経路で呼び出し、検証が1件でも失敗すれば`Err`を返す＝呼び出し側は現行構成を
//! 一切変更せずに済む（「失敗時は現行構成維持」を関数境界で保証する）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use proto_keymap::{Action, Keymap};

use crate::deck::DeckSetlist;
use crate::state::{canonical_command_id, DEFAULT_DECK_ID, IPAD_KEYMAP_ID};

#[derive(Debug)]
pub struct StartupData {
    pub keymaps: BTreeMap<String, Keymap>,
    /// P-005 段階A: Deckは複数ロードする（`decks/deck_*.json` をスキャン）。
    /// キーは`deckId`。`DEFAULT_DECK_ID`は必ず含まれる（ロード時に検証）。
    pub decks: BTreeMap<String, DeckSetlist>,
    pub command_registry: hub_core::CommandRegistry,
    /// T11（D28）: `surfaces/trackball.json`から構築したレジストリ。
    pub surfaces: crate::surface::SurfaceRegistry,
    /// P-005 段階B: `layouts/layout_*.json`。キーは`layoutId`。0件でも起動する
    /// （既存の面はレイアウトを使わないため）。
    pub layouts: BTreeMap<String, crate::layout::Layout>,
    /// 起動してよいアプリ（`apps/apps.json`）。0件でも起動する。
    pub apps: crate::app_launch::AppRegistry,
}

/// `dir`直下（サブディレクトリは対象外＝`layers/`はここに含まれない）の
/// `keymap_*.json`ファイルパスを名前順（決定的）に返す。読めないディレクトリは空扱い
/// （呼び出し側の`load_startup_data`が「1件もロードできなかった」として起動時と同じ
/// エラー経路に載せる）。
pub fn discover_keymap_paths(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with("keymap_") && name.ends_with(".json") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    paths
}

/// `dir`直下の `<prefix>*.json` を名前順（決定的）に返す。読めないディレクトリは空扱い。
/// P-005 段階A/BでDeckとレイアウトのスキャンに共用する。
pub fn discover_prefixed_json(dir: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with(prefix) && name.ends_with(".json") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    paths
}

/// P-005 段階A: Deck面・コピペリスト等を同時に出せるよう、Deckも`keymaps/`と同じ
/// ディレクトリスキャン方式にする（`decks/deck_*.json`）。
pub fn discover_deck_paths(dir: &Path) -> Vec<PathBuf> {
    discover_prefixed_json(dir, "deck_")
}

/// 起動時（main.rs）／再読込時（ws.rsの`/api/reload`）で共有する検証手順。
/// 順序: ①ディレクトリスキャンで発見した全keymapファイルのロード ②ipad面固定keymapId
/// の存在確認 ③deckのロード ④deck内`keymap.switch`参照先の存在確認 ⑤surfaces/trackball.json
/// のロード（T11。無ければ空レジストリ）。
/// 1件でもエラーがあれば集約して`Err(Vec<String>)`を返す（部分適用はしない）。
pub fn load_startup_data(
    keymaps_dir: &Path,
    decks_dir: &Path,
    surfaces_dir: &Path,
    layouts_dir: &Path,
    apps_dir: &Path,
) -> Result<StartupData, Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    let mut keymaps: BTreeMap<String, Keymap> = BTreeMap::new();

    let paths = discover_keymap_paths(keymaps_dir);
    if paths.is_empty() {
        errors.push(format!(
            "[{}] no keymap_*.json files found under {}",
            proto_keymap::LOAD_SCHEMA_INVALID,
            keymaps_dir.display()
        ));
    }
    for path in &paths {
        match proto_keymap::load_keymap_from_path(path) {
            Ok(keymap) => {
                keymaps.insert(keymap.keymap_id.clone(), keymap);
            }
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }

    // T8由来: ipad面はIPAD_KEYMAP_IDが常にロード済みである前提で動く。
    if errors.is_empty() && !keymaps.contains_key(IPAD_KEYMAP_ID) {
        errors.push(format!(
            "[{}] ipad surface requires keymapId '{IPAD_KEYMAP_ID}' to be loaded",
            proto_keymap::LOAD_SCHEMA_INVALID
        ));
    }

    // P-005 段階A: `decks/deck_*.json` を全ロードする（旧: deck_default.json 1枚固定）。
    let mut decks: BTreeMap<String, DeckSetlist> = BTreeMap::new();
    let deck_paths = discover_deck_paths(decks_dir);
    if deck_paths.is_empty() {
        errors.push(format!(
            "[{}] no deck_*.json files found under {}",
            proto_keymap::LOAD_SCHEMA_INVALID,
            decks_dir.display()
        ));
    }
    for path in &deck_paths {
        match crate::deck::load_deck_from_path(path) {
            Ok(deck) => {
                if let Some(existing) = decks.insert(deck.deck_id.clone(), deck) {
                    errors.push(format!(
                        "[{}] duplicate deckId '{}' found while scanning {}",
                        proto_keymap::LOAD_SCHEMA_INVALID,
                        existing.deck_id,
                        decks_dir.display()
                    ));
                }
            }
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }

    // 既存面（/deck・/panel）はdeckId未指定時にDEFAULT_DECK_IDを開くため、必ず要る。
    if errors.is_empty() && !decks.contains_key(DEFAULT_DECK_ID) {
        errors.push(format!(
            "[{}] deckId '{DEFAULT_DECK_ID}' is required but was not found under {}",
            proto_keymap::LOAD_SCHEMA_INVALID,
            decks_dir.display()
        ));
    }

    if errors.is_empty() {
        for action in all_actions(&keymaps, &decks) {
            if let Action::KeymapSwitch { id } = action {
                if !keymaps.contains_key(id) {
                    errors.push(format!(
                        "[{}] keymap.switch references unknown keymapId '{id}'",
                        proto_keymap::LOAD_SCHEMA_INVALID
                    ));
                }
            }
        }
    }

    // T11（D28）: surfaces/trackball.json のロード＆検証。同じエラー集約経路に乗せる
    // （1件でも失敗すれば他が正常でも起動拒否。無ければ空レジストリで正常起動＝T11-4）。
    let surfaces = match crate::surface::load_surface_registry(surfaces_dir) {
        Ok(registry) => Some(registry),
        Err(error) => {
            errors.push(error.to_string());
            None
        }
    };

    // P-005 段階B: `layouts/layout_*.json`。ディレクトリが無ければ0件で正常（既存面は使わない）。
    let mut layouts: BTreeMap<String, crate::layout::Layout> = BTreeMap::new();
    for path in discover_prefixed_json(layouts_dir, "layout_") {
        match crate::layout::load_layout_from_path(&path) {
            Ok(layout) => {
                if let Some(existing) = layouts.insert(layout.layout_id.clone(), layout) {
                    errors.push(format!(
                        "[{}] duplicate layoutId '{}' found while scanning {}",
                        crate::layout::LOAD_LAYOUT_INVALID,
                        existing.layout_id,
                        layouts_dir.display()
                    ));
                }
            }
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }

    // 起動してよいアプリの許可リスト。無ければ0件で正常（この機能を使わない人を止めない）。
    let apps = match crate::app_launch::load_app_registry(apps_dir) {
        Ok(registry) => Some(registry),
        Err(error) => {
            errors.push(error.to_string());
            None
        }
    };

    // 参照先（keymapId / deckId / surfaceId）の実在確認。全部ロードし終えた今しかできない。
    // ここで止めないと、実機で開いた瞬間に空の区画が出て原因が分からない形で壊れる。
    if errors.is_empty() {
        for layout in layouts.values() {
            for section in &layout.sections {
                let reference = &section.component.reference;
                let found = match section.component.kind {
                    crate::layout::ComponentKind::Keyboard => keymaps.contains_key(reference),
                    crate::layout::ComponentKind::Deck => decks.contains_key(reference),
                    crate::layout::ComponentKind::Trackball => surfaces
                        .as_ref()
                        .is_some_and(|registry| registry.get(reference).is_some()),
                    // ダイヤルは「jogを持つキーマップ」だけ。普通のキーボードを
                    // ダイヤルとして置くと、回しても押すキーが無く黙って無反応になる。
                    crate::layout::ComponentKind::Jog => {
                        keymaps.get(reference).is_some_and(|k| k.jog.is_some())
                    }
                };
                if !found {
                    errors.push(format!(
                        "[{}] layout '{}' section '{}': {:?} component references unknown id '{reference}'",
                        crate::layout::LOAD_LAYOUT_REF_UNKNOWN,
                        layout.layout_id,
                        section.id,
                        section.component.kind
                    ));
                }
            }
        }
    }

    // `app.launch` の参照先。キーマップ単体では確認できないので、ここでまとめて見る。
    // 押した時に初めて「そんなアプリは無い」と分かるのでは、原因が遠すぎる。
    if errors.is_empty() {
        if let Some(registry) = apps.as_ref() {
            for (source, action) in all_actions_with_source(&keymaps, &decks) {
                if let Action::AppLaunch { id, .. } = action {
                    if !registry.contains(id) {
                        errors.push(format!(
                            "[{}] {source}: app.launch references unknown app id '{id}' (add it to apps/apps.json)",
                            crate::app_launch::APP_LAUNCH_UNKNOWN
                        ));
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let surfaces = surfaces.expect("surfaces load succeeded because errors is empty");
    let apps = apps.expect("apps load succeeded because errors is empty");
    let command_ids: Vec<String> = all_actions(&keymaps, &decks)
        .filter_map(canonical_command_id)
        .collect();
    let command_registry = hub_core::CommandRegistry::new(command_ids);

    Ok(StartupData {
        keymaps,
        decks,
        layouts,
        command_registry,
        surfaces,
        apps,
    })
}

/// `all_actions` と同じものを、**どのファイル由来か**を付けて返す。
/// 「どのアプリが無い」だけでは、どのキーを直せばよいか分からないため。
fn all_actions_with_source<'a>(
    keymaps: &'a BTreeMap<String, Keymap>,
    decks: &'a BTreeMap<String, DeckSetlist>,
) -> impl Iterator<Item = (String, &'a Action)> {
    keymaps
        .iter()
        .flat_map(|(id, keymap)| {
            keymap.layers.iter().flat_map(move |layer| {
                layer
                    .keys
                    .iter()
                    .map(move |(key_id, key_def)| {
                        (
                            format!("keymap '{id}' layer {} key '{key_id}'", layer.id),
                            &key_def.action,
                        )
                    })
            })
        })
        .chain(decks.iter().flat_map(|(id, deck)| {
            deck.actions().map(move |action| (format!("deck '{id}'"), action))
        }))
}

fn all_actions<'a>(
    keymaps: &'a BTreeMap<String, Keymap>,
    decks: &'a BTreeMap<String, DeckSetlist>,
) -> impl Iterator<Item = &'a Action> {
    keymaps
        .values()
        .flat_map(|keymap| keymap.layers.iter())
        .flat_map(|layer| layer.keys.values())
        .map(|key_def| &key_def.action)
        .chain(decks.values().flat_map(|deck| deck.actions()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let n = COUNTER.fetch_add(1, Ordering::SeqCst);
            let dir = std::env::temp_dir().join(format!(
                "keydeck_startup_test_{tag}_{}_{n}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).expect("create temp dir");
            std::fs::create_dir_all(dir.join("keymaps/layers")).expect("create layers dir");
            std::fs::create_dir_all(dir.join("decks")).expect("create decks dir");
            Self(dir)
        }

        fn keymaps_dir(&self) -> PathBuf {
            self.0.join("keymaps")
        }

        /// P-005 段階A: Deckもディレクトリスキャンになったのでディレクトリを渡す。
        fn decks_dir(&self) -> PathBuf {
            self.0.join("decks")
        }

        /// P-005 段階B: レイアウトは0件でも起動するので、既定では作らない。
        fn layouts_dir(&self) -> PathBuf {
            self.0.join("layouts")
        }

        /// 意図的に作成しない。apps.json が無くても起動することを一緒に確認する。
        fn apps_dir(&self) -> PathBuf {
            self.0.join("apps")
        }

        /// T11: 意図的に作成しない（未作成のまま渡すことで空レジストリ経路も一緒に確認する）。
        fn surfaces_dir(&self) -> PathBuf {
            self.0.join("surfaces")
        }

        fn write(&self, relative: &str, contents: &str) {
            let path = self.0.join(relative);
            std::fs::write(&path, contents).expect("write fixture file");
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn write_minimal_single_keymap(dir: &TempDir, keymap_id: &str) {
        dir.write(
            &format!("keymaps/layers/{keymap_id}_layer0.json"),
            r#"{ "layer": 0, "keys": { "K101": { "label": "1", "action": { "t": "key", "vk": "1" } } } }"#,
        );
        dir.write(
            &format!("keymaps/keymap_{keymap_id}.json"),
            &format!(
                r#"{{
                    "keymapId": "{keymap_id}",
                    "kind": "single",
                    "board": {{ "cols": 13, "keys": [ {{ "id": "K101", "row": 1, "col": 1 }} ] }},
                    "layerFiles": ["layers/{keymap_id}_layer0.json"]
                }}"#
            ),
        );
    }

    fn write_empty_deck(dir: &TempDir) {
        dir.write(
            "decks/deck_default.json",
            r#"{ "deckId": "default", "grid": { "cols": 1, "rows": 1 }, "pages": [] }"#,
        );
    }

    // B1: keymap_*.jsonが複数（未知の新フォーマット含む）あっても、固定リストなしで
    // 全部発見されロードされることを確認する（=ハードコード3件を廃止したことの証明）。
    #[test]
    fn discover_and_load_picks_up_arbitrary_new_keymap_files_without_hardcoding() {
        let dir = TempDir::new("discover_all");
        write_minimal_single_keymap(&dir, "ipad01_vol12");
        write_minimal_single_keymap(&dir, "brand_new_format_added_by_dropping_a_file");
        write_empty_deck(&dir);

        let data = load_startup_data(&dir.keymaps_dir(), &dir.decks_dir(), &dir.surfaces_dir(), &dir.layouts_dir(), &dir.apps_dir())
            .expect("both keymaps + empty deck must load");
        assert_eq!(data.keymaps.len(), 2);
        assert!(data.keymaps.contains_key("ipad01_vol12"));
        assert!(data.keymaps.contains_key("brand_new_format_added_by_dropping_a_file"));
    }

    // ディレクトリのサブフォルダ(layers/)は`keymap_*.json`のスキャン対象に含まれない。
    #[test]
    fn discover_keymap_paths_ignores_layers_subdirectory() {
        let dir = TempDir::new("ignore_subdir");
        write_minimal_single_keymap(&dir, "ipad01_vol12");
        let paths = discover_keymap_paths(&dir.keymaps_dir());
        assert_eq!(paths.len(), 1);
        assert!(paths[0].to_string_lossy().ends_with("keymap_ipad01_vol12.json"));
    }

    // ipad01_vol12が1件もロードされない構成は起動/reload双方で拒否される。
    #[test]
    fn missing_ipad_keymap_id_is_rejected() {
        let dir = TempDir::new("missing_ipad");
        write_minimal_single_keymap(&dir, "some_other_format");
        write_empty_deck(&dir);

        let errors = load_startup_data(&dir.keymaps_dir(), &dir.decks_dir(), &dir.surfaces_dir(), &dir.layouts_dir(), &dir.apps_dir()).unwrap_err();
        assert!(errors.iter().any(|e| e.contains(IPAD_KEYMAP_ID)));
    }

    // 不正JSON（構文エラー）は他が正常でも全体を拒否する（=部分適用しない）。
    #[test]
    fn invalid_json_in_one_keymap_rejects_the_whole_reload() {
        let dir = TempDir::new("invalid_json");
        write_minimal_single_keymap(&dir, "ipad01_vol12");
        dir.write("keymaps/keymap_broken.json", "{ this is not json");
        write_empty_deck(&dir);

        let errors = load_startup_data(&dir.keymaps_dir(), &dir.decks_dir(), &dir.surfaces_dir(), &dir.layouts_dir(), &dir.apps_dir()).unwrap_err();
        assert!(!errors.is_empty());
    }

    // keymapディレクトリが空/存在しない場合もエラーとして報告される（起動拒否と同じ扱い）。
    #[test]
    fn empty_keymaps_dir_is_rejected() {
        let dir = TempDir::new("empty_dir");
        write_empty_deck(&dir);
        let errors = load_startup_data(&dir.keymaps_dir(), &dir.decks_dir(), &dir.surfaces_dir(), &dir.layouts_dir(), &dir.apps_dir()).unwrap_err();
        assert!(!errors.is_empty());
    }

    // T11: surfaces/ ディレクトリが無い場合は空レジストリで正常起動する（既存機能に無影響）。
    #[test]
    fn missing_surfaces_dir_does_not_block_startup() {
        let dir = TempDir::new("missing_surfaces");
        write_minimal_single_keymap(&dir, "ipad01_vol12");
        write_empty_deck(&dir);

        let data = load_startup_data(&dir.keymaps_dir(), &dir.decks_dir(), &dir.surfaces_dir(), &dir.layouts_dir(), &dir.apps_dir())
            .expect("missing surfaces dir must not block startup");
        assert!(data.surfaces.is_empty());
    }

    /// T21: リポジトリの実データ（keymaps/・decks/）で起動し、「英数⇄日本語」が撃つ
    /// ALT+GRAVEが許可リストに載っていることを確認する。単体テストだけだと
    /// 「canonical_command_idは正しいが実データでは載っていない」を取り逃すため、
    /// 実ファイル経由で確かめる。
    #[test]
    fn real_data_startup_allows_the_ime_toggle_chord_behind_tg_fire() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let data = load_startup_data(
            &root.join("keymaps"),
            &root.join("decks"),
            &root.join("surfaces"),
            &root.join("layouts"),
            &root.join("apps"),
        )
        .expect("repository data must load");

        assert!(
            data.command_registry.is_allowed("chord:ALT+GRAVE"),
            "the chord nested inside K511's tg.fire must be on the D5 allow-list,              otherwise the IME toggle is rejected at fire time"
        );
    }

    /// P-005 段階A: `decks/`に置いたファイルは全部発見される（keymapsと同じ規則）。
    /// これが無いと「Stream Deckとコピペリストを同時に出す」ができない。
    #[test]
    fn discovers_and_loads_every_deck_file() {
        let dir = TempDir::new("multi_deck");
        write_minimal_single_keymap(&dir, "ipad01_vol12");
        write_empty_deck(&dir);
        dir.write(
            "decks/deck_story_paths.json",
            r#"{ "deckId": "story_paths", "grid": { "cols": 1, "rows": 1 }, "render": "list",
                 "pages": [ { "id": 1, "slots": [
                   { "slotId": "S01", "label": "p", "action": { "t": "text", "string": "C:/x" } } ] } ] }"#,
        );

        let data = load_startup_data(&dir.keymaps_dir(), &dir.decks_dir(), &dir.surfaces_dir(), &dir.layouts_dir(), &dir.apps_dir())
            .expect("both decks must load");
        assert_eq!(data.decks.len(), 2);
        assert!(data.decks.contains_key("default"));
        assert_eq!(data.decks["story_paths"].render, crate::deck::DeckRender::List);
    }

    /// `default`が無い構成は起動拒否（/deckと/panelがdeckId未指定で開くため）。
    #[test]
    fn missing_default_deck_is_rejected() {
        let dir = TempDir::new("no_default_deck");
        write_minimal_single_keymap(&dir, "ipad01_vol12");
        dir.write(
            "decks/deck_other.json",
            r#"{ "deckId": "other", "grid": { "cols": 1, "rows": 1 }, "pages": [] }"#,
        );

        let errors = load_startup_data(&dir.keymaps_dir(), &dir.decks_dir(), &dir.surfaces_dir(), &dir.layouts_dir(), &dir.apps_dir())
            .unwrap_err();
        assert!(errors.iter().any(|e| e.contains("default")), "errors: {errors:?}");
    }

    /// deckIdが重複する構成は起動拒否（どちらが勝つか不定になるのを防ぐ）。
    #[test]
    fn duplicate_deck_id_is_rejected() {
        let dir = TempDir::new("dup_deck_id");
        write_minimal_single_keymap(&dir, "ipad01_vol12");
        write_empty_deck(&dir);
        dir.write(
            "decks/deck_copy.json",
            r#"{ "deckId": "default", "grid": { "cols": 1, "rows": 1 }, "pages": [] }"#,
        );

        let errors = load_startup_data(&dir.keymaps_dir(), &dir.decks_dir(), &dir.surfaces_dir(), &dir.layouts_dir(), &dir.apps_dir())
            .unwrap_err();
        assert!(errors.iter().any(|e| e.contains("duplicate deckId")), "errors: {errors:?}");
    }

    // T11-2: surfaces/trackball.jsonの検証失敗は、他が正常でも起動全体を同じ経路で拒否する
    // （keymap/deckの検証と同じエラー集約経路に乗っていることの確認）。
    #[test]
    fn invalid_surfaces_file_rejects_the_whole_startup() {
        let dir = TempDir::new("invalid_surfaces");
        write_minimal_single_keymap(&dir, "ipad01_vol12");
        write_empty_deck(&dir);
        std::fs::create_dir_all(dir.surfaces_dir()).expect("create surfaces dir");
        dir.write(
            "surfaces/trackball.json",
            r#"{ "surfaces": [ { "id": "tb01", "type": "trackball", "binding": { "t": "not.allowed" } } ] }"#,
        );

        let errors = load_startup_data(&dir.keymaps_dir(), &dir.decks_dir(), &dir.surfaces_dir(), &dir.layouts_dir(), &dir.apps_dir()).unwrap_err();
        assert!(errors.iter().any(|e| e.contains("LOAD_SURFACE_BINDING_UNKNOWN")));
    }
}
