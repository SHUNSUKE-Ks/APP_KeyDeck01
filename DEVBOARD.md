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
- [x] T7 スキーマv2（**Sonnet** 2026-07-20）: `crates/proto-keymap/src/lib.rs`にマニフェスト(`KeymapManifest`)＋レイヤーファイル(`LayerFile`)読込を実装。`load_keymap_from_path`はマニフェストを読み、`layerFiles`をマニフェストと同じディレクトリ基準で全読込→結合→従来の検証関数(Layer0必須/L0にtrans禁止/vk辞書/mo・tg参照先)を1回だけ実行。`Keymap`は`kind:"split"|"single"`＋`halves`(split用)/`board`(single用、D24グリッド式`{cols,keys:[{id,row,col,colSpan?,rowSpan?}]}`)に対応。`keymap_default.json`/`keymap_writing01.json`を新形式へ移行（`keymaps/layers/default_layer{0,1,2}.json`・`writing01_layer{0,1,2}.json`に分割。**キー内容は1文字も変えていない**、`diff`相当で確認済み＝labels/actions同一）。`schemas/keymap.schema.json`をv2に更新（`kind`/`board`/`text`action追加）。テスト8件追加（マニフェスト正常結合／レイヤーファイル欠損→LOAD_JSON_SYNTAX／結合後にのみ解決できるmo参照の検証／グリッドboard読込／kind不整合2種→LOAD_SCHEMA_INVALID）
- [x] T8 Vol1.2実装（**Sonnet** 2026-07-20）: `keymaps/keymap_ipad01_vol12.json`をD24グリッド式board(13列・53キー)で新規作成、`keymaps/layers/ipad01_vol12_layer{0,1,2}.json`を作成（layer0=モック①の基盤どおり・layer1=F1-F12プレースホルダ・layer2=記号レイヤー全キーtextアクション）。`Action::Text{string}`をD4に追加（`crates/proto-keymap`）。`crates/proto-adapter-win`にKEYEVENTF_UNICODE送出を実装（`encode_utf16()`でサロゲートペア対応、down/up対）。`proto-hub`: `/ipad`ルート追加、WS接続に`surface=ipad`クエリを追加しSplit(分割/Deck共有state)とは独立の`ipad_layer_state`＋固定`keymapId=ipad01_vol12`で解決（`state::SurfaceKind`）。`static/ipad.html`実装（モックの:rootトークン・.kbgrid値を移植、board配列からgrid-column/grid-row/spanを動的生成、ヘッダにD25 QRボタン・D26状態ボタン・↺↻(H01/H02、board外keyId)）。`static/kb.html`にもD25 QRボタン・D26状態ボタンを追加（分割ヘッダも「kb/ipadヘッダの右隅」という設計に合わせた）。リポジトリ直下に`start_hub.cmd`を新設しREADMEに追記
- [ ] T9 **v0.4で再定義**: VIAL型設定画面（表示のみ）＋QRポップアップ＋formats/export API（Sonnet・今回は着手しない）
- [ ] T10 検証 フェーズA（**Opus**）
- [ ] T11 フェーズB: 編集保存API＋割当実書込（G15）
- [ ] T12 フェーズC: 独自辞書DB＋予測バー（G16）

### v0.4 追記（2026-07-20 FABLE。ユーザーFB反映）

- 設計: `brief/keydeck_design_v0.4.md`（D18〜D23）。モック: `brief/mockup/screen_mock_v0.4.html`＝**Vol1.2**（v0.3モック=Vol1.1として凍結）
- 決定: Vol複製管理(D18) / 記号は専用レイヤーへ・基盤は,.?のみ(D19) / text Action=KEYEVENTF_UNICODEでIME非依存の半角/全角入力(D20) / 辞書DB+予測はフェーズC(D21) / 設定=VIAL型・編集保存APIのみD11禁止を解除・マクロは場所のみ(D22) / 「キーボード+設定アプリを対で増やす」拡張パターン明文化(D23)
- 骨組み改名: `keymaps/keymap_ipad01_vol12.json`（D18命名）
- モック検証: キーボード基盤/記号盤切替をスクショ確認。設定画面(VIAL型)はDOM検証＋ファイル直開きで確認可（Browser paneのスクショ固着のため）
- 2026-07-20 修正: ユーザーFB（Enterキー形状が反映されていない）を受け、メインキーボードをflex行→**13列CSS Grid**に全面書き換え。Enterは`grid-row:2/4`で実際にrow2〜row3を1本のキーとしてまたぐ本物のL字に（row2側はp直後の列、row3側はl+空白1マスの後）。Shift/記号キーはEnterと同じ13列目に縦一列で揃う設計に統一。スクショ・記号盤トグルで再確認済み
- Git: `C:\00_master`から独立させ`git init`＋GitHub `https://github.com/SHUNSUKE-Ks/APP_KeyDeck01` へpush済み（mainブランチ）。以後この案件のコミットは当フォルダ内で独立管理

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
- 2026-07-20 T7/T8（**Sonnet**）: `cargo test --workspace` = **49 passed, 0 failed**（hub-core 7 + proto-keymap 28 + proto-adapter-win 8 + proto-hub 6）。既存39件は削除・弱体化せず、T7に8件・T8に2件（text action系）追加。
  - `cargo run -p proto-hub`実起動＋Node.js組込WebSocketクライアントで手動プロトコル検証（実SendInput/実KEYEVENTF_UNICODE、Windows実機）:
    - `GET /ipad`・`GET /api/qr?target=ipad`・`GET /` = 200、WS無token = 401（既存どおり）
    - `/ws?surface=ipad`のsurface.config: `activeKeymapId="ipad01_vol12"`・`kind="single"`・`board.keys.length=53`・`board.cols=13`・`layers=[0,1,2]`
    - ipad面: 未知keyId→`KEY_UNKNOWN_ID`、記号レイヤーTG(K513)でtoggled=[2]⇄[]、記号レイヤー中のK203(q)がtextアクション"("として無エラーで発火（実際にKEYEVENTF_UNICODEでSendInput成功）、Fn(K101,MO(1))down/upでmomentary=[1]⇄[]、ヘッダ専用keyId H01(board外、chord CTRL+Z)が正常発火
    - **surface独立性の実証**: kb接続(Split)とipad接続を同時に張り、ipad側のK101(MO1)がkb側のlayer.stateに一切現れず、kb側のL41(MO1)もipad側に一切現れないことを確認（`ipad_layer_state`と`layer_state`が完全分離）
    - **既存回帰（G2/G6）実証**: 分割2接続（left/right相当）でL41のMO(1) down/upが両方の接続へ同一の`layer.state`としてブロードキャストされること、Deckの`keymap.reset`(S09)がSplit接続へ`surface.config`(activeKeymapId="default")を再配信すること、`deck.press`の未知slotId→`DECK_UNKNOWN_SLOT`、を確認。検証後はactive_keymap_idをwriting01へ戻して終了（プロセスは検証後に停止）
  - Browser paneで`/ipad?token=…`を表示し`get_page_text`/`read_page`でDOM構造を確認: ヘッダ(↺/↻/タイトル/Layerバッジ/接続中/QR)＋53枚の盤面ボタンが崩れず表示（空プレースホルダー5枚を含む）。screenshotツール自体がタイムアウトしたが、get_page_text/read_page/console(`[KD][T4-1][OK] surface.config received`)で正常表示・正常受信を確認済み（アプリ側の不具合ではなくスクリーンショット取得ツールの問題と判断）

## ipad02 新レイアウト候補（見た目モックのみ・2026-07-20）

ユーザー添付の参考写真（4段コンパクト・数字重ね表示のスマホ風配列）を元に、既存Vol1.2(v0.4)とは**別系統・並存**の新レイアウト案を作成。planner-html-mockupスキル使用。
- モック: `brief/mockup/screen_mock_ipad02_v0.1.html`（共有版 `MockUp/APP_KeyDeck01/`）。**v0.3/v0.4は無変更**
- ユーザー指示3点を反映: ①矢印(⇦⇒)を1段上げてRow3(記号列)右端へ ②空いたRow4右端に「英数⇄日本語」ボタン ③レイヤー切替「記号」キーはRow3左端のまま継続
- 構造差: v0.4は13列直列グリッド（全行が縦に整列）。本モックは**互い違い(スタッガード)配置**＝46サブ列grid（1キー=4サブ列）で行ごとにgrid-column開始位置をずらす技法（row1開始1／row2開始2／row3開始4／row4は全幅で1に戻る）。実物理キーボードのQWERTY標準スタッガーを近似
- 検証: DOM構造で37キー・矢印がRow3・英数⇄日本語がRow4右端・記号キーがRow3左端であることを確認。座標ベースで行の左端がrow1<row2<row3と右へずれ、row4が全幅に戻ることを確認（stagger実証）。記号盤トグル・IME表示切替（A⇄あ）の状態変化を確認、console error無し
  - 補足: `APP_KeyDeck01`は独立repo(親から見てuntracked)のためBrowser paneのfile://タブが静的スナップショット扱いになりwindow.innerWidth等の絶対値が信頼できなかった（新規の環境上の制約。PF2に近い事象として`keydeck_brain_v1.json`への追記候補）。相対座標比較で代替確認した
- 次: ユーザーが正式採用を判断。採用なら `keymap_ipad02.json`（新規board、D24の46サブ列grid拡張）として実装フェーズへ。専用ref md（写真テキスト化）はその時点で起票

## 差し戻し（SR）

`brief/spec_return_log.md` 参照。現在0件。

### 2026-07-20 T7/T8受け入れ（FABLE守護点検 PASS）

- Sonnet実装（コミット `99ad9db` T7 / `2265db1` T8）をFABLEが独立点検: 凍結領域差分0（hub-core/設計書/モック）・default原本のキー定義保全・`cargo test --workspace`=**49 passed**（7+28+8+6）を再実行確認・起動スモーク（/ipad=200, /=200, token無しWS=401, /api/qr?target=ipad=200）
- SR-001裁定済み: ①label"\n"二段規約=承認 ②記号盤アンバー強調はT9送り ③K410-412の全角割当は意図どおり（typoではない）
- 統治導入: `CLAUDE.md`（全AI規則）＋`.claude/agents/keydeck-guardian.md`（守護agent）＋`start_hub.cmd`
- **次: 一枚キーボードの実機テスト**（start_hub.cmd → PCで `/` → ヘッダQRボタン → iPad/AndroidのカメラでQR読取 → /ipad）

### 2026-07-20 実機表示崩れ修正（FABLE。ユーザー実機テストFB反映）

- 原因特定: CSS Gridの`1fr`は既定で「トラック内容の最小幅」を下回れないため、13列の合計最小幅が狭いWebView（QRスキャナー内蔵ブラウザ）の画面幅を超え、右側（asdf行・zxcv行等）が不可視になっていた
- 修正: `.kbgrid`を`repeat(13, minmax(0,1fr))`に、`.key`に`min-width:0`＋省略表示を追加。ヘッダ`h1`も同様に縮小可能化（ヘッダ自体が画面幅を超えていた別要因）。狭幅(480px以下)向けmedia queryでフォント/gapを縮小
- 簡略化: Tab・Ctrl（row2/row5先頭）をcolSpan2→1に変更（ユーザー指示）。空いた列2に空白キー追加(K202/K302/K402/K502)。モック・実装(ipad.html)・盤面JSON(keymap_ipad01_vol12.json)・layer0全て同期
- 検証: `cargo test --workspace`=49件維持。Browser paneで320px/375px幅ともに横はみ出しゼロ（`document.body.scrollWidth === window.innerWidth`）を確認。L字Enter・記号盤は無変更で維持

## 設計書v0.5 F1〜F4（フォーマット編集の仕組み。Sonnet 2026-07-20）

設計: `brief/keydeck_format_editing_design_v0.5.md`。原則「フォーマットの正=JSON、GUI/APIは道具」。

### F1: /ipad実装とモックv0.4①の差分点検・修正

配色トークン値・グリッド配置・ヘッダ構成・状態表現の4観点で`static/ipad.html`とモック①を突き合わせ。

- **色トークン**: `:root`変数は完全一致（`--err`/`#e2555a`/`#ff8a8a`はエラー状態表示のための追加のみ＝モックが想定していないerror状態を実装するための必要な追加であり差替不要）。`--ffd479`(`.zen`全角強調)はSR-001裁定#2により意図的に未実装のまま（T9送り）＝drift扱いしない
- **グリッド配置**: `keymap_ipad01_vol12.json`のboard（row/col/colSpan/rowSpan）は、直近のTab/Ctrl 1列簡略化後のモック13列グリッドと全行一致（Row1〜Row5、Enterのrow2/4またぎ、Space colSpan6等）。修正不要
- **ヘッダ構成**: モック①には無いQRボタン(#qrOpenBtn)・状態ボタン(#statusBtn)が実装に存在するが、これはD25/D26（`brief/keydeck_design_v0.4.md`のv0.4.1追記、モックv0.4作成"後"に追加された設計決定）による正当な追加と確認。ドリフトではない
- **状態表現＝実際のドリフトを検出・修正**: `static/ipad.html`に`body.layer-active .key{background:var(--panel-lit)}`（+`:active`版）というレイヤー有効時の全キー地色変更ルールがあったが、現行モック（v0.4）にはこの表現が存在しない（`--panel-lit`は`:root`に定義だけされ、どのセレクタからも参照されていなかった）。旧モックv0.3(Vol1.1)の`body.layer1 .key{background:var(--panel-lit)}`、および`static/kb.html`（分割面。スコープ外）からの持ち越しと判断し、CSSルール2行とJSのトグル行(`classList.toggle("layer-active", ...)`)を削除。Fn押下時はモックどおりバッジ文字列のみが変わる（記号盤の`.sym`地色変更は元々モックと一致していたため維持）
- 予測変換バー（モックの`.predict`）は末実装のままで正しい: モック自身が「フェーズC/見た目のみ」と注記し、設計書もT12(フェーズC)送りと明記しているため、/ipadでの不在はdriftではなく意図どおりの段階実装

### F2: B1 keymapsディレクトリスキャン

- `crates/proto-hub/src/startup.rs`を新設。`discover_keymap_paths(dir)`が`keymaps/`直下の`keymap_*.json`を名前順に列挙（`layers/`サブディレクトリは対象外）、`load_startup_data(keymaps_dir, deck_path)`が発見した全ファイルをロード→結合検証（ipad固定ID存在確認・deckロード・keymap.switch参照先確認）→成功時のみ`StartupData{keymaps, deck, command_registry}`を返す
- `main.rs`: `DEFAULT_KEYMAP_PATH`/`WRITING01_KEYMAP_PATH`/`IPAD01_VOL12_KEYMAP_PATH`の固定3パス配列と検証ロジックを削除し、`startup::load_startup_data`呼び出しに置換。起動時ログ・エラー時exit(1)の挙動は不変
- ランディングページ(`/`)・QRターゲット一覧は面(surface: kb-left/kb-right/deck/ipad)ベースであり、ロード済みkeymap集合そのものとは独立のため変更不要（新規keymapファイルは`keymap.switch`/将来のkeymap切替経路から自動的に見えるようになる）。設定画面(T9本体=フォーマット一覧UI)は本フェーズのF#に含まれず未着手（`static/settings.html`はF3の再読込ボタンのみ追加、一覧UIはTODO(T9)のまま）
- テスト5件追加（`crates/proto-hub/src/startup.rs`）: 未知の新フォーマットファイルを置くだけでハードコード無しに発見・ロードされること／`layers/`サブディレクトリがスキャン対象外であること／ipad固定ID欠落の拒否／不正JSON混在時の全体拒否（部分適用しない）／keymapsディレクトリが空の場合の拒否

### F3: B2 `/api/reload`＋設定画面の再読込ボタン

- `crates/proto-hub/src/ws.rs`に`POST /api/reload?token=…`を追加（`error.rs`に`RELOAD_INVALID`を追加）。処理: token検証→`startup::load_startup_data`で再読込・検証→現在の`active_keymap_id`が新構成にも存在するか追加確認→**すべて成功した場合のみ**`HubState.keymaps/deck/command_registry`を差替え、`layer_state`/`ipad_layer_state`をリセットしてSplit面・Ipad面**両方**へ`surface.config`を再配信。**1件でも失敗すれば現行構成を一切変更せず**、D9書式の1行tracingログ＋HTTP 422＋`{"code":"RELOAD_INVALID","cause":…,"errors":[…]}`を返す
- `.route_service("/settings", ServeFile::new("static/settings.html"))`を追加。`static/settings.html`はT9(VIAL型エディタ本体)は未着手のまま、F3で要求された「再読込」ボタンのみ実装（`fetch("/api/reload?token=…", {method:"POST"})`→結果をステータス行に表示、D9書式のconsoleログ）。フォーマット一覧・QRモーダル・割当パレットはTODO(T9)のまま先回り実装していない
- 起動時コンソール出力に`settings`のURLを追加

### F4: 品質ゲート・自己点検

- `cargo test --workspace` = **54 passed, 0 failed**（hub-core 7 + proto-keymap 28 + proto-adapter-win 8 + proto-hub 11 ＝ 6→11件、F2/F3分5件追加。既存49件は削除・弱体化なし）
- 実行時検証（`cargo run -p proto-hub`実起動、Windows実機）:
  - 起動ログで`keymaps=3`（directory scan経由でdefault/writing01/ipad01_vol12を発見・ロード。固定配列削除後も件数不変を確認）
  - **正常系**: `POST /api/reload?token=<正しいtoken>` → `200 {"activeKeymapId":"writing01","keymapsLoaded":3}`。同時に張っていた`/ws?surface=ipad`のNode.js WSクライアントが**再接続なしで**新しい`surface.config`（`activeKeymapId:"ipad01_vol12"`）を受信することを確認（既存接続への再配信を実証）
  - **不正系**: `keymaps/keymap_writing01.json`を意図的に壊れたJSON(`{ this is not valid json`)に書き換えた状態で`POST /api/reload` → `422 {"code":"RELOAD_INVALID","cause":"...LOAD_JSON_SYNTAX..."}`。直後に`GET /kb`が引き続き`200`を返すこと（Hubが落ちない・現行構成のまま動作継続）を確認。ファイルを復元して再度reload→`200`成功に戻ることも確認（現状復帰済み、`keymaps/keymap_writing01.json`はGit差分なしを確認）
  - token無し`POST /api/reload` → `401 WS_TOKEN_INVALID`（既存の他APIと同じ拒否経路）を確認
- 自己点検（守護観点。keydeck-guardianエージェントはこのセッションの登録agentには無いため、CLAUDE.mdの点検手順を手動でなぞった）:
  - 凍結領域diff: `git diff --stat crates/hub-core/ brief/ keymaps/keymap_default.json` = 出力なし（0差分）を確認
  - panic経路: 新規/変更コード中の`.unwrap()`は既存踏襲の`state.lock().unwrap()`（Mutex毒化時のみ・入力起因ではない）のみ、`.expect()`は`errors.is_empty()`ガード直後の到達不能パス（既存main.rsと同型）とテストコード内のみ。ランタイム入力起因のpanicパスなし
  - 許可リスト迂回: reload後の`command_registry`は起動時と同一関数(`startup::load_startup_data`)でロード済みJSONの`Key/Chord/Text`アクションのみから再構築（`fire_action`の`is_allowed`チェック経路は無変更）。任意文字列・任意コマンドを許可する経路は追加していない
  - `cargo build --workspace`で警告0件
- README.mdの「フォーマットの変え方」節を段階A（JSON直編集＋Hub再起動）／段階B（ディレクトリスキャン＋`/api/reload`）の手順で更新。動作確認コマンドのテスト件数を54件に更新
- 新規/変更ファイル: `crates/proto-hub/src/startup.rs`(新規)・`main.rs`・`ws.rs`・`error.rs`・`static/ipad.html`（F1差分修正）・`static/settings.html`（再読込ボタン）・`README.md`

## トラックボール面 T10〜T13（**Sonnet** 2026-08-01。設計: `brief/keydeck_trackball_design_v0.6.md`／裁定: `brief/proposals/P-002_trackball_surface.md`）

- T10 `proto-keymap`/`proto-adapter-win`: `Action::MouseMove{dx,dy}`追加（`{"t":"mouse.move",...}`）。`proto_adapter_win::send()`にSendInput+`INPUT_MOUSE`+`MOUSEEVENTF_MOVE`分岐を追加。自動テストは実SendInputを呼ばない（serde往復・vk網羅性等の純粋な部分のみ）。実移動確認は新規`crates/proto-adapter-win/examples/smoke_mouse.rs`（手動実行、cargo run -p proto-adapter-win --example smoke_mouse）
- T11 `proto-hub`: `crates/proto-hub/src/surface.rs`新設。`surfaces/trackball.json`をロード・検証し`SurfaceRegistry`を構築。`binding.t`許可リストはコード内固定（現状`"mouse.move"`のみ）。`startup::load_startup_data`にsurfaces引数を追加し同一エラー集約経路に統合。`surfaces/`ディレクトリ・ファイル不在時は空レジストリで正常起動
- T12 `proto-hub`: `SurfaceKind::Trackball`追加（`from_query`の既存`_ => Split`フォールバックは無変更）。`ClientMessage::SurfaceState`追加（`type:"surface.state"`。spin/activeは受信して読み捨て）。`ws.rs`に`handle_surface_state`を追加（surfaceId解決→有限性→clamp→丸め＋ゼロ移動スキップ→既存`adapter_tx`への発火、の順を固定）。`/trackball`ルート・起動時URL一覧・QRターゲット一覧に追加
- T13 `static/trackball.html`: `（scratchpad）trackball_canvas2d.html`を移植。変更点3点のみ（WS接続を`/ws?token=…&surface=trackball`に統一・`?ws=`手動指定機構は削除／ヘッダにQR(D25)・接続状態(D26)ボタンをipad.html同一CSS値で追加／エラーは`[KD][ERR][コード]`書式でconsole.error）。Core/View層に"mouse"/"マウス"は不出現。ヘッダがflowを占める分、`#ball`/`#lower-half`等の絶対配置基準をvh→`#stage`比の%へ変更（構造上必要な最小限の追随。数値・見た目は不変）
- **SR-002起票**（`brief/spec_return_log.md`）: T10の`Action`追加により`crates/proto-hub/src/deck.rs`のDeckスロット検証matchが非網羅コンパイルエラーになったが、同ファイルは設計書v0.6の「触ってよいファイル」表に未記載だった。1行追加（`MouseMove`をText/None同様の無検証受理枝へ）して継続、SRへ記録・報告
- ブラウザ実機（Windows、Chromeエミュレーション）でのバグ検出＋修正: `trackball.html`初回実装でTDZ（temporal dead zone）例外により球が全く描画されない不具合を発見。`hubSink`のIIFE内`connect()`が自分自身（`hubSink`）を参照する`readoutSink.refresh()`を初期化未完了のまま同期呼び出ししていたのが原因。呼び出しを削除し修正・再検証済み
- `cargo test --workspace` = **72 passed, 0 failed**（hub-core 7 + proto-keymap 29 + proto-adapter-win 8 + proto-hub 28。既存54件は削除・弱体化なし。内訳: proto-keymap +1（MouseMove serde往復）、proto-hub +17（surface.rs 10・startup.rs 2・ws.rs 5）
- `cargo run -p proto-adapter-win --example smoke_mouse` 実行: 4回のMouseMove送出すべて成功（"OK"表示、実カーソルが小さく四辺を描いて動くことを確認）
- `cargo run -p proto-hub`実起動＋Browser paneで実機検証（Windows）:
  - 起動ログ`surfaces=1`（`surfaces/trackball.json`のtb01が読み込まれたことを確認）
  - `/trackball?token=…`表示→ヘッダ「接続中」・QRボタンでモーダル正常表示（`/api/qr?target=trackball`）・パラメータパネル開閉・9項目スライダー正常動作
  - ボールをドラッグ→state読み取り欄が更新・`送信N件`カウント増加・サーバログにエラー無し（=`SURFACE_UNKNOWN_ID`/`SURFACE_STATE_RANGE`/`ADAPTER_SENDINPUT_FAIL`いずれも出ず、Action::MouseMoveが正常にSendInputまで到達したことを確認）
  - G-13d: 412px幅で新規ロード→`document.body.scrollWidth === window.innerWidth`が`true`（412===412）を確認。`window.dispatchEvent(new Event('resize'))`後も維持
  - G-13c: Networkログで`/trackball`・`/api/qr?target=trackball`のみ（全て同一オリジン、外部リクエスト0件）
  - G-13b: `static/trackball.html`のCoreブロック(行367-474)・Viewブロック(行475-605)に"mouse"/"マウス"の出現なし（grep確認）
  - **既存面の回帰確認**: `/`（5カード表示）・`/kb?half=left`・`/deck`・`/ipad`をBrowser paneで表示、いずれも「接続中」・スクリーンショットで崩れなし・console error無し。サーバログでSplit/Ipad/Trackball全surfaceのWS接続・切断がエラー無く記録されたことを確認
- G-13a（Android実機でカーソルが動く）は実機が無いため未検証。**実装完了・実機確認待ち**

### 独立検証（**Opus** 2026-08-01。Sonnetの自己申告を信用せず再実行した結果）

- `cargo test --workspace` 再実行 = **72 passed, 0 failed**（内訳 7+8+28+29）。申告と一致
- 禁止ファイルの差分0を`git diff --stat`で確認: `crates/hub-core/`・`keymaps/`・`decks/`・`static/{kb,deck,ipad,settings}.html` すべて無変更
- D28要件の実装確認: `ALLOWED_BINDING_TYPES: &[&str] = &["mouse.move"]` がコード内固定constであること、`handle_surface_state`の処理順が設計書①〜⑤どおりであることをソースで確認
- D2確認: `static/trackball.html`の外部参照は`/api/qr?target=trackball`（自Hub）のみ。CDN・外部ライブラリ0件
- G-13b再確認: Core/Viewセクション（9116文字）に`mouse`/`マウス`の出現0件
- **実Hub起動＋WS経由のエラー経路検証**（PowerShellの`ClientWebSocket`から直接送信）:
  - 未知surfaceId → `SURFACE_UNKNOWN_ID: unknown surfaceId 'NOPE'`
  - clamp超過(+9999) → `SURFACE_STATE_RANGE: ... exceed clamp 200`
  - clamp超過(-9999) → 同上（負方向も拒否）
  - `dx=0,dy=0` → エラーもAdapterJob発行もなし（設計どおり黙って終了）
  - いずれもHubは落ちず、後続メッセージを継続処理
- **実カーソル移動の方向確認**（3方向すべて正しい）:
  - `dx=+60 x5` → 実測 dx=+1014, dy=+1（X正方向・Y不動）
  - `dx=-60,dy=-60 x5` → 実測 dx=-900, dy=-600（画面左上端でクランプ）
  - Hubの`/api/qr?target=trackball` = HTTP 200・12730バイトのSVG（D25が新面でも機能）

#### 測定上の注意（次に検証するAIへ）

**ブラウザ操作ツール（Browser pane）の呼び出し自体がOSカーソルを動かす。**
WS送信なしの no-op でカーソルが (300,300)→(989,662) へ移動することを確認済み。
カーソル座標の前後比較でマウス移動を検証する場合、**ブラウザを一切介在させないこと**
（PowerShellの`System.Net.WebSockets.ClientWebSocket`から直接送るのが正しい方法）。
この罠により、当初「送信値と実測値が一致しない＝不具合」と誤判定しかけた。

#### 実測: Windowsポインタ加速の影響（不具合ではない。仕様上の性質）

`MOUSEEVENTF_MOVE`はWindows側の「ポインターの精度を高める」(`HKCU:\Control Panel\Mouse\MouseSpeed=1`)の
影響を受け、**1イベントあたりの値が大きいほど加速が強くかかる**。実測値（10回送信の合計）:

| 1回あたりdx | 送信計 | 実移動 | 倍率 |
|---|---|---|---|
| 1 | 10px | 10px | x1.00 |
| 3 | 30px | 43px | x1.43 |
| 5 | 50px | 82px | x1.64 |
| 10 | 100px | 212px | x2.12 |
| 20 | 200px | 644px | x3.22 |

実機の指移動は1イベントあたり数十pxに達しうるため、**`感度`の既定値1.0では速すぎる可能性が高い**。
面の`加速度`パラメータ既定値が1.0（加速なし）なのはこの二重加速を避ける意味で妥当。
実機テスト時は画面右上⚙から`感度`を下げて調整すること（そのための調整UIである）。

### T14 UI状態（settings ⇄ trackball）＋ Vol凍結（**Opus** 2026-08-01）

- **実機テスト成功**（Android／ユーザー確認）。G-13a達成。これを区切りにVol凍結する
- 追加仕様T14（実機テスト後のユーザー要望）: 面に2値のUI状態を持たせ、`settings`のとき
  **パネル枠外タップで`trackball`へ戻る**。「閉じるためのタップ」がそのままカーソルを
  動かすのは操作として不自然なため、captureフェーズで捕まえて閉じるだけで終わらせる
  （`e.stopPropagation()`＋`preventDefault()`。パネル自身と⚙は「外」に含めない）
- 検証（Browser paneで実測。console error 0件）:
  - G-14a パネル内タップ → 閉じない ✓
  - G-14b 枠外タップ → `trackball`へ戻る ✓
  - G-14c その閉じるタップでカーソル発火なし（`stateOut`が不変であることで確認）✓
  - G-14d 閉じた後の通常ドラッグは効く ✓
  - G-14e ⚙で開き直せる ✓
- **Vol凍結**: `static/trackball.html` を `brief/mockup/screen_mock_trackball_v0.6.html` へ
  複製（`diff -q`で同一を確認）。以後この面の見た目・構造の正は凍結版を参照する
- gitタグ `trackball-v0.6` を復元ポイントとして作成（`format-*`と同じ運用。削除・上書き禁止）

### 市場・セキュリティ・法務の外部調査と方針決定（**Opus** 2026-08-01）

- ChatGPT・Geminiに市場調査／セキュリティ法務の2件を依頼し、計4本の回答を取得。
  統合して `reports/report_market_security_legal_v1.md` に保存
- **方針決定: 当面は個人利用に留める。販売は保留**（採算が成立しないため。根拠は報告書§5）
  - ¥1,500の製品・コード署名年3万円で、損益分岐は年20本超。サポート1件で赤字
  - 無料OSS競合（Bitfocus Companion・Macro Deck）が基礎機能を全て埋めている
  - Windows専用のため、クロスプラットフォーム競合と同価格を正当化できない
- **実コード照合の結果、両調査が「絶対に実装してはいけない」と挙げた項目は全て回避済み**
  （任意コマンド実行APIなし・出口は固定許可リスト・トークン128bit CSPRNG＋プロセス毎失効＋
  定数時間比較・書込APIは`keymaps/layers/`限定）。D5/D22/D28の設計判断の結果
- 未対応として実測で確認した項目（報告書§3.2）:
  - **A トークンがURLクエリ**（両調査とも最優先と判定）
  - B HTTP平文（WSS非対応） / **C WS接続のOrigin検証なし**（grepで0件確認）
  - **D `0.0.0.0`にbind**（private IP限定でない） / E text action文字列が平文JSON
  - F 接続時のPC側承認なし / G 接続期限なし / H コード署名なし
  - **個人利用の範囲では実害は小さい。ただしP-001（マンガ喫茶のレンタルPC）は前提が変わる**
- 安価で効くC・Dは着手候補として保留。A・B・F・G・H・EULAは販売を決めた時点でまとめて対応
- 外部AI調査の信頼性メモ（報告書§7）: 2調査に食い違いが実在（Loupedeckの販売終了有無など）。
  **ChatGPTにMemory汚染の証拠**（プロンプトに無い「TRPG」「マスターの」が出現）があり、
  購買層の分析が願望に寄った疑い。次回の調査プロンプト改善方針を報告書§7.3に記録
