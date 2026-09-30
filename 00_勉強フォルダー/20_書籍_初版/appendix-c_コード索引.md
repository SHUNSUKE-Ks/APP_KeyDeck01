# 付録 C　コード索引

本書で引用・言及した関数と型の場所です。**行番号は 2026-09-15 時点** のもので、コードが変わればずれます。
ずれていたら関数名で検索してください（関数名のほうが長持ちします）。

| 章 | ファイル | 関数・型 | 行 |
|---|---|---|---|
| 1 | `crates/proto-hub/src/ws.rs` | `router` | 40 |
| 2 | `crates/proto-hub/src/protocol.rs` | `ClientMessage` | 12 |
| 2 | `crates/proto-hub/src/state.rs` | `AccessToken`、`is_valid` | 59、81 |
| 2 | `crates/proto-hub/src/state.rs` | `canonical_command_id` | 109 |
| 3 | `crates/proto-keymap/src/lib.rs` | `load_keymap_from_path` | 419 |
| 4 | `crates/proto-hub/src/startup.rs` | `load_startup_data` | 90 |
| 4 | `crates/proto-hub/src/startup.rs` | `all_actions_with_source`、`all_actions` | 281、305 |
| 5 | `crates/proto-hub/src/ws.rs` | `ws_handler` | 188 |
| 5 | `crates/proto-hub/src/ws.rs` | `handle_socket` | 1471 |
| 5 | `crates/proto-hub/src/ws.rs` | `handle_client_text` | 1516 |
| 6 | `crates/proto-keymap/src/lib.rs` | `Edge`、`LayerState`、`active_layers` | 846、853、878 |
| 6 | `crates/proto-keymap/src/lib.rs` | `Resolved` | 888 |
| 6 | `crates/proto-keymap/src/lib.rs` | `resolve` | 908 |
| 6 | `crates/proto-hub/src/ws.rs` | `handle_key_press` | 1627 |
| 6 | `crates/proto-hub/src/ws.rs` | `broadcast_layer_state` | 2278 |
| 7 | `crates/proto-hub/src/ws.rs` | `fire_action` | 2010 |
| 7 | `crates/proto-adapter-win/src/lib.rs` | `send` | 123 |
| 7 | `crates/proto-adapter-win/src/lib.rs` | `send_chord` | 152 |
| 7 | `crates/proto-adapter-win/src/lib.rs` | `send_text` | 192 |
| 8 | `crates/proto-adapter-win/src/lib.rs` | `send_key` | 145 |
| 8 | `crates/proto-hub/src/ws.rs` | `release_held_keys` | 1977 |
| 9 | `static/layout.html` | `renderComponent` | 437 |
| 9 | `static/components.js` | `fitDeck` | 692 |
| 10 | `crates/proto-hub/src/ws.rs` | `handle_surface_state` | 1736 |
| 10 | `static/trackball.html` | `MIN_INTERVAL`（送信間隔 8 ミリ秒） | 746 |
| 11 | `crates/proto-hub/src/ws.rs` | `layout_id_is_safe` | 561 |
| 11 | `crates/proto-hub/src/ws.rs` | `layout_save_handler` | 582 |
| 12 | `crates/proto-hub/src/app_launch.rs` | `launch` | 225 |
| 12 | `crates/proto-hub/src/app_launch.rs` | テスト `refuses_script_interpreters` | 286 |

## 引用した JSON

| 章 | ファイル |
|---|---|
| 3, 6 | `keymaps/layers/ipad01_vol12_layer0.json`、`keymaps/layers/ipad01_vol12_layer1.json` |
| 3 | `keymaps/keymap_ipad01_vol12.json` |
| 9 | `layouts/layout_ipad_main.json` |
| 10 | `surfaces/trackball.json`、`keymaps/keymap_jog_frame.json` |
| 12 | `apps/apps.json` |

## 本書で扱わなかった主なファイル

| ファイル | 中身 |
|---|---|
| `crates/hub-core/` | 外部から取り込んだ凍結コピー。解説対象外 |
| `crates/proto-hub/src/layout.rs` | ボードの型と、重なり・はみ出しの検査 |
| `crates/proto-hub/src/surface.rs` | 面の JSON の読み込みと出口の検査 |
| `crates/proto-hub/src/deck.rs` | Deck の型と検査 |
| `crates/proto-hub/src/error.rs` | Hub のエラーコード一覧 |
| `static/editor.html`、`static/keys.html` | ボードとキーの編集画面 |
| `static/trackball.html` | トラックボールのジェスチャーの判定 |
