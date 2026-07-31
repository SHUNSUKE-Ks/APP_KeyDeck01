//! 手動smoke（T10完了の目印）。自動`cargo test`では実行しない。
//!
//! 使い方:
//!   cargo run -p proto-adapter-win --example smoke_mouse
//! カウントダウン中に何もフォーカスを移す必要はない（マウス移動はフォーカス非依存）。
//! 実カーソルが小さく動けば成功。**SendInputは実カーソルを本当に動かすため、
//! 自動テストからは絶対に呼ばない**（proto-adapter-win/src/lib.rsのコメント・
//! brief/keydeck_trackball_design_v0.6.md T10参照）。

use proto_keymap::Action;

fn main() {
    println!("proto-adapter-win 手動smoke（マウス相対移動）");
    println!("カーソルの現在位置を見ながら実行してください。");
    for remaining in (1..=5).rev() {
        println!("  {remaining}秒後に送出します…");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    let steps: [(i32, i32); 4] = [(50, 0), (0, 50), (-50, 0), (0, -50)];
    for (dx, dy) in steps {
        println!("送出: MouseMove dx={dx} dy={dy}");
        match proto_adapter_win::send(&Action::MouseMove { dx, dy }) {
            Ok(()) => println!("  OK（カーソルが少し動いたか確認してください）"),
            Err(error) => println!("  失敗: {error}"),
        }
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    println!("完了: 四辺を小さく描いて元の位置付近へ戻っていれば成功です。");
}
