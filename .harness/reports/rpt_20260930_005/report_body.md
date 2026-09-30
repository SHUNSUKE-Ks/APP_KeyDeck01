# 統合編集画面（studio）段階1 — 新しいトップ `/`

## 完成状態
- Hub のトップ `/` が新しい統合編集画面になった（モック v0.4.1 の見た目で、実物のデータに繋いだもの）
- 端末（iPad Pro・Pixel 6a）を並べて実物の盤面を描き、キーと Deck のタイルの中身・表示名を変えて保存できる。保存後に端末ごとの QR
- 旧画面は消していない（歯車の「旧画面」から開く）。旧レイアウト編集は `/editor`

## 変更内容
- `devices/devices.json`（iPad Pro / Pixel 6a）
- Hub: `GET /api/devices`、`/` → `static/studio.html`、`/editor` → 旧 `editor.html`
- `static/studio.html`（新規）、`static/components.js`（メニューの行き先）

## 検証結果
- `cargo test --workspace` = 174 passed、警告0
- 本物の Hub で実マウス・実キーボード: 選ぶ→表示名→Ctrl+Z→Ctrl+Y→保存→Hub が配り直した構成に反映・QR 表示。部品を入れる→Ctrl+Z。Deck のタイルを選ぶ。試験で変えたファイルは git から戻した
- 未確認: 実機（iPad Pro・Pixel 6a）

## 次の行動
- 利用者が実機で端末ごとの入口 URL / QR を開き、studio で変えて保存→端末に反映されるかを確かめる
- 段階2: 区画の編集（旧 /editor の機能）・キーの位置（/keys）・Deck の色/アイコンを studio に移す。表示レイアウトの Hub 保存（views/）。キーテスター
