# KeyDeck 技術書・Claude Code 引継ぎの絶対パス参照一覧

引継ぎの正本は、次の2ファイルです。Claude Code へ渡すときは、この別紙も一緒に渡してください。

- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\12_ClaudeCode引継ぎ_書籍執筆依頼.md`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\13_初版_完成条件チェックリスト.md`

## 作業開始時に読む資料

- `C:\00_master\AGENTS.md`
- `C:\00_master\DevApps\APP_KeyDeck01\CLAUDE.md`
- `C:\00_master\DevApps\APP_KeyDeck01\INDEX.md`
- `C:\00_master\DevApps\APP_KeyDeck01\DEVBOARD.md`
- `C:\00_master\DevApps\APP_KeyDeck01\docs\BOOK_REQUEST_CODEX.md`
- `C:\Users\enjoy\.codex\skills\bookmaker\SKILL.md`

## 原稿・図・問題の既存成果物

- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\02_KeyDeck技術書_初稿.md`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\03_本のレイアウト検討.md`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\04_確認問題.md`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\05_確認問題_解答.md`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\06_技術書_第2稿_第1〜5章差し替え.md`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\07_状態変化図_レイアウト見本.html`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\08_状態変化図_縦フロー見本.html`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\09_Fn状態変化_詳細.html`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\10_状態変化図_矢印入り見本.html`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\11_状態変化図_最終版v1.1.html`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\schemas\bookmaker-diagram-v1.1.schema.json`
- `C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\examples\fn-momentary-bookmaker-v1.1.json`

## 本文で引用する主な実装

- `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-keymap\src\lib.rs` — `resolve`
- `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-hub\src\state.rs` — `canonical_command_id`
- `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-hub\src\ws.rs` — `router`、`handle_socket`、`handle_key_press`
- `C:\00_master\DevApps\APP_KeyDeck01\crates\proto-adapter-win\src\lib.rs` — `send_text`
- `C:\00_master\DevApps\APP_KeyDeck01\static\layout.html` — `renderComponent`
- `C:\00_master\DevApps\APP_KeyDeck01\static\components.js` — `fitDeck`

## 実スクリーンショット候補（内容を確認してから使う）

- `C:\00_master\DevApps\APP_KeyDeck01\static\shots\layout_ipad_v13.png`
- `C:\00_master\DevApps\APP_KeyDeck01\static\shots\layout_ipad_trackball.png`
- `C:\00_master\DevApps\APP_KeyDeck01\static\shots\layout_ipad_main.png`

既存画像は、操作前・操作後が同一シナリオであると確認できた場合だけ使用します。確認できない場合は、利用者が次を手動で実行して port 8770 の画面を撮影します。Hub は自動起動しません。

```powershell
Set-Location C:\00_master\DevApps\APP_KeyDeck01
cargo run -p proto-hub
```

## 新規成果物の置き場所

新しい原稿、図の JSON、詳細ページ、画像注釈 HTML はすべて次に追加します。既存ファイルは消さず、版を上げた名前にします。

`C:\00_master\DevApps\APP_KeyDeck01\00_勉強フォルダー\`

## 作業報告の置き場所

各作業単位の報告は、次の配下に作成します。

`C:\00_master\DevApps\APP_KeyDeck01\.harness\reports\<reportId>\report.json`

`C:\00_master\DevApps\APP_KeyDeck01\.harness\reports\<reportId>\report_body.md`
