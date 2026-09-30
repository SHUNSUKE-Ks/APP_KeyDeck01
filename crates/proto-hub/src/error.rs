//! D9のエラーコード（proto-hubが生成する範囲）。LOAD_*系はproto_keymapのものを再輸出する。

pub const WS_TOKEN_INVALID: &str = "WS_TOKEN_INVALID";
pub const WS_PARSE: &str = "WS_PARSE";
pub const KEY_UNKNOWN_ID: &str = proto_keymap::KEY_UNKNOWN_ID;
pub const KEY_RESOLVE_NONE: &str = proto_keymap::KEY_RESOLVE_NONE;
pub const ADAPTER_SENDINPUT_FAIL: &str = "ADAPTER_SENDINPUT_FAIL";
pub const KEYMAP_SWITCH_UNKNOWN: &str = "KEYMAP_SWITCH_UNKNOWN";
pub const DECK_UNKNOWN_SLOT: &str = "DECK_UNKNOWN_SLOT";
pub const INTERNAL: &str = "INTERNAL";
/// B2（設計書v0.5）: `/api/reload`のディスク再読込・検証に1件でも失敗した場合。
/// 現行構成は一切変更されない（呼び出し元は`startup::load_startup_data`のErrで判定する）。
pub const RELOAD_INVALID: &str = "RELOAD_INVALID";

/// D11: Deck/Keymapの起動時ロード検証で使う。中身はproto_keymapと同じ文字列だが、
/// deck.rsのロード処理はproto-hub側にあるためここにも定数として持つ（値は1箇所の文字列に一致）。
pub const LOAD_JSON_SYNTAX: &str = proto_keymap::LOAD_JSON_SYNTAX;
pub const LOAD_SCHEMA_INVALID: &str = proto_keymap::LOAD_SCHEMA_INVALID;
pub const LOAD_VK_UNKNOWN: &str = proto_keymap::LOAD_VK_UNKNOWN;

/// P-005 段階D: レイアウト保存の拒否（検証に落ちた／idの形が不正）。
/// ディスクは変更されないか、変更されても巻き戻される。
pub const LAYOUT_SAVE_REJECTED: &str = "LAYOUT_SAVE_REJECTED";
/// P-005 段階D: レイアウト保存の失敗（ファイル操作そのものが失敗した）。
pub const LAYOUT_SAVE_FAILED: &str = "LAYOUT_SAVE_FAILED";

/// P-007: レイヤー保存の拒否（keymapId/layerが実在しない、または検証に落ちた）。
pub const LAYER_SAVE_REJECTED: &str = "LAYER_SAVE_REJECTED";
/// P-007: レイヤー保存の失敗（ファイル操作そのものが失敗した）。
pub const LAYER_SAVE_FAILED: &str = "LAYER_SAVE_FAILED";

/// P-007 段階B: 盤面保存の拒否（keymapIdが実在しない／検証に落ちた）。
pub const BOARD_SAVE_REJECTED: &str = "BOARD_SAVE_REJECTED";
/// P-007 段階B: 盤面保存の失敗（ファイル操作そのものが失敗した）。
pub const BOARD_SAVE_FAILED: &str = "BOARD_SAVE_FAILED";

/// V2.1: Deck保存の拒否（deckIdの形が不正／検証に落ちた）。
/// ディスクは変更されないか、変更されても巻き戻される。
pub const DECK_SAVE_REJECTED: &str = "DECK_SAVE_REJECTED";
/// V2.1: Deck保存の失敗（ファイル操作そのものが失敗した）。
pub const DECK_SAVE_FAILED: &str = "DECK_SAVE_FAILED";

/// 2026-09-25: アイコン保存の拒否（nameの形が不正／PNG・JPEG・WebP以外／空／1MiB超）。
/// ディスクは変更されない。
pub const ICON_SAVE_REJECTED: &str = "ICON_SAVE_REJECTED";
/// 2026-09-25: アイコン保存の失敗（ファイル操作そのもの、または読み直しの不一致）。巻き戻される。
pub const ICON_SAVE_FAILED: &str = "ICON_SAVE_FAILED";

/// P-008: `device=<id>` で名乗った端末が `devices/devices.json` に無い。WS の確立前に断る。
pub const WS_DEVICE_UNKNOWN: &str = "WS_DEVICE_UNKNOWN";
