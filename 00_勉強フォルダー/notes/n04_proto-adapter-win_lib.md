# n04 — `crates/proto-adapter-win/src/lib.rs`（674行）

## 役割

「決まったアクションを実際にWindowsへ送る」ことだけを担当する crate。
**Win32 APIに触るのはこのcrateだけ**（1〜3行目）。proto-keymapが「何をすべきか」を
決め、このcrateは「それをどう送るか」だけを知っている。

## 前提

- ここに来る時点で、そのアクションは既に許可リストを通過した「意味の確定した
  操作」である（不変条件1）。このファイルは「送ってよいか」を判断しない。
  判断はもっと手前（proto-keymapのresolve・proto-hub）で終わっている。

## 読みどころ

- 37行目 `use proto_keymap::{is_known_vk, Action, MouseButtonKind};` … 依存の向きに注目。
  proto-keymap → proto-adapter-win という一方通行（adapterがkeymapの型を使う）。
- 41〜60行目 `AdapterError` … `SendFailed{cause}`（SendInputの戻り値が送出数と
  不一致）と、OSへ送出できないActionが来た場合のエラー。
- 69〜122行目 `vk_code` / `send(action)` … 公開APIの入口。`send()`が`Action`を受け取り、
  種類ごとに`send_key`/`send_chord`/`send_text`等へ振り分ける。
- 144〜260行目 `send_key` / `send_chord` / `send_text` / `send_mouse_*` /
  `resolve_code` / `release_all_best_effort` … 実際の組み立てロジック。
  - `send_chord`: 修飾キー↓…本体キー↓ → 本体キー↑…修飾キー↑ の順で送る
    （例: CTRL+S は CTRL↓ S↓ S↑ CTRL↑。コメント11〜12行目）。
  - `send_text`: `KEYEVENTF_UNICODE`でIME状態に関係なく文字列を直接注入する
    （設計決定D20）。`str::encode_utf16()`でUTF-16コード単位に分解し、
    絵文字などのサロゲートペアも「1組ずつ」機械的に処理する（21〜27行目）。
- 317行目〜 `mod win { ... }` … 実際のWin32呼び出し（`cfg(windows)`）。
  非Windowsビルド（テスト用）は463行目以降に「ログだけ出して成功扱いにする
  ダミー実装」がある（16行目のコメント）。これのおかげでLinux上でも
  `cargo test`が通る。
- `examples/smoke_notepad.rs`（別ファイル）… 自動テストでは本物のSendInputを
  呼ばない（フォーカス中のウィンドウへ本物のキーが飛ぶと危険なため）。
  代わりに手動実行する確認用バイナリ。実際に動かして確かめたい場合は
  `cargo run -p proto-adapter-win --example smoke_notepad` を試す
  （数秒のカウントダウン中にメモ帳へフォーカスを移すとキーが打たれる）。

## なぜこうなっているか

- Win32依存を1crateに閉じ込めることで、それ以外の全部（proto-keymap・proto-hub・
  フロントエンド）はWindows特有の話を一切知らなくて済む。将来もし別OS対応が
  必要になっても、差し替えるのはこのcrateだけで済む設計。
- panic禁止（設計決定D9）: 未対応のActionが来ても`AdapterError::Unsupported`を
  返すだけで、プロセスを落とさない。実際にはHub側で弾かれて到達しない想定だが、
  「防御として」ここでも二重に守っている。

## 理解度チェック

1. なぜ`send_chord`は「修飾→本体→本体→修飾」という順序でなければならないのか？
2. `Text`アクションが`is_known_vk`の辞書チェックを経由しないのはなぜか？
3. なぜ自動テストの中で本物のSendInputを呼ばないのか。呼んでしまうと何が起きるか？
