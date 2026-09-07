# 引き継ぎメモ 2026-08-29

このチャットをリセットする前の作業内容メモ。次セッションで拾えるように残す。

## 1. capture_sequence プロトタイプの状態

対象: [prototypes/capture_sequence/capture_sequence.py](../prototypes/capture_sequence/capture_sequence.py)（178行）

KeyDeckのパネルボタン用Action「action.capture_sequence」の実現可能性検証スクリプト。tkinter + mss + pyautoguiで、

1. GUIで撮影枚数・保存名・保存先・撮影範囲（ドラッグ矩形選択）を設定
2. 「OK」で5秒カウントダウン後、指定枚数だけ「スクショ保存 → `pyautogui.press("right")` でページ送り」を繰り返す
3. 停止手段はPyAutoGUIフェイルセーフ（マウスを画面四隅へ）またはCtrl+Cのみ

**未検証点（次にやるならここから）:**
- このセッションではまだ実行していない。実際に動かして単体動作確認が必要
- `pyautogui.press("right")` は対象ウィンドウがアクティブでないと効かない（フォーカス依存）
- 停止導線が弱い（四隅 or Ctrl+Cのみ）。KeyDeck本番の`action.capture_sequence`として仕上げるなら、KeyDeck本体の停止方式（他Actionと合わせる）を検討する必要あり

進め方の分岐点: (a) 一度実行して動作確認する / (b) KeyDeck本体の`action.capture_sequence`仕様（停止方法・エラー時挙動）に合わせて先に仕上げる、のどちらに進むかは未決定。

## 2. 「Android keyboard」「Hub」に関わるフォルダー調査の結果

ユーザーから「androidkeyboardとHubに関わる全てのフォルダーパス」を聞かれて調査した結果、**KeyDeckという名前のプロジェクトがリポジトリ内に2系統存在する**ことが判明。今後どちらを作業対象にするか要認識合わせ（前回セッションでは未確定のまま終了）。

### ① `DevApps/APP_KeyDeck01/`（現行・本流）
[[keydeck-project]]memoryにある通り、これがユーザーの開発の根本アプリ（独立git、GitHub: SHUNSUKE-Ks/APP_KeyDeck01）。
- Hub本体はRust（`crates/proto-hub`, `crates/hub-core`, `crates/proto-adapter-win`, `crates/proto-keymap`）
- Android側はネイティブアプリではなく、Android ChromeがただのWebページとして接続する構成（`static/kb.html`＝分割キーボード, `static/deck.html`＝Stream Deck面, `static/ipad.html`, `static/trackball.html`, `static/settings.html`）
- 起動: `start_hub.cmd` → `cargo run -p proto-hub`
- README（[README.md:3-4](../README.md)）に「CODEXのAPP_ControlDeckとは別アプリ」と明記あり（`DevApps/APP_ControlDeck/`と混同しないこと）

### ② `CreatorHub/DEV/KeyDeck/`（別系統・Expo RNアプリ＋Pythonハブ）
こちらは①とは別に、このリポジトリ内で独立git管理されている並行実装。
- `app/` = Expo(React Native)アプリ本体（Android端末上で動くネイティブキーボードアプリ）
- `server/` = **Hub本体（Python）**: `server.py`, `key_sender.py`, `macro_engine.py`, `macros.json`, `keymap_cache.json`
- `devstudio/` に管理UIとTODO
- `fix_firewall.bat`, `TROUBLESHOOT.md`, `CLAUDE.md` あり

### その他（参考・KeyDeck本体ではない）
- `MockUp/APP_KeyDeck01/` — 画面モックHTML（v0.3〜v0.4, ipad02）。①の見た目検討用
- `DevApps/APP_ControlDeck/` — CODEX側の別アプリ。`hub-core`等①と似た構成語彙を使うが無関係

**次セッションでの確認事項:** ②(`CreatorHub/DEV/KeyDeck`)が過去の試作で①に統合済み/破棄予定なのか、それとも並行して活きているのか未確認。触る前にユーザーに確認する。

## 3. その他

- CMD貼り付けが時々効かなくなる件について、QuickEdit Mode／Ctrl+V無効化／Admin⇔通常のUIPI／クリップボードチェーン詰まり等の一般的な原因を回答済み（KeyDeck固有の話ではない）
