# INDEX — APP_KeyDeck01 全ファイル索引（AI用の地図）

迷ったらここ。**読み順: ①CLAUDE.md → ②vision → ③DEVBOARD → ④brain JSON**。その後は目的別に下表へ。

## 0. 最初に読む4点（この順）

| # | ファイル | 何か |
|---|---|---|
| ① | `CLAUDE.md` | 憲法。凍結領域・不変条件6箇条・品質ゲート。**違反する変更は書く前に止まる** |
| ② | `brief/keydeck_vision_and_agents_v1.md` | 北極星。ゴールの言語化・守り/攻めのハーネス・製品化ループ・新Surface追加時の手順 |
| ③ | `DEVBOARD.md` | 現在地。決定事項ログ・タスク進捗・検証記録（時系列の全履歴） |
| ④ | `brief/keydeck_brain_v1.json` | 暗黙知DB。ユーザープロファイル・罠PF1〜8・未決スレッドOT・検証レシピ |

## 1. 設計書（brief/。番号が新しいほど優先。古いものも有効なD番号を含む）

| ファイル | 内容 |
|---|---|
| `brief/keydeck_design_v0.2.md` | 基礎設計。G1〜G7・D1〜D12（レイヤー意味論D3・許可リストD5・ログD9・token D8） |
| `brief/keydeck_design_v0.3.md` | レイヤー別JSON(D13)・ipad面初版(D14〜D17) |
| `brief/keydeck_design_v0.4.md` | Vol1.2。Vol管理D18・記号レイヤーD19・text注入D20・辞書D21・VIAL設定D22・対アプリ拡張D23＋v0.4.1追記(グリッド盤面D24・QRボタンD25・状態ボタンD26・統治D27) |
| `brief/keydeck_format_editing_design_v0.5.md` | フォーマットPC編集（段階A=JSON直編集/B=自動発見+reload/C=GUI）。実装済み |
| `brief/keydeck_backlog_agents_docs_v1.json` | 将来のagent/スキル候補・資料計画（名前と要点のみ） |
| `brief/ref_ipad_keyboard_parts_v1.md` | iPad配置の正（スクショのテキスト化。画像は開かない） |
| `brief/spec_return_log.md` | SR差し戻しログ。曖昧さはここに起票→FABLE裁定 |

## 2. 見た目の正（モック）

| ファイル | 内容 |
|---|---|
| `brief/mockup/screen_mock_v0.4.html` | **現行の正（Vol1.2）**。①キーボード（13列グリッドのgrid指定が盤面データの正）②VIAL型設定 |
| `brief/mockup/screen_mock_v0.3.html` | Vol1.1（凍結・戻し用） |
| `brief/mockup/screen_mock_streamdeck_v0.8.html` | **Stream Deck v2の見た目の正**（①分割画面 上Deck/下キーボード ②Deck単独 ③設定=登録画面）。①②は`static/panel.html`・`static/deck.html`として実装済み。③（登録画面）は未実装＝まだ案 |
| `C:\00_master\MockUp\APP_KeyDeck01\` | 共有アーカイブ（複製＋README） |

## 3. データ（=フォーマットの正。コード不要で編集可）

| 場所 | 内容 |
|---|---|
| `keymaps/keymap_*.json` | 盤面マニフェスト（kind・halves/board・layerFiles）。**ファイルを置くだけで自動発見** |
| `keymaps/layers/*.json` | キー割当（1レイヤー=1ファイル）。ここを編集→`/settings`の再読込で即反映。**レイヤー構成表は`keymaps/layers/README.md`**（ipad01_vol12は 0=基盤 / 1=Fn / 2=記号盤 / 3=日本語モード表示） |
| `keymaps/keymap_default.json` | リセット用原本。**編集禁止** |
| `decks/deck_*.json` | Deckのセットリスト。**ファイルを置くだけで自動発見**（P-005段階Aでkeymapsと同じスキャン方式に）。`render:"grid"`=正方形スロット／`"list"`=縦リスト。`grid.cols×rows`が容量の上限 |
| `schemas/*.schema.json` | keymap/layer/deckの書式検証辞書 |

## 4. コード（Rust workspace。起動=`start_hub.cmd`）

| crate/ファイル | 責務 |
|---|---|
| `crates/hub-core/` | **vendored凍結。1行も変更禁止**（許可リスト・Inspector契約が眠る） |
| `crates/proto-keymap/` | キーマップ型・ロード検証・レイヤー解決エンジン（stdのみ） |
| `crates/proto-adapter-win/` | SendInput（vk表・chord・text=KEYEVENTF_UNICODE）。実発火テストは`examples/smoke_notepad.rs`手動 |
| `crates/proto-hub/` | 基地局。`startup.rs`=スキャン/検証、`ws.rs`=WS処理/reload、`state.rs`=状態、`qr.rs`=QR |
| `static/ipad.html` | 一枚キーボード面（現在の主力） |
| `static/kb.html` `static/deck.html` | 分割キーボード面・Deck面（Deckは2026-09-05に四角スロット化） |
| `static/panel.html` | **分割面（上=Deck／下=キーボード）**。P-003 Ver1-a。WSは`/ws?surface=ipad`1本でプロトコル追加ゼロ |
| `static/layout.html` | **レイアウト面**。`layouts/*.json`の区画割りをそのまま描く1枚。keyboard/deck/trackballの3部品 |
| `crates/proto-hub/src/layout.rs` | 区画割りの型・検証（はみ出し／重なり／参照先の実在） |
| `layouts/layout_*.json` | 画面の区画割り。`?id=`で選ぶ。`ipad_main`=十字キー版／`ipad_trackball`=トラックボール版 |
| **`docs/HUB_MANUAL.md`** | **Hub操作マニュアル。部品配置・スロット数・割当をJSONだけで変える手順（コード不要）** |
| `docs/BOOK_REQUEST_CODEX.md` | 技術書の制作依頼書（CODEX宛）。**§0でTauri前提の崩れを指摘しているので、着手前に必読** |
| `static/settings.html` | 設定（再読込ボタン実装済。VIAL編集GUIはT9でここに実装） |

## 5. 検証・統治

| ファイル | 内容 |
|---|---|
| `reports/report_keydeck_v0.2_verification.md` | Opus独立検証シート（書式の見本） |
| `.claude/agents/keydeck-guardian.md` | 守護agent。変更後の点検（凍結diff・回帰・不変条件） |
| `.claude/agents/keydeck-explorer.md` | 提案agent。`brief/proposals/P-###.md`を起票（実装禁止） |
| `brief/proposals/P-003_streamdeck_v2.md` | Stream Deck v2（Ver1-a実装済み・Ver1-b/1-c裁定待ち） |
| `brief/proposals/P-005_layout_components.md` | **レイアウトと部品の三層化**（section/Component/slot）。段階A実装済み・段階B以降は未着手 |
| `brief/proposals/P-004_ssd_field_deck.md` | SSD外出先専用キーボードView（**別セッション担当**。P-005段階Bと領域が重なる） |
| `README.md` | 起動手順・フォーマットの変え方・トラブルシュート |

## 6. Git

- リポジトリ: `https://github.com/SHUNSUKE-Ks/APP_KeyDeck01`（main）。`C:\00_master`親repoからは独立
- 復元タグ: `format-flex-v0.4`（flex版）／`format-grid-v0.4`（グリッド版）。**削除禁止**
- コミット規律: テスト全pass＋guardian点検後に。pushは原則ユーザー確認

## 7. いまの状態（2026-07-20時点。最新はDEVBOARD参照）

- 実装済み: 分割キーボード（2台同期）・Deck・iPad一枚キーボードVol1.2（記号盤text入力）・QR接続・reload即反映・テスト54件
- 待ち: ユーザー実機テスト → Vol1.2凍結
- 次: T9（VIAL型編集GUI）→ backlogのordering_hint参照
