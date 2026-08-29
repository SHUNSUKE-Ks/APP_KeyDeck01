//! 手動smoke（T16完了の目印）。自動`cargo test`では実行しない。
//!
//! 使い方:
//!   cargo run -p proto-adapter-win --example smoke_mouse_click
//! カウントダウン中にメモ帳やテキストエディタなどへフォーカスを移し、カーソルを
//! クリックしやすい場所に置いておくこと。**SendInputは実際にクリック/ボタン押下/
//! スクロールを送出するため、自動テストからは絶対に呼ばない**
//! （proto-adapter-win/src/lib.rsのコメント・brief/keydeck_trackball_gestures_v0.7.md T16参照）。

use proto_keymap::{Action, MouseButtonKind};

fn main() {
    println!("proto-adapter-win 手動smoke（マウスクリック/ダブルクリック/ボタン/スクロール）");
    println!("クリックされても問題ない場所にカーソルを置いてください。");
    for remaining in (1..=5).rev() {
        println!("  {remaining}秒後に送出します…");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    println!("送出: MouseClick(Left)");
    match proto_adapter_win::send(&Action::MouseClick { button: MouseButtonKind::Left }) {
        Ok(()) => println!("  OK（左クリックされたか確認してください）"),
        Err(error) => println!("  失敗: {error}"),
    }
    std::thread::sleep(std::time::Duration::from_millis(800));

    println!("送出: MouseDoubleClick(Left)");
    match proto_adapter_win::send(&Action::MouseDoubleClick { button: MouseButtonKind::Left }) {
        Ok(()) => println!("  OK（ダブルクリック=単語選択等が起きたか確認してください）"),
        Err(error) => println!("  失敗: {error}"),
    }
    std::thread::sleep(std::time::Duration::from_millis(800));

    println!("送出: MouseClick(Right)");
    match proto_adapter_win::send(&Action::MouseClick { button: MouseButtonKind::Right }) {
        Ok(()) => println!("  OK（右クリックメニューが出たか確認してください）"),
        Err(error) => println!("  失敗: {error}"),
    }
    std::thread::sleep(std::time::Duration::from_millis(800));

    println!("送出: MouseButton(Left, down=true) → 1秒後 down=false（押しっぱなし確認）");
    match proto_adapter_win::send(&Action::MouseButton { button: MouseButtonKind::Left, down: true }) {
        Ok(()) => println!("  OK（downを送出。この間ドラッグ選択できるか確認してください）"),
        Err(error) => println!("  失敗: {error}"),
    }
    std::thread::sleep(std::time::Duration::from_secs(1));
    match proto_adapter_win::send(&Action::MouseButton { button: MouseButtonKind::Left, down: false }) {
        Ok(()) => println!("  OK（upを送出。ボタンが離れたか確認してください）"),
        Err(error) => println!("  失敗: {error}"),
    }
    std::thread::sleep(std::time::Duration::from_millis(800));

    println!("送出: MouseScroll(dy=5) x3（下スクロール相当）");
    for _ in 0..3 {
        match proto_adapter_win::send(&Action::MouseScroll { dy: 5 }) {
            Ok(()) => println!("  OK"),
            Err(error) => println!("  失敗: {error}"),
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    println!("完了: 各操作がフォーカス中のウィンドウへ反映されていれば成功です。");
}
