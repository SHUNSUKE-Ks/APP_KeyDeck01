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
