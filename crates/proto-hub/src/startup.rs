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
use crate::state::{canonical_command_id, IPAD_KEYMAP_ID};

#[derive(Debug)]
pub struct StartupData {
    pub keymaps: BTreeMap<String, Keymap>,
    pub deck: DeckSetlist,
    pub command_registry: hub_core::CommandRegistry,
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

/// 起動時（main.rs）／再読込時（ws.rsの`/api/reload`）で共有する検証手順。
/// 順序: ①ディレクトリスキャンで発見した全keymapファイルのロード ②ipad面固定keymapId
/// の存在確認 ③deckのロード ④deck内`keymap.switch`参照先の存在確認。
/// 1件でもエラーがあれば集約して`Err(Vec<String>)`を返す（部分適用はしない）。
pub fn load_startup_data(keymaps_dir: &Path, deck_path: &Path) -> Result<StartupData, Vec<String>> {
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

    let deck = match crate::deck::load_deck_from_path(deck_path) {
        Ok(deck) => Some(deck),
        Err(error) => {
            errors.push(format!("{}: {error}", deck_path.display()));
            None
        }
    };

    if let Some(deck) = &deck {
        if errors.is_empty() {
            for action in all_actions(&keymaps, deck) {
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
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let deck = deck.expect("deck load succeeded because errors is empty");
    let command_ids: Vec<String> = all_actions(&keymaps, &deck)
        .filter_map(canonical_command_id)
        .collect();
    let command_registry = hub_core::CommandRegistry::new(command_ids);

    Ok(StartupData {
        keymaps,
        deck,
        command_registry,
    })
}

fn all_actions<'a>(
    keymaps: &'a BTreeMap<String, Keymap>,
    deck: &'a DeckSetlist,
) -> impl Iterator<Item = &'a Action> {
    keymaps
        .values()
        .flat_map(|keymap| keymap.layers.iter())
        .flat_map(|layer| layer.keys.values())
        .map(|key_def| &key_def.action)
        .chain(deck.actions())
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

        fn deck_path(&self) -> PathBuf {
            self.0.join("decks/deck_default.json")
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
            r#"{ "deckId": "deck_default", "grid": { "cols": 1, "rows": 1 }, "pages": [] }"#,
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

        let data = load_startup_data(&dir.keymaps_dir(), &dir.deck_path())
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

        let errors = load_startup_data(&dir.keymaps_dir(), &dir.deck_path()).unwrap_err();
        assert!(errors.iter().any(|e| e.contains(IPAD_KEYMAP_ID)));
    }

    // 不正JSON（構文エラー）は他が正常でも全体を拒否する（=部分適用しない）。
    #[test]
    fn invalid_json_in_one_keymap_rejects_the_whole_reload() {
        let dir = TempDir::new("invalid_json");
        write_minimal_single_keymap(&dir, "ipad01_vol12");
        dir.write("keymaps/keymap_broken.json", "{ this is not json");
        write_empty_deck(&dir);

        let errors = load_startup_data(&dir.keymaps_dir(), &dir.deck_path()).unwrap_err();
        assert!(!errors.is_empty());
    }

    // keymapディレクトリが空/存在しない場合もエラーとして報告される（起動拒否と同じ扱い）。
    #[test]
    fn empty_keymaps_dir_is_rejected() {
        let dir = TempDir::new("empty_dir");
        write_empty_deck(&dir);
        let errors = load_startup_data(&dir.keymaps_dir(), &dir.deck_path()).unwrap_err();
        assert!(!errors.is_empty());
    }
}
