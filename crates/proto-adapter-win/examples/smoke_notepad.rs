//! 手動smoke（T2完了の目印）。自動`cargo test`では実行しない。
//!
//! 使い方:
//!   cargo run -p proto-adapter-win --example smoke_notepad
//! メモ帳を開いてカーソルをテキスト欄に置き、カウントダウン中にメモ帳へフォーカスを移すこと。
//! "A" が1文字入力され、続けて Ctrl+S（保存ダイアログ）が発火すれば成功。

use proto_keymap::Action;

fn main() {
    println!("proto-adapter-win 手動smoke");
    println!("メモ帳を開き、テキスト入力欄にカーソルを置いてフォーカスしてください。");
    for remaining in (1..=5).rev() {
        println!("  {remaining}秒後に送出します…");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    println!("送出: Key A");
    match proto_adapter_win::send(&Action::Key { vk: "A".into() }) {
        Ok(()) => println!("  OK（メモ帳に 'a' が入力されたか確認してください）"),
        Err(error) => println!("  失敗: {error}"),
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    println!("送出: Chord CTRL+S");
    match proto_adapter_win::send(&Action::Chord {
        keys: vec!["CTRL".into(), "S".into()],
    }) {
        Ok(()) => println!("  OK（保存ダイアログが開いたか確認してください。閉じてOK）"),
        Err(error) => println!("  失敗: {error}"),
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    println!("送出: Text \"(「。\" (D20: KEYEVENTF_UNICODE。IME全角/半角状態に関係なく指定どおりに入るはず)");
    match proto_adapter_win::send(&Action::Text {
        string: "(「。".into(),
    }) {
        Ok(()) => println!("  OK（メモ帳に '(「。' がそのまま入力されたか確認してください）"),
        Err(error) => println!("  失敗: {error}"),
    }
}
