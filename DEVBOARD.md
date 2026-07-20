# DEVBOARD — APP_KeyDeck01（Stream Deck面＋2台分割レイヤーキーボード原型）

最終更新: 2026-07-19（FABLE） ／ **CODEXのAPP_ControlDeckとは別アプリ。相互不可侵。**

## 現在地

- 設計凍結: `brief/keydeck_design_v0.2.md`（G1〜G7・D1〜D11・T1〜T6）
- 骨組み作成済み（FABLE）: 全crate・HTML・キーマップ2枚・デッキ1枚・スキーマ2枚。**各ファイル内コメントが実装指示の一部**
- 実装: **T1〜T6完了**。Opus独立検証でV1〜V6・G1〜G7全合格（`reports/report_keydeck_v0.2_verification.md`）。ブロッカー無し。
- 残り: 人手確認2点のみ（実機/メモ帳フォーカスでの物理タイプ・物理ミュート。設計が「手動E2E」と規定する部分）。改善候補F2/F3（軽微・非ブロッカー）は将来対応。
- 2026-07-19 D12追加（振動は見送り・ユーザー指示）: QRコード付きランディングページ`GET /`を追加。`/api/qr?target=kb-left|kb-right|deck`がSVG QRを都度生成（`qrcode`クレート、`svg`機能のみ）。tokenはHub内で完結しクライアントHTML/JSには一切埋め込まない。`cargo test --workspace`= **40 passed**（proto-hub 5→6件、qr.rsのSVG生成テスト追加）。実行時検証: 3ターゲット全てHTTP 200・正しい`image/svg+xml`、不正target→400、`get_page_text`でランディングページの3URL表示を確認。
  - **注記（軽微・意図的なトレードオフ）**: `/`と`/api/qr`はLAN上の誰でもtoken無しで閲覧可能（既存のkb.html/deck.html静的配信と同じ閾値）。`/api/qr`はtoken込みURLをSVGに埋めるため、同一LAN上の第三者がこのエンドポイントを直接叩けばtokenを知り得る。既存のD8簡略化（LAN限定・非インターネット向け）の範囲内の trade-off として許容。

## 技術スタック（凍結）

- Hub: Rust（axum + tokio + serde + tracing）。ポート8770。Tauri不使用（CLI起動）
- 入力: `proto-adapter-win`（Win32 SendInput、vk辞書はD4）
- 端末: Android Chrome（素のHTML+JS。PWA・フレームワーク・ビルドツールなし）
- 再利用: `crates/hub-core`＝検証済crateのvendored凍結コピー（変更禁止）
- データ: `keymaps/`（default=リセット用・writing01=改造用）、`decks/`（セットリスト）、`schemas/`（検証）

## 決定事項ログ

| # | 決定 | 日付 |
|---|---|---|
| D1〜D8 | v0.1裁定の継承（独立WS / 素ブラウザ / VIAL簡約レイヤー / アクション型 / 許可リスト=ロード済JSON集合・位置IDのみ受信 / 押下プロトコル / 汎用SendInput / token） | 2026-07-19 |
| D9 | ログ・例外設計: `[KD][CHK/ERR][コード]`書式、エラーコード12種、panic禁止、エラーはcode+causeをクライアントへ返しブラウザconsoleで読める | 2026-07-19 |
| D10 | キーマップState: default（不変・リセット用）/ writing01（改造用）。各1枚のJSON。切替はDeckアクション、切替時レイヤーリセット | 2026-07-19 |
| D11 | Deckセットリスト: Import=decks/へ配置＋再起動、Export=GET /api/deck/export。スキーマ検証あり。アップロードAPIは設けない | 2026-07-19 |
| — | MIDIは廃止。ノートアプリ連動自動表示はスコープ外（future） | 2026-07-19 |

## 未完タスク（仕様凍結済。実装Sonnet・検証Opus）

- [x] T1 proto-keymap: 型・ロード検証・レイヤー解決＋テスト12件以上
- [x] T2 proto-adapter-win: 汎用SendInput
- [x] T3 proto-hub: axum/WS・状態同期・エラー整形・切替・export
- [x] T4 kb.html / deck.html 実装
- [x] T5 README整備＋G4確認
- [x] T6 検証 V1〜V6・G1〜G7判定（**Opus**）— **全合格**。詳細 `reports/report_keydeck_v0.2_verification.md`

## v0.3 増築（仕様凍結済 2026-07-20。実装Sonnet・検証Opus）

設計: `brief/keydeck_design_v0.3.md` ／ 配置の正: `brief/ref_ipad_keyboard_parts_v1.md`（UpNoteスクショをテキスト化済み・以後画像は開かない）
ユーザー評価: v0.2は「かなりいい感じ・満点」。次はレイアウト修正フェーズ。

- [x] T6.5 見た目モック＋骨組み（**FABLE** 2026-07-20）: `brief/mockup/screen_mock_v0.3.html`（ipad01キーボード＋設定/QRポップアップの2画面。CSS変数=デザイントークン。実装はこれを踏襲）。共有版 `C:\00_master\MockUp\APP_KeyDeck01\`。骨組み: `static/ipad.html`・`static/settings.html`（コメントのみ）・`schemas/layer.schema.json`・`keymaps/keymap_ipad01.json`（マニフェスト雛形）・`keymaps/layers/README.md`。Browser paneでキーボード画面（Layer0/1）をスクショ確認、設定/QRモーダルはDOM検証（4行一覧・320px単一QR・開閉動作OK）
- [ ] T7 スキーマv2: レイヤー別JSON（マニフェスト＋layers/、既存2キーマップ移行）（Sonnet・v0.3のまま有効）
- [ ] T8 **v0.4で再定義**: Vol1.2レイアウト＋記号レイヤー＋text Action（D20 UNICODE注入）＋`/ipad`（Sonnet）
- [ ] T9 **v0.4で再定義**: VIAL型設定画面（表示のみ）＋QRポップアップ＋formats/export API（Sonnet）
- [ ] T10 検証 フェーズA（**Opus**）
- [ ] T11 フェーズB: 編集保存API＋割当実書込（G15）
- [ ] T12 フェーズC: 独自辞書DB＋予測バー（G16）

### v0.4 追記（2026-07-20 FABLE。ユーザーFB反映）

- 設計: `brief/keydeck_design_v0.4.md`（D18〜D23）。モック: `brief/mockup/screen_mock_v0.4.html`＝**Vol1.2**（v0.3モック=Vol1.1として凍結）
- 決定: Vol複製管理(D18) / 記号は専用レイヤーへ・基盤は,.?のみ(D19) / text Action=KEYEVENTF_UNICODEでIME非依存の半角/全角入力(D20) / 辞書DB+予測はフェーズC(D21) / 設定=VIAL型・編集保存APIのみD11禁止を解除・マクロは場所のみ(D22) / 「キーボード+設定アプリを対で増やす」拡張パターン明文化(D23)
- 骨組み改名: `keymaps/keymap_ipad01_vol12.json`（D18命名）
- モック検証: キーボード基盤/記号盤切替をスクショ確認。設定画面(VIAL型)はDOM検証＋ファイル直開きで確認可（Browser paneのスクショ固着のため）

追加裁定: D13（レイヤー別JSON/スキーマv2）・D14（ipad01配置とIME切替=ALT+GRAVE）・D15（設定画面・1画面1QR）・D16（クリップボードboardは保留=ユーザー明記）・D17（keyIdパターン拡張）

## 検証記録

- 2026-07-19 T1: `cargo test -p proto-keymap` — 20 passed, 0 failed（要件12件以上を達成。実ファイル`keymaps/keymap_default.json`・`keymap_writing01.json`のロード成功テスト込み）
- 2026-07-19 T2: `cargo test -p proto-adapter-win` — 7 passed, 0 failed（vk辞書網羅性・Unsupported経路の自動テスト）。実SendInput smokeは`examples/smoke_notepad.rs`を手動実行する設計（自動テストでは危険なため意図的に分離）
- 2026-07-19 T3: `cargo build -p proto-hub` warning無し成功。`cargo run -p proto-hub`実起動＋Node.js組込WebSocketクライアントで手動プロトコル検証:
  - surface.config受信、key.press→実SendInput成功（エラー無し）
  - 2クライアント同時接続でMO(1)のlayer.stateが**両方**へブロードキャストされることを確認（G2の中核: 片方の操作がもう片方の画面にも反映）
  - deck.press(keymap.reset)でsurface.configが両クライアントへ再配信されactiveKeymapIdが切替わることを確認（G6）
  - エラー系4種のうち3種を実接続で確認: 不正JSON→WS_PARSE、未知keyId→KEY_UNKNOWN_ID、未知slotId→DECK_UNKNOWN_SLOT（各1行のerrorフレーム、接続は維持）。token不一致→401はcurlで確認済み。未知vkは起動時拒否としてunit test済み
  - `/api/deck/export`がtoken必須でDeckSetlistのJSONを返すことを確認
- 2026-07-19 T4: `static/kb.html`/`static/deck.html`実装。Browser paneで実機相当（モバイル幅375px）検証:
  - kb.html: surface.configからDeck生成（ハードコード無し）、実キー押下→console.info(T4-2)→SendInput成功
  - **G2実証**: 左タブでMO(1)を押下保持（pointerdown発火・pointerupなし）→**別接続の右タブのLayer表示も即座に"Layer 1"へ変化**（両画面とも実キーマップ切替まで確認）。離すと両方Layer 0に復帰
  - **G6実証**: deck.htmlから「Default戻し」タップ→**kb.html左右両タブのタイトルが(writing01)→(default)へ即時切替**（Deckとkbが別デバイス・別接続でもHub経由で全体同期することを確認）
  - Deck面「消音」タップ→エラー無しでSendInput実行（実ミュート）。Export JSONリンクにtoken付与済み
  - 切断検知: Hubプロセス停止→ドット灰色化・全キーdisabled・「再接続中」表示。Hub再起動（新token発行）→古いtokenでの自動再接続はWS_TOKEN_INVALIDで拒否される仕様（D8: tokenはプロセス寿命）。新URLへ手動navigate後は正常に再接続・再描画
- 2026-07-19 T5: README更新（起動はworkspace root必須の注記、token再生成の注記、手動smoke手順）。**G4実証**: `keymaps/keymap_writing01.json`の一部labelを編集し、**再ビルド無しで既存バイナリを再起動しただけ**でブラウザ表示に反映されることをget_page_textで確認。検証後は元の内容（D10のdefault/writing01同一という不変条件）に復元済み
- 2026-07-19 最終: `cargo test --workspace` = **39 passed, 0 failed**（hub-core 7 + proto-keymap 20 + proto-adapter-win 7 + proto-hub 5）。リグレッション無し

## 差し戻し（SR）

`brief/spec_return_log.md` 参照。現在0件。
