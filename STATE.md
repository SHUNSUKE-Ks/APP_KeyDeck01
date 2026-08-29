# APP_KeyDeck01 STATE

最終更新: 2026-08-29（チャットリセット直前の引き継ぎ用に更新。旧内容はDEVBOARD.mdの該当セクション参照）

> **外部登録**: `C:\00_ProjectDirector`（プロジェクト横断の管理台帳）に APP-2026-005 / TOOL-003 として登録済み（2026-07-30）。「新規デスクトップアプリ開発時に専用keyboard/shortcutを提供する選択肢」としての参照登録で、本体はここから物理移動していない。詳細: `C:\00_ProjectDirector\台帳\アプリ目録.md`・`ToolList.md`

## 目的（変わらない）
PC（Rust Hub）を頭脳にし、スマホ/タブレットのブラウザを「ただのボタンの板」にして、キーボード・Stream Deck等の入力デバイスに変えるアプリ。全フォーマットはコード不要のJSONで定義。完成の定義=各Surface（分割キーボード/Deck/一枚キーボード/トラックボール等）がJSON編集だけで自由に作り替えられ、ユーザーが実機で「使える」と判断すること。

## 実体の場所
- 作業ルート: `C:\00_master\DevApps\APP_KeyDeck01`（存在確認済み、独立gitリポジトリ、`C:\00_master`親repoとは無関係）
- リモート: https://github.com/SHUNSUKE-Ks/APP_KeyDeck01 （main）。**現在ローカルHEADが `origin/main` より2コミット以上先行**（`3a83df3` トラックボールD28、`b6dab96` market/security/legalレポート）。**加えて今回のジェスチャー拡張(T15〜T19)は未コミットのまま作業ツリーに残存**。push・commitともにユーザー未承認
- 真実源: `DEVBOARD.md`（全履歴・決定事項・検証記録）／`CLAUDE.md`（不変条件・凍結領域）／`INDEX.md`（全ファイル索引）

## 現在地（2026-08-29時点）
- 分割キーボード・Deck・iPad一枚キーボード・トラックボール面(D28)は実装済み・コミット済み
- **今回のセッションで実装したが未コミット**: トラックボール面のVer1ジェスチャー拡張（T15〜T19、設計書 `brief/keydeck_trackball_gestures_v0.7.md`）
  - 内容: 1本タップ=左クリック／ダブルタップ=ダブルクリック／長押し=左ボタン押しっぱなし(実OSドラッグ選択と両立)／2本タップ=右クリック／2本上下=スクロール／3本タップ=Esc
  - ユーザー裁定済み: 「Ver1最小構成のみ設計→実装」「長押し=マウス左ボタン押しっぱなし(推奨案採用)」「2本長押し=リングメニューはVer1では見送り」
  - 変更ファイル: `crates/proto-keymap/src/lib.rs`／`crates/proto-hub/src/{surface,protocol,ws,deck}.rs`／`crates/proto-adapter-win/src/lib.rs`（+新規`examples/smoke_mouse_click.rs`）／`static/trackball.html`／`surfaces/trackball.json`／`DEVBOARD.md`。新規: `brief/keydeck_trackball_gestures_v0.7.md`
  - 独立検証済み（Sonnetの自己申告を鵜呑みにせず自分で再実行・再読）: `cargo test --workspace` **95件全pass**（0 failed）。禁止ファイル（`crates/hub-core/`・`keymaps/keymap_default.json`・`decks/`等）への差分ゼロを確認。`static/trackball.html`のCore/View（既存のボール描画・慣性計算）は無改変であることをコード上で確認（`mouse`/`マウス`という語がCore/View内に一切出現しないことも確認）
  - Sonnet自身も合成PointerEvent→実DOM→実WebSocket→実Hubの疎通確認済み（`DEVBOARD.md`該当セクションに記録）
- 未確認: **G-19a（Android実機での指タッチ確認）**。ブラウザ合成イベントでのロジック検証は済んでいるが、実指でのタップ判定(250ms)・長押し判定(500ms)・ダブルタップ猶予(300ms)が体感として妥当かは実機でしか確認できない。URLは既存と同じ（`/trackball?token=...`）なので新しいQR発行は不要、ページのリロードのみで良い
- 気づいた副産物: `prototypes/capture_sequence/capture_sequence.py`という未追跡ファイルが作業ツリーにある。今回のセッションでは作成した記憶がなく、由来未確認（Browser pane検証時の副産物の可能性）。中身を確認してから要否判断すること

## 未回答・判断待ち【最重要】
1. T15〜T19（トラックボールジェスチャー拡張）を**コミットしてよいか**— ユーザー未確認
2. コミット後、**pushしてよいか**（`origin/main`は3a83df3以降に2コミット遅れている）— ユーザー未確認
3. `keydeck-guardian`（大規模変更時の守護レビューエージェント）を今回・前回(トラックボールD28本体)分ともまだ一度も実行していない。実行するか判断待ち
4. このジェスチャー拡張をD28同様の正式なP-003提案書（`brief/proposals/`、裁定つき）として文書化するか、それとも今回の2回のAskUserQuestion回答（上記「ユーザー裁定済み」）で足りるとするか未決定
5. G-19a（実機Android確認）待ち。ユーザーが実機で試した結果次第で微調整が必要になる可能性あり（特に3つの閾値定数）
6. `prototypes/capture_sequence/capture_sequence.py`の扱い（上記「気づいた副産物」参照）
7. （旧項目・継続未決）`brief/mockup/screen_mock_ipad02_v0.1.html`（互い違い配列レイアウト案）採用可否。T9（VIAL型設定編集GUI）着手可否。両方とも進捗なし

## 読む順
1. `INDEX.md` — 全ファイルの地図・読み順
2. `CLAUDE.md` — 全AI必読規則（凍結領域・不変条件6箇条、D28含む）
3. `DEVBOARD.md` — 決定事項ログ・タスク進捗・検証記録の全時系列（**末尾の「トラックボール面 Ver1ジェスチャー拡張 T15〜T19」セクションが最新**）
4. `brief/keydeck_trackball_gestures_v0.7.md` — 今回のジェスチャー拡張の設計書（まだAIが自分では通読していない箇所がある。参照時は全文読むこと）
5. `brief/keydeck_design_v0.4.md`／`brief/keydeck_trackball_design_v0.6.md` — 現行設計書

## 落とし穴
- Hub再起動のたびにtokenが変わる（D8）。「繋がらない」と言われたら最初にHub生死とtoken鮮度を疑う
- CSS Gridの`1fr`は内容の最小幅を下回れない。狭いWebView（QRスキャナ内蔵ブラウザ等）で表示崩れが実際に発生した（修正済み。同種の罠に注意）
- Browser paneのスクリーンショットツールがこのプロジェクトの検証中に頻繁に固着した。`get_page_text`/`javascript_tool`のDOM検証で代替すること
- 英数⇄日本語=ALT+GRAVE chordはMS-IME既定挙動への依存。他環境で効かない可能性は未検証
- クライアント側ジェスチャー閾値（タップ250ms・ダブルタップ猶予300ms・長押し500ms・移動許容10px）は設計書で「Sonnetの裁量に委ねてよい」とされた固定値実装。JSON化されていないので変更時はコード修正が必要
- 作業再開時は`proto-hub.exe`が前回セッションから起動しっぱなしになっていないか確認し、必要なら`start_hub.cmd`で再起動（tokenは再生成される）

## 履歴
- `DEVBOARD.md`（時系列の全検証記録・決定事項ログ、本体）
- `reports/report_keydeck_v0.2_verification.md`（Opus独立検証シートの書式見本）
- `brief/spec_return_log.md`（SR-001裁定済み。今回のT15〜T19は設計書側の事前警告により新規SR起票なしで実装完了）
