//! proto-adapter-win — 汎用SendInput（T2）
//!
//! 設計書D7/D9。本crateだけがWin32に触る。proto-keymapのvk辞書 → Windows VKコード表を持つ。
//!
//! ── 実装するもの（元の骨組みコメントをそのまま維持）─────────────
//! 1. [T2-1] vk名→VKコード変換表（D4辞書の全エントリを網羅。漏れがあればコンパイル時 or
//!    テストで検出できる形にする。例: match網羅 + 全辞書を回す単体テスト）
//!    - メディア系は VK_VOLUME_MUTE / VK_VOLUME_DOWN / VK_VOLUME_UP /
//!      VK_MEDIA_PLAY_PAUSE / VK_MEDIA_NEXT_TRACK / VK_MEDIA_PREV_TRACK。
//! 2. [T2-2] send(action) -> Result<(), AdapterError>
//!    - Key{vk}: down→up を1組送出。
//!    - Chord{keys}: 修飾↓…本体↓ → 本体↑…修飾↑ の順（例 CTRL+S: CTRL↓ S↓ S↑ CTRL↑）。
//!    - 失敗（SendInput戻り値が送出数と不一致）は AdapterError::SendFailed
//!      → 呼び出し側がD9の code=ADAPTER_SENDINPUT_FAIL で整形。cause にvk名と戻り値を入れる。
//!    - 呼び出しは直列前提（キュー管理はHub側の責務。本crateは同期関数でよい）。
//!    - 非Windowsビルド用に cfg(not(windows)) では「ログだけ出して成功扱い」のダミーを用意
//!      （テスト・CI用。ダミー使用時はその旨をログに1回出す）。
//! 3. panic禁止（D9）。未対応Action（Mo/Tg/Trans等が来たら）AdapterError::Unsupported。
//!    ※ 本来Hub側で弾かれて到達しないが、防御として。
//!
//! ── T8: Text Action（D20。v0.4のkeymap_design_v0.4.md D20）─────────────
//! 4. [T8-1] Text{string}: KEYEVENTF_UNICODEでIME状態に依存せず文字列を直接注入する。
//!    - vk辞書は使わない（wVk=0、wScanにUTF-16コード単位を積む方式）。
//!    - サロゲートペア対応: `str::encode_utf16()`でUTF-16コード単位に分解し、
//!      各コード単位ごとにdown→upを1組ずつ送出する（絵文字等の非BMP文字も1組の
//!      サロゲートペア=2組のdown/upとして機械的に扱える）。
//!    - 既存のkey/chord経路（vk辞書ベース）はいっさい変更しない。
//!
//! ── smoke（T2完了の目印）────────────────────────────────────
//! 自動`cargo test`の中では実際にSendInputを呼ばない（フォーカス中の任意ウィンドウへ本物の
//! キーが飛んでしまうため、CI・通常のテスト実行では危険）。代わりに手動実行用のバイナリ
//! `examples/smoke_notepad.rs` を用意した:
//!   cargo run -p proto-adapter-win --example smoke_notepad
//! 数秒のカウントダウン中にメモ帳へフォーカスを移すと、"A" の入力と Ctrl+S 相当の
//! chord送出（保存ダイアログが出るはず）が実行される。

use proto_keymap::{is_known_vk, Action};

/// send()が返す失敗理由。呼び出し側（proto-hub）がD9のエラーコードへ整形する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    /// SendInputの戻り値が送出予定数と一致しなかった場合。cause=「vk名: 詳細」。
    /// 呼び出し側は code=ADAPTER_SENDINPUT_FAIL として整形すること。
    SendFailed { cause: String },
    /// Mo/Tg/Trans/None/KeymapSwitch/KeymapReset等、OSへ送出できないActionが渡された場合。
    /// 本来はproto-keymap::resolve()のResolved::Fireにこれらは現れないため到達しない想定だが、
    /// 防御的にエラーとして扱う（panicしない）。
    Unsupported { cause: String },
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SendFailed { cause } => write!(f, "SendFailed: {cause}"),
            Self::Unsupported { cause } => write!(f, "Unsupported: {cause}"),
        }
    }
}

impl std::error::Error for AdapterError {}

// ============================================================================
// [T2-1] vk名 → Windows VKコード表。VK_DICTIONARY（proto-keymap）の全エントリを網羅する。
// 網羅性は下部の単体テストで担保する（漏れがあればテストが失敗する）。
// ============================================================================

/// D4のvk名をWindows仮想キーコード（u16）へ変換する。この関数自体はWin32の型に依存しない
/// ため、非Windows環境でも変換ロジックの正しさをテストできる。
pub fn vk_code(vk: &str) -> Option<u16> {
    Some(match vk {
        "A" => 0x41, "B" => 0x42, "C" => 0x43, "D" => 0x44, "E" => 0x45, "F" => 0x46,
        "G" => 0x47, "H" => 0x48, "I" => 0x49, "J" => 0x4A, "K" => 0x4B, "L" => 0x4C,
        "M" => 0x4D, "N" => 0x4E, "O" => 0x4F, "P" => 0x50, "Q" => 0x51, "R" => 0x52,
        "S" => 0x53, "T" => 0x54, "U" => 0x55, "V" => 0x56, "W" => 0x57, "X" => 0x58,
        "Y" => 0x59, "Z" => 0x5A,
        "0" => 0x30, "1" => 0x31, "2" => 0x32, "3" => 0x33, "4" => 0x34,
        "5" => 0x35, "6" => 0x36, "7" => 0x37, "8" => 0x38, "9" => 0x39,
        "F1" => 0x70, "F2" => 0x71, "F3" => 0x72, "F4" => 0x73, "F5" => 0x74, "F6" => 0x75,
        "F7" => 0x76, "F8" => 0x77, "F9" => 0x78, "F10" => 0x79, "F11" => 0x7A, "F12" => 0x7B,
        "F13" => 0x7C, "F14" => 0x7D, "F15" => 0x7E, "F16" => 0x7F, "F17" => 0x80, "F18" => 0x81,
        "F19" => 0x82, "F20" => 0x83, "F21" => 0x84, "F22" => 0x85, "F23" => 0x86, "F24" => 0x87,
        "ENTER" => 0x0D,
        "ESC" => 0x1B,
        "TAB" => 0x09,
        "SPACE" => 0x20,
        "BKSP" => 0x08,
        "DEL" => 0x2E,
        "UP" => 0x26,
        "DOWN" => 0x28,
        "LEFT" => 0x25,
        "RIGHT" => 0x27,
        "CTRL" => 0x11,
        "SHIFT" => 0x10,
        "ALT" => 0x12,
        "WIN" => 0x5B,
        "COMMA" => 0xBC,
        "PERIOD" => 0xBE,
        "SLASH" => 0xBF,
        "SEMICOLON" => 0xBA,
        "QUOTE" => 0xDE,
        "MINUS" => 0xBD,
        "EQUALS" => 0xBB,
        "LBRACKET" => 0xDB,
        "RBRACKET" => 0xDD,
        "BACKSLASH" => 0xDC,
        "GRAVE" => 0xC0,
        "VOL_UP" => 0xAF,
        "VOL_DOWN" => 0xAE,
        "MUTE" => 0xAD,
        "MEDIA_PLAY" => 0xB3,
        "MEDIA_NEXT" => 0xB0,
        "MEDIA_PREV" => 0xB1,
        _ => return None,
    })
}

// ============================================================================
// [T2-2] send(action)
// ============================================================================

/// D4のActionをOSへ送出する。Key/Chord/Text以外はUnsupported（防御。呼び出し側で弾かれる想定）。
pub fn send(action: &Action) -> Result<(), AdapterError> {
    match action {
        Action::Key { vk } => send_key(vk),
        Action::Chord { keys } => send_chord(keys),
        Action::Text { string } => send_text(string),
        other => Err(AdapterError::Unsupported {
            cause: format!("action cannot be sent to the OS: {other:?}"),
        }),
    }
}

fn send_key(vk: &str) -> Result<(), AdapterError> {
    let code = resolve_code(vk)?;
    press(code, vk)?;
    release(code, vk)?;
    Ok(())
}

fn send_chord(keys: &[String]) -> Result<(), AdapterError> {
    let (modifiers, main) = match keys.split_last() {
        Some((last, rest)) => (rest, last),
        None => {
            return Err(AdapterError::Unsupported {
                cause: "chord must contain at least one key".into(),
            })
        }
    };

    // 修飾↓ …本体↓ → 本体↑ … 修飾↑ の順（例 CTRL+S: CTRL↓ S↓ S↑ CTRL↑）。
    let mut pressed: Vec<(u16, &str)> = Vec::with_capacity(modifiers.len());
    for modifier in modifiers {
        let code = match resolve_code(modifier) {
            Ok(code) => code,
            Err(error) => {
                release_all_best_effort(&pressed);
                return Err(error);
            }
        };
        if let Err(error) = press(code, modifier) {
            release_all_best_effort(&pressed);
            return Err(error);
        }
        pressed.push((code, modifier.as_str()));
    }

    let main_result = resolve_code(main).and_then(|code| {
        press(code, main)?;
        release(code, main)
    });

    // 本体キーの成否に関わらず、押しっぱなしの修飾キーを残さないよう逆順で必ず離す。
    release_all_best_effort(&pressed);

    main_result
}

/// [T8-1] D20: KEYEVENTF_UNICODEで文字列を直接注入する。vk辞書は経由しない。
/// サロゲートペア対応のためUTF-16コード単位ごとにdown→upを1組ずつ送出する。
fn send_text(string: &str) -> Result<(), AdapterError> {
    if string.is_empty() {
        return Err(AdapterError::Unsupported {
            cause: "text action string must not be empty".into(),
        });
    }
    for code_unit in string.encode_utf16() {
        press_unicode(code_unit)?;
        release_unicode(code_unit)?;
    }
    Ok(())
}

fn resolve_code(vk: &str) -> Result<u16, AdapterError> {
    if !is_known_vk(vk) {
        // proto-keymapのロード検証を通っていればここには来ないはずだが、防御的に扱う。
        return Err(AdapterError::Unsupported {
            cause: format!("'{vk}' is not in the vk dictionary"),
        });
    }
    vk_code(vk).ok_or_else(|| AdapterError::Unsupported {
        cause: format!("'{vk}' has no VK mapping in proto-adapter-win"),
    })
}

fn release_all_best_effort(pressed: &[(u16, &str)]) {
    for (code, name) in pressed.iter().rev() {
        let _ = release(*code, name);
    }
}

// ============================================================================
// Windows実送出（cfg(windows)）
// ============================================================================

#[cfg(windows)]
fn press(code: u16, vk_name: &str) -> Result<(), AdapterError> {
    win::send_vk(code, false)
        .map_err(|cause| AdapterError::SendFailed { cause: format!("{vk_name}: {cause}") })
}

#[cfg(windows)]
fn release(code: u16, vk_name: &str) -> Result<(), AdapterError> {
    win::send_vk(code, true)
        .map_err(|cause| AdapterError::SendFailed { cause: format!("{vk_name}: {cause}") })
}

#[cfg(windows)]
fn press_unicode(code_unit: u16) -> Result<(), AdapterError> {
    win::send_unicode(code_unit, false)
        .map_err(|cause| AdapterError::SendFailed { cause: format!("text U+{code_unit:04X}: {cause}") })
}

#[cfg(windows)]
fn release_unicode(code_unit: u16) -> Result<(), AdapterError> {
    win::send_unicode(code_unit, true)
        .map_err(|cause| AdapterError::SendFailed { cause: format!("text U+{code_unit:04X}: {cause}") })
}

#[cfg(windows)]
mod win {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VIRTUAL_KEY,
    };

    pub fn send_vk(vk: u16, key_up: bool) -> Result<(), String> {
        let flags = if key_up {
            KEYEVENTF_KEYUP
        } else {
            KEYBD_EVENT_FLAGS(0)
        };
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk),
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
        if sent != 1 {
            return Err(format!("SendInput returned {sent}, expected 1"));
        }
        Ok(())
    }

    /// D20: KEYEVENTF_UNICODE。wVkは0固定・wScanにUTF-16コード単位を積む
    /// （windows crateのKEYBDINPUT.wScan経由。IME状態に依存しない直接注入）。
    pub fn send_unicode(code_unit: u16, key_up: bool) -> Result<(), String> {
        let mut flags = KEYEVENTF_UNICODE;
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: code_unit,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
        if sent != 1 {
            return Err(format!("SendInput returned {sent}, expected 1"));
        }
        Ok(())
    }
}

// ============================================================================
// 非Windowsダミー（テスト・CI用。ログだけ出して成功扱い）
// ============================================================================

#[cfg(not(windows))]
fn press(code: u16, vk_name: &str) -> Result<(), AdapterError> {
    let _ = code;
    dummy_log();
    let _ = vk_name;
    Ok(())
}

#[cfg(not(windows))]
fn release(code: u16, vk_name: &str) -> Result<(), AdapterError> {
    let _ = code;
    dummy_log();
    let _ = vk_name;
    Ok(())
}

#[cfg(not(windows))]
fn press_unicode(code_unit: u16) -> Result<(), AdapterError> {
    let _ = code_unit;
    dummy_log();
    Ok(())
}

#[cfg(not(windows))]
fn release_unicode(code_unit: u16) -> Result<(), AdapterError> {
    let _ = code_unit;
    dummy_log();
    Ok(())
}

#[cfg(not(windows))]
fn dummy_log() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        eprintln!(
            "[KD][T2-2][INFO] non-Windows build: proto-adapter-win uses a no-op dummy \
             (SendInput is not called; this call always succeeds)"
        );
    });
}

// ============================================================================
// 単体テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use proto_keymap::VK_DICTIONARY;

    // [T2-1] VK_DICTIONARY（proto-keymapの正）の全エントリがvk_code()でマッピングされていること。
    // 漏れがあればこのテストが失敗し、追加漏れをコンパイル後すぐに検出できる。
    #[test]
    fn t2_1_vk_code_covers_the_entire_dictionary() {
        for vk in VK_DICTIONARY {
            assert!(
                vk_code(vk).is_some(),
                "vk_code() is missing a mapping for dictionary entry '{vk}'"
            );
        }
    }

    #[test]
    fn t2_1_unknown_vk_name_maps_to_none() {
        assert_eq!(vk_code("NOT_A_REAL_KEY"), None);
    }

    #[test]
    fn t2_1_vk_codes_are_unique() {
        use std::collections::BTreeSet;
        let mut seen = BTreeSet::new();
        for vk in VK_DICTIONARY {
            let code = vk_code(vk).unwrap();
            assert!(
                seen.insert(code),
                "vk '{vk}' maps to code {code:#x} which is already used by another vk"
            );
        }
    }

    // [T2-2] Unsupported経路（OSに実際に触れない範囲）はどの環境でも自動テストできる。
    #[test]
    fn t2_2_mo_action_is_unsupported() {
        let error = send(&Action::Mo { layer: 1 }).unwrap_err();
        assert!(matches!(error, AdapterError::Unsupported { .. }));
    }

    #[test]
    fn t2_2_trans_action_is_unsupported() {
        let error = send(&Action::Trans).unwrap_err();
        assert!(matches!(error, AdapterError::Unsupported { .. }));
    }

    #[test]
    fn t2_2_empty_chord_is_unsupported() {
        let error = send(&Action::Chord { keys: vec![] }).unwrap_err();
        assert!(matches!(error, AdapterError::Unsupported { .. }));
    }

    #[test]
    fn t2_2_unknown_vk_in_key_action_is_unsupported_not_panic() {
        let error = send(&Action::Key {
            vk: "NOT_A_REAL_KEY".into(),
        })
        .unwrap_err();
        assert!(matches!(error, AdapterError::Unsupported { .. }));
    }

    // [T8-1] D20: 空文字列のtextはUnsupported（送出する内容が無いため）。
    #[test]
    fn t8_1_empty_text_action_is_unsupported_not_panic() {
        let error = send(&Action::Text { string: String::new() }).unwrap_err();
        assert!(matches!(error, AdapterError::Unsupported { .. }));
    }

    // 非Windows環境（CI等）ではKey/Chordの送出がダミーとして成功することを確認する。
    // Windows実機ではsend()が実際にSendInputを呼ぶため、このテストはここでは実行しない
    // （実機smokeは examples/smoke_notepad.rs を手動実行すること）。
    #[cfg(not(windows))]
    #[test]
    fn t2_2_dummy_backend_reports_success_for_key_and_chord() {
        assert!(send(&Action::Key { vk: "A".into() }).is_ok());
        assert!(send(&Action::Chord {
            keys: vec!["CTRL".into(), "S".into()]
        })
        .is_ok());
    }

    // [T8-1] 非Windows環境ではtext送出もダミーとして成功する。全角記号・非BMP文字
    // （サロゲートペア）を含めて、encode_utf16の分解経路がpanicしないことを確認する。
    #[cfg(not(windows))]
    #[test]
    fn t8_1_dummy_backend_reports_success_for_text_including_surrogate_pairs() {
        assert!(send(&Action::Text { string: "(".into() }).is_ok());
        assert!(send(&Action::Text { string: "。、「」".into() }).is_ok());
        // 🎉 U+1F389 は非BMP文字でありUTF-16ではサロゲートペア(2コード単位)になる。
        assert!(send(&Action::Text { string: "🎉".into() }).is_ok());
    }
}
