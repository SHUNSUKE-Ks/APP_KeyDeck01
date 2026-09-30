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

## トラックボール面 Ver1ジェスチャー拡張 T15〜T19（**Sonnet** 2026-08-02。設計: `brief/keydeck_trackball_gestures_v0.7.md`）

D28の精神（クライアントは「何に繋がるか」を指定できない／出口はHub側JSONだけが決める）を踏襲し、
discrete（タップ・ダブルタップ・長押し・Esc）は新WS `surface.gesture`、continuous（2本上下スクロール）
は既存`surface.state`を`surfaceId`だけ変えて再利用する2経路構成で実装。

- **T15** `crates/proto-keymap/src/lib.rs`: `Action`に`MouseClick{button}`/`MouseDoubleClick{button}`/
  `MouseButton{button,down}`/`MouseScroll{dy}`の4variantと`MouseButtonKind{Left,Right}`を追加。
  exhaustive matchを3箇所とも最初から更新（`validate_merged()`・`resolve()`・
  **`crates/proto-hub/src/deck.rs`**——前回T10でここが漏れてSR-002になった箇所を、設計書の事前警告どおり
  最初から直したため今回SR無し）。serde往復テスト4件追加（G-15b）
- **T16** `crates/proto-adapter-win/src/lib.rs`: `send()`に4アーム追加。
  `send_mouse_click`(down→up1組)・`send_mouse_double_click`(click相当を2回)・
  `send_mouse_button`(down/up単発、対を作らない)・`send_mouse_scroll`(`MOUSEEVENTF_WHEEL`、
  `mouseData=-(dy*SCROLL_UNIT)`、`SCROLL_UNIT=8`は仮置き)。Win32側は
  `MOUSEEVENTF_LEFTDOWN/LEFTUP/RIGHTDOWN/RIGHTUP/WHEEL`を追加インポート。非Windowsダミー4関数＋
  自動テスト4件（G-16a、`#[cfg(not(windows))]`）。手動smoke `examples/smoke_mouse_click.rs`を新規追加
  （自動テストからは実SendInputを呼ばない既存方針を継続）
- **T17** `crates/proto-hub/src/surface.rs`: `ClickButton{Left,Right}`・`GestureAction{Click,DoubleClick,
  ButtonHold,Key}`型と`SurfaceDef.gestures: BTreeMap<String,GestureAction>`を追加。新規エラーコード
  `SURFACE_GESTURE_UNKNOWN_ID`・`SURFACE_GESTURE_EDGE_REQUIRED`・`LOAD_SURFACE_GESTURE_INVALID`。
  `ALLOWED_BINDING_TYPES`に`"mouse.scroll"`を追加（`["mouse.move"]`→`["mouse.move","mouse.scroll"]`）。
  テスト11件追加（G-17a: 未知t/未知button/vk欠落・不正・重複gestureId、G-17b: mouse.scroll許可）
- **T18** `crates/proto-hub/src/protocol.rs`に`ClientMessage::SurfaceGesture{surfaceId,gestureId,edge?}`
  を追加。`crates/proto-hub/src/ws.rs`に`handle_surface_gesture`を新設
  （surfaceId解決→gestureId解決→`GestureAction`+`edge`から`Action`組み立て→既存`adapter_tx`へ発火、
  新しい発火経路は作らず・command_registry許可リストも通さない設計書どおり）。
  `handle_surface_state`のbinding解決matchに`"mouse.scroll" => Action::MouseScroll{dy}`を追加。
  テスト13件追加（G-18a: 未知surfaceId/未知gestureId/tap1/dtap1/tap2/tap3/hold1 down・up・edge省略、
  G-18b: tb01-scrollでMouseScroll発火）
- **T19** `surfaces/trackball.json`を更新（§2.2の例をそのまま反映。`tb01`に`gestures`5件＋
  `tb01-scroll`面を追加）。`static/trackball.html`に新モジュール`Gesture`を追加
  （pointerIdごとの`Map`管理。1本タップ=tap1・ダブルタップ=dtap1（300ms内の2回目で確定）・
  長押し=hold1（500ms経過でedge:"down"、finger up/cancelでedge:"up"）・2本タップ=tap2・
  2本上下=`tb01-scroll`へ継続送信・3本タップ=tap3）。**Core/View（①②）は無変更**
  （grep確認: `mouse`/`マウス`の出現なし）。2本目が乗った瞬間、進行中のCore drag（1本ドラッグ）は
  打ち切る。既存の`hubSink`に`sendGesture`/`sendScrollDelta`を追加（同一WebSocketインスタンスを共有、
  新しい接続は作らない）

### 自己解決した判断点（SR起票なし。設計書の裁量範囲内）

- **`gestures`マップの`gestureId`重複検出**: 標準の`serde_json`は`BTreeMap<String,T>`へJSONオブジェクトを
  デシリアライズする際、同名キーが複数回現れても後勝ちで黙って上書きし重複を検出しない。
  `LOAD_SURFACE_GESTURE_INVALID`で確実に拒否するため、`GesturesWire`という薄いラッパー型を新設し
  `Deserialize`を自前実装（`visit_map`内で挿入時に既存キーへの衝突を検出してエラーにする）。
  このカスタムエラーは`serde_json::from_str::<SurfaceFileRoot>`全体の失敗として返ってくるため、
  メッセージ文字列に`"duplicate gestureId"`を含むかどうかで`LOAD_SURFACE_SCHEMA_INVALID`と
  `LOAD_SURFACE_GESTURE_INVALID`を判別してエラーコードを割り当てている
  （`crates/proto-hub/src/surface.rs`の`load_surface_registry_str`冒頭）
- **クライアント側の閾値**（`T_TAP_MAX=250ms`/`T_DTAP_MAX=300ms`/`T_HOLD=500ms`/`MOVE_THRESHOLD=10px`）は
  設計書§6が明記する「Sonnetの裁量に委ねてよい範囲」どおり仮決め。JSON化はせずコード内固定値のまま
  （設計書の指示どおり）

### 検証記録

- `cargo test --workspace` = **95 passed, 0 failed**（hub-core 7 + proto-adapter-win 8 + proto-hub 47 +
  proto-keymap 33。既存72件から削除・弱体化なし。内訳: proto-keymap +4（MouseClick/DoubleClick/
  Button/Scrollのserde往復）、proto-hub +19（surface.rs 10→19 = +9・ws.rs 5→15 = +10。
  deck.rs/qr.rs/startup.rsは変更なしで既存件数のまま）
- `cargo build --workspace` / `cargo build -p proto-adapter-win --examples`: warning無し（新規追加分）で成功
- Node.js (`new Function(...)`) で`static/trackball.html`のインラインスクリプト全体を構文チェック: エラー無し
- grep確認: `static/trackball.html`のCore/Viewブロックに`mouse`/`マウス`の出現0件（G-13b継続）
- `cargo run -p proto-hub`実起動（`surfaces=2`のログでtb01・tb01-scroll両方の読込を確認）＋Browser paneで
  `/trackball?token=…`を表示し、Node実装ではなくブラウザのjavascript_tool（デバッグ用途、ソース非改変）
  から**実際のDOM・実際のWebSocket接続**を通して合成`PointerEvent`を発火させ、送出されたWSフレームを
  検証（実SendInput・実Hubを介した本物のエンドツーエンド確認。合成イベントは
  `setPointerCapture`が`InvalidPointerId`で例外を投げるため、テスト実行中のみ
  `HTMLElement.prototype.setPointerCapture/releasePointerCapture`を一時的に無害化した。
  ソースファイルは無変更）:
  - 1本タップ → `{"type":"surface.gesture","surfaceId":"tb01","gestureId":"tap1"}`
  - 1本ダブルタップ（300ms以内の2回目）→ `dtap1`が1件だけ送信（途中のtap1は送られない＝
    保留タイマーの取消しが機能）
  - 長押し（650ms経過）→ `hold1`+`edge:"down"`、離した瞬間 → `hold1`+`edge:"up"`
  - 2本タップ → `tap2`
  - 3本タップ → `tap3`
  - 2本を同時に下へ移動 → `surfaceId:"tb01-scroll"`の`surface.state`が移動のたび継続送信
    （`delta.dy`が正値、`dx`は常に0固定。125Hz相当のスロットリングも機能）
  - **既存1本ドラッグの回帰確認**: 1本指のpointerdown→move×5→upで、従来どおり`surfaceId:"tb01"`の
    `surface.state`が継続送信されることを確認（Gesture導入後もCore/View経路が無傷であることの実証）
  - Hubサーバーログにエラー0件（`SURFACE_GESTURE_UNKNOWN_ID`/`SURFACE_GESTURE_EDGE_REQUIRED`/
    `SURFACE_STATE_RANGE`/`ADAPTER_SENDINPUT_FAIL`いずれも出ず、実際のSendInputまで
    到達したことを確認）。**注意**: この検証は実機Windows上で実際にクリック・ダブルクリック・
    右クリック・Esc・ホイールを実際に送出している（実カーソル位置に対して実際に発火する。
    T10検証時と同じ既知の性質）
  - G-19a（実機Androidでの指タッチ確認）は実機が無いため今回も未検証（v0.6のG-13aと同じ扱い。
    上記のPointerEvent検証で「Hub側ロジック・クライアントJSロジックの両方が正しく動く」ことは
    確認済みだが、実指でのタップ/長押しの体感（閾値の妥当性等）はユーザーの実機確認が必要）
- SR起票: **なし**（設計書に明記の無い判断は上記「自己解決した判断点」の範囲に収まった）

---

## Stream Deck v2（アクション拡張＋分割画面）— P-003 起票・見た目モック作成

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **裁定待ち。実装は未着手**

### 発端（ユーザー要望・原文の要点）

1. Elgato Stream Deck ソフトの登録用アクション一覧（システム8種＋マルチアクション4種、
   折りたたみの Stream Deck／サウンドボード）と同等の機能を作りたい
2. **画面分割**を可能にして、下をキーボード・上をStream Deckにしたい
3. 渡したリストは**登録用（設定画面）**のもの。**表示側は四角いスロット**にする

### 今回やったこと（コード変更ゼロ）

- 新規 `brief/mockup/screen_mock_streamdeck_v0.8.html` — 見た目の正（3画面）
  - ① 分割画面: 上=Deck（四角スロット5×2）／下=キーボード（`screen_mock_v0.4.html` の13列配置を転記）
    ＋比率3プリセット（デッキ大／半々／キーボード大）
  - ② Deck単独: 四角スロット5×3＋ページドット
  - ③ 設定（登録画面）: 中央=スロット盤（クリックで選択）／右サイドバー=アクション一覧
    （ユーザー提示リストをそのまま構造化）／下=プロパティ欄＋生成されるJSONのプレビュー
  - 四角スロットは `aspect-ratio:1 / 1`＋`width:100%`。D2どおり素のHTML+JSのみ（CDN・外部依存ゼロ）
- 新規 `brief/proposals/P-003_streamdeck_v2.md` — 提案書（裁定欄つき）
  - アクション対応表（ユーザーのリスト16項目 → JSONの `t`）。うち**5項目は既存アクションで足りる**
  - 中核の設計判断: **Deck JSONに生パス・生コマンドを書かない**（`apps/apps.json` 登録簿に
    id で登録し、Deckは id だけ持つ。D28の「出口はHub側のJSONだけが決める」を1段進めた形）
  - **CLAUDE.md 不変条件6との関係を §7 に明記**（後述）
  - 受け入れ基準 G-a〜G-i、段階分割 Ver1-a／1-b／1-c

### 不変条件との照合（実装前の自己点検）

- **不変条件1（クライアントは位置IDのみ）: 維持**。クライアントが送るのは今までどおり
  `{"type":"deck.press","slotId":"…"}` だけ。**新しいWS APIは1本も足さない**
- **不変条件2（D3レイヤー意味論）: 無関係**。Deck発火は `resolve()` を通らない
- **不変条件4（D2）: 維持**。モックは素のHTML+JS、外部依存ゼロ
- **不変条件6: 🚩 要裁定**。`app.launch` / `shell.open` / `app.close` は条文の文言
  （＝任意〜**API**）には触れないが精神には触れる。P-003 §7 に、採用を推す根拠
  （`WIN+R`＋`text`＋`ENTER` の3スロットで今でも実質同じことができる＝リスクの実質的増分は小さい）と、
  条件とする防御A〜G（登録簿方式／シェル非経由／ロード時の実在・拡張子検証／URLスキーム許可リスト／
  `app.close`はactive限定／D9実行ログ／多重発火抑制）を記載した
- **G5（決定性）: 🚩 要裁定**。`random`（ランダムアクション）が唯一の例外になる。
  Deckスロット限定（`layers/*.json` には置けない）とする案を P-003 §8-1 に記載

### 検証記録

- Browser pane（`python -m http.server` でモックのみ配信）で3画面すべてを表示確認
  （1280×900 デスクトップ／820×1180 iPad相当の2サイズ）
  - タブ切替・分割比率プリセット・スロット選択→アクション割当→プロパティ生成→JSON出力の
    一連が動作。`read_console_messages` でJSエラー0件
  - 作成中に自分で見つけて直した不具合3件（いずれもモック内で修正済み）:
    (a) `.slot` に `width:100%` が無く `aspect-ratio` がつぶれてスロットが極小になっていた
    (b) `.screen{display:none}`（クラス）が `#scr-split{display:flex}`（ID）に特異性で負け、
        タブを切り替えても①が消えなかった → `.screen:not(.on){display:none !important}` で解決
    (c) `body{height:100%}` のため設定画面で縦にスクロールすると地色が塗られず黒く抜けた
        → `min-height:100%` に変更
- `cargo test --workspace`: **未実行**（Rust側の変更が1行も無いため。実装着手時に実施する）
- 凍結領域への差分: **ゼロ**（今回の新規ファイルは `brief/mockup/` と `brief/proposals/` の2点のみ。
  `crates/` `keymaps/` `decks/` `static/` `schemas/` は無変更）
- SR起票: なし（曖昧さは P-003 §12 の裁定チェックリストに集約した）

---

## Stream Deck v2 — P-003 裁定 ＋ Ver1-a（分割面／四角スロット化）実装

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **Ver1-a 完了・未コミット**

### ユーザー裁定（P-003 §12）

- **§7 採用**: `app.launch` / `shell.open` / `app.close` を**防御A〜G条件付きで採用**してよい
  （A登録簿方式／Bシェル非経由／Cロード時の実在・拡張子検証／DURLスキーム許可リスト／
  E`app.close`はactive限定／FD9実行ログ／G多重発火抑制）。**A〜Gは実装の必須条件**
- **§11 採用**: 段階分割（Ver1-a／1-b／1-c）で進め、**Ver1-a から先に着手**する
- 未裁定のまま残したもの: §5細部（`web.open` の url 直書きという非対称）／§8-1（`random` の採否）／
  §8-4（登録画面をT9と同居させるか）／§8-6（`icon` に絵文字を認めるか）

### Ver1-a でやったこと

| ファイル | 内容 |
|---|---|
| `static/panel.html`（新規） | 分割面。上=Deck（四角スロット）／下=キーボード（13列グリッド）。比率3プリセット |
| `static/deck.html` | 四角スロット化（`aspect-ratio:1/1`＋`width:100%`）。`gridTemplateColumns` を `minmax(0,1fr)` に。**JSロジックは無変更**（CSSと1行のみ） |
| `crates/proto-hub/src/ws.rs` | `/panel` の `route_service` 追加＋ランディングページのtarget一覧に `panel` を追加 |
| `crates/proto-hub/src/state.rs` | `connection_url` に `"panel" => "/panel"` を追加（`/api/qr?target=panel` 用） |
| `crates/proto-hub/src/main.rs` | 起動時バナーに panel のURLを1行追加 |

### Ver1-aで確定した設計判断（Ver1-b以降も踏襲）

1. **分割面はWS 1本・`/ws?surface=ipad` で接続する。プロトコル追加ゼロ。**
   `surface.config` が元から `keymap` と `deck` を同一メッセージで配っているため、
   新しい `SurfaceKind::Panel` は**作らなかった**（作ると broadcast 経路を全て触ることになり、
   得るものが無い）。キーボードはipad面と同じ `ipad01_vol12` 固定＋専用 `ipad_layer_state`（T8）、
   Deck発火は surface非依存の `handle_deck_press` をそのまま使う
2. **四角スロットの大きさは「上ペインの高さ」から逆算する**（`fitDeckGrid()`）。
   四角形は幅で大きさが決まるため、5列×3行のDeckをそのまま置くと上ペインからはみ出す。
   1マスの**下限は44px**（タップ目標の下限）。下限に達したら縮めず上ペインをスクロールさせる
3. **ページドットは `pages` が2枚以上のときだけ出す**。表示専用でHubへは何も送らない
   （`deck.page` アクション自体はVer1-bの範囲）

### 検証記録

- `cargo test --workspace` = **99 passed, 0 failed**（hub-core 7 + proto-adapter-win 8 +
  proto-hub 51 + proto-keymap 33）。前回95件から**+4**（削除・弱体化ゼロ）:
  - `state::tests::connection_url_resolves_panel_target`
  - `state::tests::connection_url_resolves_every_landing_page_target`（targetの足し忘れ防止）
  - `state::tests::connection_url_rejects_unknown_target`
  - `ws::tests::every_served_static_file_exists`（`ServeFile` の相対パスは綴り違いを
    コンパイルで捕まえられず、実機でQRを読んだ瞬間に404で初めて分かるため固定した）
- `cargo build --workspace`: 新規warningなし（既存の `surface.rs: is_empty is never used` のみ）
- **実Hub・実WebSocketでの疎通確認**（`cargo run -p proto-hub` を起動し、Browser paneから
  `/panel?token=…` を実際に開いた。モックではなく本番経路）:
  - `surface.config` 1通で `keymap`(ipad01_vol12・3レイヤー) と `deck`(default・15スロット) の
    両方が届き、上下ペインが同時に描画されることを確認（コンソール `[KD][T20-1][OK]`）
  - **G-e相当の回帰確認**: 「記号」キー（`tg` layer2）を実押下 → レイヤーバッジが「記号」に変わり
    キーボードが記号盤に再描画され、**同じWS上でDeckは無傷**（15スロット・先頭ラベル「消音」が不変）。
    もう一度押して Layer 0 に戻ることも確認。`tg` は `LayerChanged` で終わるためSendInputは発生しない
  - **G-f/G-g相当**: 比率3プリセット（デッキ大／半々／キーボード大）を切り替え、
    デスクトップ1024×800・iPad 820×1180・Android縦412×915・Android横915×412 の4サイズで
    `scrollWidth-clientWidth` / `scrollHeight-clientHeight` を実測し、**横はみ出しゼロ**を確認。
    スロットは全サイズで正方形（幅=高さ、実測差1px未満）
- 実装中に自分で見つけて直した不具合4件（すべて修正済み・再検証済み）:
  1. `deckPaneEl` の宣言漏れ（`ReferenceError`）。page側のグローバルエラーハンドラが
     D9書式で拾ったため発見できた（=D9の仕組みが機能した実例）
  2. `.pagedots{display:flex}` が UAスタイルシートの `[hidden]{display:none}` に勝ってしまい、
     ページが1枚でも空の箱がflex gapぶんの高さを食っていた（上ペインが4px溢れる）
     → `.pagedots[hidden]{display:none}` を明示
  3. 横向きスマホ（915×412）の「半々」で1マスが**実測35px**まで縮み、タップ目標として
     小さすぎた → `MIN_CELL = 44px` の下限を導入。下限到達後は上ペインをスクロールさせる
  4. 狭い画面でヘッダのタイトルが幅0まで潰れ「空白が空いているだけ」に見えた
     → 560px以下ではタイトルを畳む（レイヤーバッジ・接続状態・QRを優先）
  - なお検証中、ブラウザが `static/panel.html` の**古い版をキャッシュから返す**事象があり、
    修正が効いていないように見えた。以後この面の検証ではクエリを足して読み直すこと（罠として記録）
- 凍結領域への差分: **ゼロ**（`crates/hub-core/` / `keymaps/keymap_default.json` / `decks/` /
  `brief/` の既存設計書・既存モック いずれも無変更。`git status --porcelain` で確認）
- **未確認**: 実機Android/iPadでの指タッチ確認（G-19a と同種）。ブラウザのビューポート
  エミュレーションでの寸法検証は済んでいるが、44px下限が実指で妥当かは実機でしか分からない。
  URLは新規（`/panel?token=…`）なので、**実機で試すにはランディングページのQR（5枚目）が必要**
- SR起票: なし

---

## T21 — 実機で見つかった不具合の修正: 「英数⇄日本語で画面表示が切り替わらない」

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **修正完了・未コミット**
発見: ユーザーのiPad実機確認（P-003 Ver1-a の実機テスト中）

### 報告内容

> 表示keyboard日本語英語切り替え＝キーボードのかな変換は切り替わっていても、
> iPadや画面に表示されたキーボードは切り替わらない

### 原因

`keymaps/layers/ipad01_vol12_layer0.json` の K511 が **ただの `chord`** だった。

```json
"K511": { "label": "英数⇄日本語", "action": { "t": "chord", "keys": ["ALT", "GRAVE"] } }
```

`chord` は PC へ ALT+GRAVE を撃つだけで **Hub側に何の状態も残らない**。
Hubが `layer.state` を配信するのはレイヤー状態が変わったときだけなので、
配信すべきものが無く、クライアントは再描画のきっかけを得られない。
（対して「記号」K513 は `tg` なので状態が残り、正しく切り替わっていた ── この差が症状の正体）

### ユーザー裁定（2026-09-05）

- **日本語モードで変える表示**: 「モード表示＋実際に出力が変わるキーだけ」。
  この盤面は**ローマ字入力**前提（全キーがQWERTYの`key`アクション）であり、日本語モードでも
  a/k/s… は物理的に同じ英字を送っている。「日本語だから」とかな配列を出すのは
  **押しても出ないかなを表示する嘘**になるため採らない
- **ずれ対策**: レイヤーバッジ長押しで、PCへ何も送らずに表示だけ反転できるようにする

### D29（新しい決定。DEVBOARDに記録。設計書への反映が要るならSR経由）

**`tg.fire` — レイヤー切替とアクション発火を1打鍵で同時に行うアクション型。**

`tg` は状態を変えるが発火しない。`chord` は発火するが状態を変えない。
「OS側のモードを変えると同時に画面表示も変えたい」キーには、その両方が要る。

```json
{ "t": "tg.fire", "layer": 3, "fire": { "t": "chord", "keys": ["ALT", "GRAVE"] } }
```

- `fire` に置けるのは**葉アクションのみ**（key/chord/text）。mo/tg/tg.fire/keymap.* の入れ子は
  ロード時に `LOAD_SCHEMA_INVALID` で拒否（再帰と、1打鍵での状態二重変更を防ぐ）
- 参照先レイヤー不在は `LOAD_LAYER_REF_INVALID`、`fire` 内のvkも辞書検証の対象（`LOAD_VK_UNKNOWN`）
- **Deck面には置けない**（mo/tg/transと同じ扱い。Deckにレイヤーの概念が無いため）
- **不変条件1に抵触しない**: クライアントが送るのは今までどおり keyId のみ。WS APIの追加ゼロ
- **G5（決定性）を壊さない**: トグル則は `tg` と同一で、乱数・時刻を使わない（テストで固定）

### 変更内容

| ファイル | 内容 |
|---|---|
| `crates/proto-keymap/src/lib.rs` | `Action::TgFire { layer, fire }`／`Resolved::FireAndLayerChanged(Action)`／resolveのarm／`validate_merged`の検証 |
| `crates/proto-hub/src/ws.rs` | `FireAndLayerChanged` を処理（**先に`layer.state`配信 → 後に発火**。発火はadapter往復のawaitを挟むため画面が先の方が体感が速く、発火が失敗しても画面とHub状態の整合は保たれる） |
| `crates/proto-hub/src/state.rs` | `canonical_command_id` が `tg.fire` の中の `fire` まで潜るよう修正（**後述の落とし穴**） |
| `crates/proto-hub/src/deck.rs` | `tg.fire` をDeckで拒否（mo/tg/transと同じ列に追加） |
| `schemas/keymap.schema.json` | `tg.fire` を `$defs/action` に追加。`deck.schema.json` にも「Deckには置けない」旨を明記 |
| `keymaps/layers/ipad01_vol12_layer3.json`（新規） | 日本語モード表示レイヤー。K213(Enter/確定)・K504(Space/変換)・K511(⇄英数/日本語) の3キーのみ |
| `keymaps/layers/ipad01_vol12_layer0.json` | K511を`tg.fire`へ。**H03**（board外keyId、`tg` layer3のみ＝PCへ何も送らない）を追加 |
| `keymaps/keymap_ipad01_vol12.json` | `layerFiles` に layer3 を追加 |
| `static/ipad.html` / `static/panel.html` | バッジを「日本語／英数」表示に・`body.jp` の地色・2行ラベルの主従入れ替え・バッジ長押し(H03) |
| `keymaps/layers/README.md` | ipad01_vol12のレイヤー構成表と、layer3編集時の注意を追記 |

### なぜ layer3 に「、。？」を入れなかったか（重要・触る前に読むこと）

有効レイヤーは**番号の大きい方が勝つ（D3）**ため、layer3はlayer2（記号盤）より強い。
layer2が定義しているキー（K102-111/K203-212/K303-311/**K403-412**）をlayer3に足すと、
**記号盤を出している間まで日本語表示が勝ってしまい記号盤が壊れる。**
そのため K410/K411/K412（、。？）の強調は JSON ではなく**クライアント側のCSS**で行った:

```css
body.jp:not(.sym) .key.dual       { font-size: 10px; color: var(--ink-muted); }
body.jp:not(.sym) .key.dual small { font-size: 17px; color: var(--ink); }
```

`.dual` は「ラベルが2行である」ことを`renderLabel`が付ける印。`:not(.sym)` で記号盤中は除外する。

### 落とし穴（今回踏んで直したもの。同種の追加をするとき必ず読む）

**D5の起動時許可リストは、入れ子になったアクションを見つけられない。**

`canonical_command_id` は top-level のアクションしか見ないため、`tg.fire` の中に入れた
`chord:ALT+GRAVE` が許可リストに載らず、実行時に

```
ERROR ... action resolved but is absent from the startup allow-list: chord:ALT+GRAVE
```

で**発火だけが拒否された**。症状は「画面表示は切り替わるのにPCのIMEが切り替わらない」＝
**元の不具合の左右反転**で、しかもクライアント側は無言（Hubのログにしか出ない）。
今後 `multi` / `logic`（P-003 Ver1-b/1-c）で入れ子アクションを増やすときは、
**許可リストの構築を同時に更新すること**。回帰テストを2本置いた（後述）。

### 検証記録

- `cargo test --workspace` = **108 passed, 0 failed**（hub-core 7 + proto-adapter-win 8 +
  proto-hub 53 + proto-keymap 40）。Ver1-a時点の99件から**+9**（削除・弱体化ゼロ）:
  - proto-keymap +7: `t21_tg_fire_changes_layer_and_fires_in_one_press`（核心＝1打鍵で両方起きる）／
    `t21_tg_fire_toggles_back_and_still_fires`／`t21_tg_fire_ignores_key_up`／
    `t21_tg_fire_is_deterministic`（G5）／`t21_tg_fire_referencing_missing_layer_is_rejected_at_load`／
    `t21_tg_fire_rejects_nested_layer_action`／`t21_tg_fire_rejects_unknown_vk_inside_fire`
  - proto-hub +2（**許可リスト落とし穴の回帰テスト**）:
    `state::canonical_command_id_descends_into_tg_fire`（単体）と
    `startup::real_data_startup_allows_the_ime_toggle_chord_behind_tg_fire`
    （**リポジトリの実データで起動して `chord:ALT+GRAVE` が許可リストにあることを確認**。
    単体だけだと「関数は正しいが実データでは載っていない」を取り逃す）
- **実Hub・実SendInputでの疎通確認**（`cargo run -p proto-hub` を再起動して実施）:
  - `/ipad` でK511を押下 → バッジ 英数→**日本語**、`body.jp` 付与、
    K511「⇄英数/日本語」・K504「Space/変換」・K213「Enter/確定」に変化、
    K410は本文10px・`<small>`17px（＝「、」が主）に入れ替わることを`getComputedStyle`で実測
  - もう一度押下 → すべて元通り。**Hubのエラーログ0件**＝ALT+GRAVEが実際にOSへ送出された
    （許可リスト修正前は同じ操作で `INTERNAL` エラーが出ていた。前後で対比を取っている）
  - **記号盤との併用**: 日本語モード中に「記号」を押すと `body="jp sym"`、バッジ「記号」、
    K410は「：」（layer2が勝つ）、フォント16px（主従入れ替えは`:not(.sym)`で無効）を確認。
    記号盤を消すと日本語表示に戻る＝**layer2/layer3の衝突が設計どおり回避されている**
  - **バッジ長押し(H03)**: 750ms押して離すと バッジ 日本語→英数・`body.jp`解除、
    フッタに「表示だけ切り替えました（PCへは何も送っていません）」。
    Hub側は `tg` のみでadapterへは何も流れないことをログで確認
  - `/panel`（分割面）でも同一挙動。**切替中もDeckは無傷**（15スロット・先頭「消音」が不変）
  - 押下は`computer`ツールに「押しっぱなし」操作が無いため、長押しのみT18と同じ
    合成PointerEvent（実DOM・実WebSocket・実Hub・実SendInput経由）で実施。通常押下は実クリック
- 凍結領域への差分: **ゼロ**（`crates/hub-core/` / `keymaps/keymap_default.json` / `decks/` /
  `brief/` の既存設計書 いずれも無変更）
- **未確認**: 実機iPadでの再確認（今回の修正が実指で期待どおりかはユーザー確認待ち）。
  URLは変わらないので**ページの再読込だけでよい**（ただし後述のキャッシュに注意）
- SR起票: なし（D29としてここに記録）

---

## T22 — ipad01_vol12の配置を1列左へ寄せる（実機指摘）

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **完了・未コミット**
発見: ユーザーのiPad実機確認（T21の再確認中）／ 起票: `brief/spec_return_log.md` **SR-003**

### 指示（原文の要点）

> 一段感覚がずれてるな。一つ左にずらして、バックスペースはよこはば１マス、英数も1マス。
> Pと？の場所はずれた後空白でいいよ

原因は、文字段が col3 から始まっているのに数字段の `1` が col2 にあり、
**q / a / z が数字より1マス右にずれて見えていた**こと（row2のcol2、row3/4のcol1-2が空白だった）。

### 変更後の盤面（13列）

```
col:     1     2   3   4   5   6   7   8   9   10  11   12    13
row1: [ Fn ][ 1][ 2][ 3][ 4][ 5][ 6][ 7][ 8][ 9][ 0][    ][ Bksp]
row2: [ Tab][ q][ w][ e][ r][ t][ y][ u][ i][ o][ p][    ][Enter]
row3: [    ][ a][ s][ d][ f][ g][ h][ j][ k][ l][   ][    ][Enter]
row4: [    ][ z][ x][ c][ v][ b][ n][ m][、][。][？][    ][Shift]
row5: [Ctrl][  ][      Space (col4-9)      ][  ][英数][    ][記号]
```

- Bksp・英数⇄日本語はいずれも**横幅1マス**（従来 colSpan 2）
- **Bkspは右端(col13)**。同日の追加指示で col12 → col13 へ移動し、右端が Bksp / Enter / Shift / 記号 で揃った。row1の空白は col12。
  これに伴い **layer1のF11も K112 → K113 へ追随**させた（「Fn+Bksp=F11」を保つため）。実機で Fn 押しっぱなし時の row1 が `F1..F10 / 空白 / F11` になることを確認済み
- 旧pと旧？の位置（col12）は空白キー。row1のcol12・row3のcol11-12も空白
- **Spaceは col4-9 のまま動かしていない**。文字段が col2-11 になった結果、
  Spaceの中心(6.5)と文字段の中心(6.5)が一致するため

### keyIdを位置に合わせて付け替えた

マニフェストの「keyIdはK+行+列2桁」規約を保つため、動いた文字キーのIDを付け替えた。

| 対象 | 旧 | 新 |
|---|---|---|
| row2 q..p | K203..K212 | **K202..K211** |
| row3 a..l | K303..K311 | **K302..K310** |
| row4 z..？ | K403..K412 | **K402..K411** |

- 付け替えは **layer0 と layer2（記号盤）の両方**へ同じ規則で適用（記号盤も一緒にずれる）
- layer1(Fn)・layer3(日本語モード) は row1・K213・K504・K511 しか持たないため変更不要
- 新規の空白キー: `K113` / `K212`(旧p) / `K311`・`K312` / `K412`(旧？) / `K512`
- **board のidと layer0 のキーが1対1**であることをスクリプトで突合して確認（迷子ゼロ）

### 1マス化に伴うラベル調整（実機スクショで見切れを確認して対処）

「英数⇄日本語」は6文字あり、1マスでは左右が切れて `数⇄日本` のようにしか読めなかった。
**2行ラベル `"日本語\n英数"`** に変更（小=日本語 / 主=英数）。

T21で入れたCSS（`body.jp:not(.sym) .key.dual` の主従入れ替え）がそのまま効くため、
**layer0の1定義だけで両方の状態が正しく出る**:

| モード | 主（大） | 小 |
|---|---|---|
| 英数 | 英数 (16px) | 日本語 (10px) |
| 日本語 | 日本語 (17px) | 英数 (10px) |

このため **layer3のK511上書きは削除した**（残すと入れ替えと二重になって逆さまに出る）。
layer3は K213(Enter/確定)・K504(Space/変換) の2キーのみになった。

### 凍結モックとの食い違い（SR-003）

CLAUDE.mdが「配置の正」と定める `brief/mockup/screen_mock_v0.4.html` は凍結領域のため
**一切変更していない**。結果として row2〜row5 でモックと実装が意図的に食い違う。
次に読むAIが「実装が壊れている」と誤認して戻さないよう、SR-003に新配置と経緯を記録し、
マニフェストのdescriptionにも「モックは古い」と明記した。
**新しい配置の正をどこに置くか（案A=実装のboard／案B=v0.5モック新規作成）はFABLE裁定待ち。**

### 検証記録

- `cargo test --workspace` = **108 passed, 0 failed**（テスト件数はT21から変化なし＝データ変更のため）
- 途中で1件failさせて直した: layer3のdescriptionへ**生の改行**を書き込んでJSONを壊した
  （`LOAD_JSON_SYNTAX`）。`real_data_startup_allows_the_ime_toggle_chord_behind_tg_fire` が
  検出した＝**T21で足した実データテストが早速効いた**
- **実Hubで `POST /api/reload`（B2の正規経路）→ 再配信**して確認（再起動していない）:
  - `/ipad` の実DOMからgrid座標を吸い出して照合。row1〜row5すべて上表のとおり
  - **記号盤も一緒にずれていること**を確認（`( ) { } < > [ ] = _` が q..p の真下=col2-11、
    `/ * + - # " ' & %` が a..l の真下=col2-10、`@ ~ | ¥ ^ $ ; ： ； 〜` が z..？ の真下=col2-11）
  - 「英数」キーの2行ラベルを実測: 英数モード=主16px/小10px、日本語モード=主10px/小17px
    （＝入れ替わる）。押して戻すとPCのIMEも表示も元通り
  - `/panel`（分割面）でも同一配置。Deckは無傷（15スロット）
  - Hubのエラーログ **0件**
- 凍結領域への差分: **ゼロ**（`brief/mockup/screen_mock_v0.4.html` を含め無変更）
- **未確認**: 実機iPadでの再確認。**ページ再読込だけでよい**（Hub再起動不要・token不変）

---

## T23 — P-005 段階A（`grid.rows`の欠陥修正・Deck縦2段・コピペリスト・Deck複数ロード）

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **完了・未コミット**
提案書: `brief/proposals/P-005_layout_components.md`（§7で裁定済み）

### 提案書の番号衝突と改番（重要）

当初 **P-004** として起票したが、ほぼ同時刻（1分差）に**別セッション**が
`brief/proposals/P-004_ssd_field_deck.md` を**同じ番号で**起票していた。
向こうは専属チャットへの依頼文で既に「P-004」を参照済みだったため、
**こちら（レイアウトと部品の三層化）を P-005 へ改番**した。
コード内コメント・JSON・STATE/INDEXの参照31箇所も同時に付け替え済み。

> **今後の運転上の注意**: `brief/proposals/` の採番は複数セッションが並行すると衝突する。
> 起票直前に `ls brief/proposals/` で最大番号を確認するだけでは不十分（同時起票が起きる）。
> 番号を取ったら**すぐ空ファイルを置く**か、採番をユーザーに確認するのが安全。

### ユーザー裁定（P-005 §7）

- 三層（**section / Component / slot**、1 section = 1 Component）で進める
- section は**自由グリッド**（row/col/colSpan/rowSpan）
- **`key.hold` を追加してよい**（切断時の強制解放を同時に入れること）
- **不変条件6の書き込み許可に `layouts/` を追加してよい**
- **段階A から着手**

### 段階Aでやったこと

| 変更 | 内容 |
|---|---|
| **`grid.rows`の欠陥修正** | 宣言されているだけで**一度も使われていなかった**（描画も検証もcolsのみ、行数はスロット数÷colsの暗黙値）。1ページのスロット数が `cols×rows` に収まることをロード時に検証し、クライアントは `grid.rows` を行数の正として使うようにした。溢れたスロットが黙って消える事故も同時に防いだ |
| **Deckの複数ロード** | `main.rs` の `DECK_PATH = "decks/deck_default.json"` 1枚固定をやめ、`keymaps/` と同じ**ディレクトリスキャン**（`decks/deck_*.json`）へ。`deckId`重複と`default`欠落はロード時に拒否 |
| **`deck.press` に `deckId`** | Deckが複数になりslotIdだけでは一意に決まらないため。省略時は`"default"`。deckIdもslotIdと同じ「位置ID」なので**不変条件1に抵触しない**。おかげでslotIdはDeckごとにローカルでよく、ユーザーが全Deck横断で一意なIDを考える必要がない |
| **`render: "grid" \| "list"`** | Deckの描き方。データ（label＋actionの並び）は同じで見た目だけが違う。省略時`grid`＝既存JSONは書き換え不要 |
| `decks/deck_default.json` | `grid` を `5×3` → **`8×2`（縦2段）**。スロットは15個のまま1つも減らしていない（容量16） |
| `decks/deck_story_paths.json`（新規） | コピペリスト。`render:"list"`＋`text`アクション2件 |
| `static/deck.html` / `static/panel.html` | `?deck=<deckId>` で描くDeckを選ぶ／list描画／`grid.rows`を使う／`deckId`を送る |
| `schemas/deck.schema.json` | `render` を追加。`rows` の説明に「段階Aから実際に効く」と明記 |

### コピペリストに新しいアクションは1つも要らなかった

既存の `text`（D20・KEYEVENTF_UNICODE）がフルパスをそのまま打ち込む。
D20が「クリップボードを黙って書き換えない」と定めているため、
**クリップボード経由ではなく直接入力がこの設計での正解**。新規なのは縦リストの見た目だけ。

### 検証記録

- `cargo test --workspace` = **113 passed, 0 failed**（T22の108件から**+5**、削除・弱体化ゼロ）
  - `deck::rejects_more_slots_than_grid_capacity`（`grid.rows`の欠陥の回帰テスト）
  - `deck::render_defaults_to_grid_and_accepts_list`
  - `startup::discovers_and_loads_every_deck_file`
  - `startup::missing_default_deck_is_rejected`
  - `startup::duplicate_deck_id_is_rejected`
- **実Hubでの確認**（`cargo run -p proto-hub` 再起動。起動ログに `decks=2`）:
  - `/deck` → 8列×2段・正方形スロット15個（実測68×68）。**縦2段になった**
  - `/deck?deck=story_paths` → 縦リスト（実測496×46の横長行・左寄せラベル）。タイトルも `(story_paths)`
  - `/panel` → 上ペインが8列×2段（実測86×86）、はみ出しゼロ、キーボードも無傷
  - スロット押下 → `deck.press{deckId,slotId}` がHubに届き、slotが解決され、
    **D5許可リストを通ってadapterへ発火（エラーログ0件）**
- **確認できなかったこと（正直に）**: 打ち込まれた文字が「意図した入力欄に入るか」は
  **このプレビュー環境では確認できない**。SendInputはOSの前面ウィンドウへ届くため、
  ページ内に`<input>`を作ってfocusしても（`document.hasFocus()`はtrueでも）文字は入らなかった。
  Hub側は成功を報告しているので**送出自体はできている**。着地の確認はユーザーの実機・実環境で必要。
  **なお検証中に2回発火しているため、どこか別のウィンドウにパスが打ち込まれた可能性がある**
- 凍結領域への差分: `crates/hub-core/` / `keymaps/keymap_default.json` / `brief/`の既存設計書・モック
  いずれも無変更。**`decks/deck_default.json` は変更した**（CLAUDE.mdの変更禁止リストには
  含まれておらず、D11が「ユーザーが編集するセットリスト」と定めているファイルのため）

---

## T24 — P-005 段階B/C（レイアウト面・十字キー・トラックボール部品化・Hub操作マニュアル）

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **完了・未コミット**

### ユーザー要望

1. 十字キー・リスト・Deck・キーボードを**JSONで自由に並べられる**ようにする（今回のゴール）
2. 以前作ったトラックボールを**iPadの部品としても使え**、**現在の十字キーと手動で取り換えられる**ように
3. そのための**Hub操作マニュアル**も一緒に開発する

### やったこと

| 新規/変更 | 内容 |
|---|---|
| `crates/proto-hub/src/layout.rs`（新規） | 区画割りの型・ロード・検証。**はみ出し／重なり／section id重複**をロード時に拒否 |
| `startup.rs` | `layouts/layout_*.json` をスキャンし、**参照先id（keymapId/deckId/surfaceId）の実在を全ロード後に検証** |
| `state.rs` | `SurfaceKind::Layout` ／ `layouts` ／ **keymapIdごとの `layer_states`** ／ 押しっぱなしキーの台帳 `held_keys` |
| `protocol.rs` | `key.press` に `keymapId`（省略時は従来動作）／`layer.state` に `keymapId`／`surface.config` に `keymaps`・`layouts`・`layerStates` |
| `ws.rs` | `/layout` ルート・QR target・Layout面の解決と配信・**切断時の押しっぱなしキー強制解放** |
| `proto-keymap` | **`key.hold`**（JSONに書く側）と `key.button`（出口）。`key.hold`は**upでもFireを返す唯一のアクション** |
| `proto-adapter-win` | `key.button` を press / release に分けて送出（`send_key()`はpress+release一体なので押しっぱなしにできなかった） |
| `deck.rs` | **Deckに `key.hold` を置くのを拒否**（`deck.press`にedgeが無く「離す」機会が来ないため） |
| `static/layout.html`（新規） | レイアウト面。keyboard / deck / trackball の3部品を描く1枚 |
| `static/trackball.html` | **`?embed=1` を追加**（ヘッダ・パラメータUI・下半分を隠すCSSと1行のクラス付与のみ。**Core/View/Gestureは無改変**） |
| `keymaps/keymap_dpad01.json` ＋ `layers/dpad01_layer0.json`（新規） | 十字キー。**新部品ではなく「3×3の小さなboardを持つkeyboard」** |
| `layouts/layout_ipad_main.json`（新規） | 左=コピペリスト／中=Deck／右=十字キー／下=キーボード |
| `layouts/layout_ipad_trackball.json`（新規） | 上記の右を**トラックボールに入れ替えた版**（`?id=` で切替） |
| `docs/HUB_MANUAL.md`（新規） | Hub操作マニュアル。コードを書かずJSONだけで部品配置・スロット数・割当を変える手順 |

### 設計上の判断（理由つき）

- **十字キーは新しい部品種別にしなかった。** 3×3のboardを持つkeyboardでしかない。
  おかげで新プロトコル・新描画経路がゼロで済み、割当変更も既存のレイヤーJSONの書き換えだけになる
- **レイヤー状態をkeymapIdごとにした。** 1画面に複数のキーボード部品（十字キー＋一枚キーボード）を
  置けるようになったため、面ごと（`layer_state`/`ipad_layer_state`）では足りない。
  同じkeymapを2区画に置いたら状態は共有される＝同じキーボードなら同じレイヤー、が正しい挙動
- **`key.press` に `keymapId`、`deck.press` に `deckId`。** どちらも「位置ID」であり、
  実行内容を決めるのは相変わらずHub側のJSONだけ。**不変条件1には抵触しない**
- **トラックボールはiframe埋め込みにした。** `trackball.html` の Core/View/Gesture は
  検証済みの資産（DEVBOARD T15〜T19）で、切り出しは壊すリスクが高い。
  **取引: この区画は自前のWS接続を1本持つ**（Hubから見ると別クライアント）。
  将来ちゃんと共有モジュール化するときは Core/View を切り出すこと
- **区画の重なりをロード時に拒否する。** 「パズルのように入れ替える」で最も起きやすい事故で、
  しかも画面上は「片方が消えた」ようにしか見えないため、原因究明に時間を取られる

### 検証記録

- `cargo test --workspace` = **119 passed, 0 failed**（T23の113件から**+6**、削除・弱体化ゼロ）
  - `layout::loads_a_valid_layout` / `rejects_duplicate_section_id` / `rejects_section_outside_the_grid`
    / `rejects_overlapping_sections` / `rejects_empty_sections` / `real_layout_files_load_successfully`
- **実Hubでの確認**（起動ログ `keymaps=4 decks=2 surfaces=2 layouts=2`）:
  - `/layout` → 4区画すべて描画。実測: 左リスト 291×329（行278×42）／中央Deck 588×329（マス67×67）／
    右十字キー 291×329（キー89×101）／下キーボード 1182×412（キー84×75）。**はみ出しゼロ**
  - **配置替えの実演**: `layout_ipad_main.json` の colSpan を 4/4/4 → 3/6/3 に書き換えて
    `/api/reload` → 中央Deckのマスが 42×42 → **67×67 に変わった**（HTMLは無改変・Hub再起動なし）
  - **十字キー⇄トラックボールの入れ替え**: `SEC-RIGHT` の component 1行を
    `{"kind":"trackball","ref":"tb01"}` に書き換えて `/api/reload` → その区画がボールに変わることを確認。
    `?id=ipad_trackball` でのURL切替も確認（`embed`クラスが当たっていることをiframe内DOMで実測）
  - **押しっぱなし（key.hold）の安全網**: 十字キー「↑」をpointerdownしたまま画面を離脱 →
    Hubログに `releasing keys still held by a disconnecting client keys=["W"]` を確認。
    **押したまま切断してもPCが操作不能にならない**ことを実証
- 凍結領域への差分: `crates/hub-core/` / `keymaps/keymap_default.json` / `brief/` の既存設計書・モック
  いずれも無変更
- **未確認**: 実機iPadでの操作感（十字キーのタップ目標・トラックボール区画の大きさ・
  iframeが2本目のWSを張ることによる体感）。**実機で触ってからの調整が要る**

---

## T25 — 技術書化の制作依頼書（CODEX宛）

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **依頼書作成済み・題名の裁定待ち**

### 発端

ユーザーより「このコードを **Tauri＋Rust＋customkeyboard の作り方**として技術書にしたい。
CODEXへの作成依頼と注意点・引継ぎを」との依頼。

### 🚩 前提が1つ崩れていた

**このリポジトリにTauriは1行も使われていない。** `Cargo.toml` / `Cargo.lock` /
`crates/*/Cargo.toml` を検索して0件。実体は:

- Hub: axum 0.8（HTTP＋WS）＋ tokio ＋ `windows` crate の SendInput
- 端末: 素のHTML+JS（フレームワーク・ビルド工程・CDN依存ゼロ）

しかも `CLAUDE.md` はこの構成を**確定アーキテクチャ**と定め、D2でフレームワーク導入を禁じている。
そのため「このコードからTauriの作り方を書く」ことは原理的にできない（書けば架空の実装になる）。

`docs/BOOK_REQUEST_CODEX.md` §0 に3案を提示し、**着手前にユーザー裁定を取るよう指示**した。

| 案 | 内容 | 裏付け |
|---|---|---|
| **A（推奨）** | 題名を実態に合わせる（「Rust＋WebSocketで作る自作キーボード／Stream Deck」） | 全章が動作するコードで裏付けられる。今すぐ書ける |
| B | Aに「HubをTauriで包む」最終部を足す | **Tauri章ぶんの実装が先に必要**。未検証コードを本に載せない線引きを明記 |
| C | Tauriで作り直して書く | CLAUDE.mdの確定アーキテクチャに反する。提案書＋裁定が先 |

### 依頼書に入れたもの

- 本の芯にすべき問い（「クライアントに何を送らせてよいか」＝不変条件1の設計思想）
- **落とし穴8件**を一次資料（DEVBOARD）つきで列挙。チュートリアルではなくこれが本の価値
- 読む順番10ファイル・規模の実測（Rust 6,793行／クライアント 3,466行／JSON 24件／テスト119件）
- 章立て案（全5部＋案Bのみ第6部）と、各章の裏付けコード
- **守ってほしいこと**（重要度順）:
  1. 🚩 **ユーザーの個人情報を載せない** — `deck_story_paths.json` に実在するDropboxパス、
     DEVBOARD/STATEにLAN IPとtokenが含まれる。伏字ルールを先に決めさせる
  2. コードを創作しない（全て実リポジトリからの引用・テストが通らないコードを載せない）
  3. 事実を盛らない（**Windows専用**・`hub-core`はvendored凍結で自作ではない）
  4. **セキュリティの書き方** — 題材は「ネットワーク越しにPCへキー入力を注入するアプリ」であり、
     書き方を誤ると攻撃ツールの作り方になる。認証を外す手順を書かせない
  5. リポジトリを書き換えない（原稿のみ作成）
- **未確認事項5件**を明示（実機操作感／`text`の着地／Android実指／ALT+GRAVE依存／IME状態は推定）
- 引き継ぎ（P-004採番衝突・別セッションのP-004・未コミット・exeロックでビルド失敗する件）

### 検証記録

- `cargo test --workspace` = **119 passed, 0 failed**（T24から変化なし。本件はドキュメントのみ）
- Tauri不在は `grep -rin "tauri" Cargo.toml Cargo.lock crates/*/Cargo.toml` で0件を確認
- 規模の数値は `wc -l` / `ls | wc -l` の実測値
- 凍結領域への差分: ゼロ

---

## T26 — 十字キーが文字入力になる件の確認と、矢印キー版の追加

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **完了・未コミット**

### ユーザーからの確認依頼

> 十字キーが、AやDの文字入力になってないかチェック

### 確認結果: **なっている。仕様（ただし説明不足だった）**

実データと adapter の変換表を突き合わせた結果:

```
D102 ↑ -> key.hold vk="W" -> 0x57 (VK_W)
D201 ← -> key.hold vk="A" -> 0x41 (VK_A)
D203 → -> key.hold vk="D" -> 0x44 (VK_D)
D302 ↓ -> key.hold vk="S" -> 0x53 (VK_S)
```

**ラベルは矢印だが、送っているのは英字キーそのもの。** テキスト欄にフォーカスがあれば
`wasd` と打ち込まれる。T24で「ゲームの十字キー」という要望に対しWASD（多くのゲームの移動キー）を
既定にしたのは妥当だが、**その副作用をJSONにもマニュアルにも書いていなかった**のが落ち度。

### 対応

- `keymaps/keymap_dpad_arrows.json` ＋ `layers/dpad_arrows_layer0.json`（新規）
  … 盤面の形はdpad01と同一、`vk` だけ `UP`/`LEFT`/`RIGHT`/`DOWN`。**矢印キーは文字を生まない**
- `dpad01_layer0.json` の description に⚠️注意書きを追記（後で読む人が驚かないように）
- `docs/HUB_MANUAL.md` §5 に「WASD版は文字が出ます」の節と比較表、§8のトラブル表に1行追加

**既定は `dpad01`（WASD）のまま**にした。元の要望が「ゲームの十字キー」だったため。
差し替えは `layouts/layout_ipad_main.json` の `ref` を `dpad_arrows` にして `/api/reload` するだけ。

### 検証記録

- `cargo test --workspace` = **119 passed, 0 failed**（データ追加のみ）
- 実Hubで `/api/reload` → `keymapsLoaded: 5`、`config.keymaps` に `dpad_arrows` が出現。
  レイヤー内容を実測: `dpad01 = ↑:W ←:A →:D ↓:S` / `dpad_arrows = ↑:UP ←:LEFT →:RIGHT ↓:DOWN`
- 凍結領域への差分: ゼロ

---

## T27 — 十字キーを矢印版へ／押した感（押し込み＋波紋）／キーボードVol1.3を新設

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **完了・未コミット**

### 1. 十字キーを矢印版に切替（ユーザー指示）

`layouts/layout_ipad_main.json` と `layout_ipad_trackball.json` の `ref` を
`dpad01`(WASD) → **`dpad_arrows`**(UP/LEFT/RIGHT/DOWN) に変更。文字が混入しなくなった。
`dpad01` は消さずに残してある（ゲーム用に戻したくなったら `ref` を戻すだけ）。

### 2. 押した感（P-005提案の②③を実装）

| | 内容 |
|---|---|
| ② 押し込み | `pointerdown` で `.pressing` を付け、CSSで **`transition: none` ＋ `scale(.94)`**。
戻りだけ `cubic-bezier(.2,.9,.3,1.3)` で軽く弾ませる。**「入り0ms・戻りだけ緩やか」が体感を決める** |
| ③ 波紋 | 触れた座標を要素内相対に直し、そこを中心に円を広げる。半径は押した点から四隅までの最大距離＝角を押しても全体を覆う |
| 付随 | `prefers-reduced-motion: reduce` の端末では拡縮も波紋も出さない |

### 🚩 実装中に見つけた欠陥: 波紋がDOMに溜まり続ける

`animationend` で自分を消す実装にしたが、**12連打して9個が残り、1.2秒後も消えなかった**。

原因は **タブが非表示のあいだCSSアニメが進まず、`animationend` が永久に来ない**こと。
iPadでは他アプリへの切替・画面ロックが日常なので、**実運用で確実に踏む**種類の漏れ。

対策を3重にした:

1. `prefers-reduced-motion` の端末では波紋を**作らない**（`display:none` だとアニメが走らず
   `animationend` も来ないため、作るだけ無駄に溜まる）
2. 新しい波紋を作るとき、そのキーに残っている**古い波紋を捨てる**（連打時の上限）
3. `animationend` に加えて **`setTimeout(cleanup, 700)`** を必ず仕掛ける
   （非表示中でもタイマーは進むため、これが最後の砦）

修正後の実測: 12連打しても**常に最大1個**、1.2秒後に**0個**。

### 3. キーボード Vol1.3 を新設（スクショの配列）

ユーザー提示のスクリーンショット（Android Gboardの日本語配列）を盤面化。

**番号についての判断**: ユーザーの言う「Ver1.1（現行）→ Ver1.2（新規）」は、リポジトリの既存採番
（現行が既に `ipad01_vol12`＝Vol1.2）と衝突する。さらに **`IPAD_KEYMAP_ID = "ipad01_vol12"` が
`state.rs` にハードコードされており、改名すると `/ipad` が起動時に落ちる**。
そのため **現行= `vol12`（凍結）／新規= `vol13`** とした（CLAUDE.md「複製して新Volを作ること」に一致）。

| 新規ファイル | 中身 |
|---|---|
| `keymaps/keymap_ipad01_vol13.json` | **22列グリッド**の盤面。1キー=2列にすることで、スクショのA段の**半キーずれ（スタガード）を1列ぶんのずれとして正確に表現**した |
| `layers/ipad01_vol13_layer0.json` | 基盤。Q〜Pの右肩に数字ヒント、確定=ENTER、ー=MINUS、日本語⇄英数=tg.fire、←→=矢印 |
| `layers/ipad01_vol13_layer2.json` | 記号盤。**1段目に数字**（Q=1…P=0 とヒストの位置を一致させてある） |
| `layers/ipad01_vol13_layer3.json` | 日本語モード表示。確定・Spaceのみ（layer2と1つも重ならないことを検証済み） |
| `layouts/layout_ipad_v13.json` | `ipad_main` の複製で、`SEC-KEYBOARD` の ref だけ `ipad01_vol13` |

**Vol1.3を作るにあたってVol1.2のファイルは1文字も触っていない**（新規5ファイルのみ）。
※ `git status` で `keymap_ipad01_vol12.json` 等が変更扱いになるのは、**同日の T21（tg.fire）と
T22（1列左寄せ）による変更**が未コミットで残っているためで、本タスク（T27）由来ではない。

切替は `/layout?id=ipad_main`（Vol1.2）と `/layout?id=ipad_v13`（Vol1.3）で、URLだけ。
片方を壊しても他方は無傷。

### 🚩 Vol1.3で顕在化した既存機能との衝突

T21で入れた「日本語モードでは2行ラベルの主従を入れ替える」CSSは、Vol1.3では**害になる**。
Vol1.3はQ〜Pの全キーが2行ラベル（数字ヒント＋英字）なので、日本語モードにすると
**数字が主・英字が従になって盤面が読めなくなる**。

対策: `renderLabel` で **1行目が数字だけのラベルには `.dual` を付けない**ようにした。
「小さい行が日本語側の代替表記のときだけ入れ替える。数字ヒントは代替表記ではない」という規則。

### 検証記録

- `cargo test --workspace` = **119 passed, 0 failed**（データ・クライアントのみの変更）
- 盤面の自動突合（スクリプト）: board↔layer0が**1対1で迷子ゼロ**／各行が22列に収まり**重なりゼロ**／
  row2がcol2始まり（半キーずれ）／row3はcol1-18で右側が空く（スクショと一致）／
  **layer2 ∩ layer3 = 空**（D3のレイヤー優先順位の罠を回避）
- 実Hubで `keymaps=6 layouts=3` をロード。`/layout?id=ipad_v13` を実際に描画して目視確認
  （Q〜Pの数字ヒストが右肩に出ること、A段が半キーずれること、右下に矢印が出ること）
- 押した感の実測: `transition: none` / `transform: matrix(0.94,…)` / 波紋が押した点(15,13)から
  半径220pxで発生 / 離すと `.pressing` が外れて戻りのtransitionが効く
- `ipad_main`（Vol1.2）の回帰確認: キー59個・十字キーが `↑←→↓`（矢印版）で無傷
- 凍結領域への差分: ゼロ
- **未確認**: 実機iPadでの押した感（波紋の見え方・押し込みの深さ）と、Vol1.3の打鍵感

---

## T28 — 「PCでは変わるのに端末が変わらない」の原因究明と再発防止

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **完了・未コミット**

### 症状（ユーザー報告）

> PCはレイアウト変わってるけどIPADは変わらないよ

### 原因: **端末が古いtokenを掴んだままだった**

`curl` で実測して確定:

```
古いtoken (9c48…) -> 401
現在のtoken (a0f4…) -> 200
```

本セッション中にHubを複数回再起動しており、**再起動のたびにtokenが再生成される（D8）**。
端末が前のURLを開いたままだと、WSが延々と拒否され新しい`surface.config`を受け取れない。
**画面には前の内容が残るので「変わらないだけ」に見える。**

Hubのログに5秒おきに出ていた `WS_TOKEN_INVALID` は、まさにこの端末の再接続試行だった
（T27の報告時点では「どこかの古いタブ」と書いたが、**実際にはiPadだった**）。

### さらに悪いことに、端末側は原因を知らせていなかった

クライアントはWSが閉じると「切断中は操作できません。再接続中…」としか出さない。
**「tokenが古い」と「単に切れている」を区別していなかった**ため、ユーザーは
原因が分からないまま待ち続けることになる。これが本質的な欠陥。

### 対策1: レイアウトごとにQRを出す

`connection_url` に **`layout:<layoutId>`** ターゲットを追加し、ランディングページが
**読み込まれているレイアウトの数だけQRカードを自動生成**するようにした。
レイアウトを足せばQRも自動で増える（Hub側の表示コードを触らなくてよい）。

これで端末は「QRを読み直す」だけで最新tokenの目的の画面へ行ける。URLを手打ちしなくてよい。

### 対策2: 端末に「tokenが古い」と言わせる

- 新規 `GET /api/ping?token=` … token検証だけを行う軽量エンドポイント（200 / 401）
- クライアント4枚（layout/panel/ipad/deck）で、**WSが閉じたらpingを叩き、401なら断定して案内**
  「このURLのtokenは古くなっています。PCのHub画面を開き直して、QRを読み直してください」
- 接続バッジも **「tokenが古い」** に変える

### 🚩 実装中に見つけた欠陥: 警告が2秒ごとに消されていた

最初の実装では案内が**画面に出なかった**。原因は**2秒ごとの再接続が
`setStatus("接続中…")` → `setStatus("切断中…")` で警告を上書きし続けていた**こと。
警告を出す仕組みを足しても、それを消す仕組みが既にあったので意味が無かった。

対策: `setStatus` に**貼り付き（sticky）**を入れ、tokenが古いと判明した後は
エラー以外の更新を無視するようにした。修正後は6秒経っても警告が残ることを実測。

### 検証記録

- `cargo test --workspace` = **119 passed, 0 failed**
- `/api/ping`: 古いtoken→**401** / 現在のtoken→**200** を curl で実測
- ランディングページ: QRカードが **9枚**（既存6＋レイアウト3）。
  **全部の画像が実際に生成されている**ことを `naturalWidth>0` で確認。
  URLエンコード（`target=layout%3Aipad_v13`）も正しく解決
- 古いtokenで `/layout` を開く → バッジ「tokenが古い」＋赤字の案内が表示され、
  **6秒後も消えない**ことを実測
- 新しいtokenで開く → 通常どおり接続（「接続しました」／キー41個＝Vol1.3の36＋十字5）
- 凍結領域への差分: ゼロ

### 学び（今後この種の報告を受けたとき）

**「PC側は変わったのに端末が変わらない」は、まずtokenを疑う。**
`curl -o /dev/null -w "%{http_code}" "http://localhost:8770/api/ping?token=<端末のURLのtoken>"`
で1秒で切り分けられる。

---

## T29 — 端末が古い画面を出し続ける（真因はHTTPキャッシュ）

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **完了・未コミット**

### 症状

T28で「tokenが古い」と診断し、**新しいtoken付きのURLを渡したのに**:

> IPADは再起動してもずっと古いものを表示している

### 真因: 配信ヘッダに `Cache-Control` が無かった

`curl -D -` で実測:

```
HTTP/1.1 200 OK
content-type: text/html
accept-ranges: bytes
last-modified: Sat, 05 Sep 2026 10:56:29 GMT
etag: "6a9bf55d.10a0fd74-86b0"
          ← cache-control が無い
```

同時に、**Hubが配っているHTMLは新しい**ことも確認（`ripple`/`pressFeedback`/
`diagnoseDisconnect` が19箇所ヒット）。つまり**サーバは正しく、端末が取りに来ていなかった**。

`Cache-Control` が無いと、ブラウザは `last-modified`/`etag` だけを見て
**ヒューリスティックキャッシュ**（独自判断での再利用）を行う。iOS Safariはこれが特に強く、
**端末を再起動してもHTTPキャッシュは消えない**。結果、Hub側が何を変えても端末は
何時間も古い画面を出し続ける。

**T28の診断（tokenが古い）は、その時点では正しかったが、症状の原因は2つあった。**
tokenを直したら2つ目（キャッシュ）が残っていた。1つ直して直らなかった時点で
「別の原因が重なっている」と考えるべきだった。

### 対策: ルータ全体に `Cache-Control: no-store`

`tower_http::set_header::SetResponseHeaderLayer` をルータ全体に適用。

判断の根拠: この面は**JSONを書き換えて即反映するのが売りの道具**で、配るのは
LAN内の数十KBのHTMLでしかない。キャッシュで得られるものより、
**古い画面が出る害の方が明確に大きい**。

### 検証記録

- `cargo test --workspace` = **119 passed, 0 failed**
- `curl -D -` で `cache-control: no-store` が `/layout` と `/`（ランディング）の両方に
  付くことを実測
- 新tokenで `/layout?id=ipad_v13` を開き、Vol1.3が正しく描画されることを確認
  （keyboards=`[dpad_arrows, ipad01_vol13]` / キー41個 / 十字キー`↑←→↓`）
- 凍結領域への差分: ゼロ

### ⚠️ 端末側の後始末が一度だけ必要

`no-store` は**次に端末が実際に取りに来たとき**から効く。
既にキャッシュを掴んでいる端末は、**一度だけ**URL末尾に `&cb=1` 等を足して
取りに行かせる必要がある。それ以降は二度と起きない。

### 学び

**「サーバは正しいのに端末が古い」を見たら、`curl -D -` で配信ヘッダを見る。**
`Cache-Control` が無い静的配信は、ブラウザに好き放題キャッシュさせている状態。
tower-httpの `ServeFile` は既定で `Cache-Control` を付けない。

---

## T30 — ヘッダに切り替えセレクタを2つ（登録board / キー配列）

日付: 2026-09-05 ／ 担当: Claude Code（Opus 5）／ 状態: **完了・未コミット**

### ユーザー要望

> headerのTitleロゴ：左上のレイアウトでkeyboardformatを切り替えれるようにしたらどうだろう？
> セレクターは二つ作ってほしくて、
> KeyDeck：｛登録board切り替え（今は一つだけど、これからboardを増やしたら、丸ごと表示を
> 差し替えるステート駆動で）、Layout→文字入力のkeyboardがあった場合、キーLayoutを切り替えられる｝

### 実装: **Hub側の変更はゼロ**

`surface.config` は既に全レイアウト・全キーマップを配っているので、
**端末側の状態だけで両方とも実現できた**（ユーザーの言う「ステート駆動」そのもの）。

| セレクタ | 効果 | 実装 |
|---|---|---|
| **BOARD** | 画面を丸ごと差し替え | `layoutId` を差し替えて `render()` |
| **LAYOUT** | キーボードの配列だけ差し替え | `keymapOverride` を持ち、区画描画時に `effectiveKeymapId(section)` で解決 |

### 設計上の判断

- **上書きは「文字入力キーボード同士」のときだけ効かせる。**
  そうしないと、十字キー（5キー）の区画まで一枚キーボードに置き換わってしまう。
  判定は `board.keys.length >= 20`（`TEXT_KEYBOARD_MIN_KEYS`）。
  十字キーは選択肢にも出ないし、差し替え対象にもならない
- **LAYOUTセレクタは、いま出している画面に文字入力キーボードがあるときだけ出す**
  （要望の「文字入力のkeyboardがあった場合」をそのまま条件にした）
- **`layouts/*.json` は書き換えない。** これは「この端末ではこう出す」という表示上の選択。
  全端末の既定を変えたいときは従来どおりJSONを編集する。段階D（配置GUI）で
  ファイルへ書き戻す話になったときに、不変条件6の議論をすればよい
- **選択は端末が覚える**（localStorage）。URLの `?id=` が指定されていればそちらが優先
  （ランディングページのQRは `?id=` 付きなので、QRから入れば必ずそのレイアウトになる）
- BOARD変更は `history.replaceState` でURLへ反映。そのURLを共有すればいきなりその画面へ

### 検証記録

- `cargo test --workspace` = **119 passed, 0 failed**（クライアントのみの変更）
- 選択肢の自動生成: BOARD=`[ipad_main, ipad_trackball, ipad_v13]`（3レイアウト）、
  LAYOUT=`[ipad01_vol12, ipad01_vol13]`（**dpad_arrows は正しく除外**）
- **キー配列の切替**: vol12→vol13 で描画キーが **64→41** に変化し、
  **十字キーの区画は `↑←→↓` のまま無傷**であることを実測（＝上書きが文字入力キーボードに限定されている）
- **board切替**: `ipad_main`→`ipad_trackball` で SEC-RIGHT が `kind-trackball`（iframe）に変わり、
  URLに `id=ipad_trackball` が反映。**キー配列の選択は独立して維持**された
- **永続化**: `?id=` 無しで再読込 → BOARD/LAYOUT ともlocalStorageから復元されることを実測
- 途中で `layoutId` の二重宣言（既存の `const` と新しい `let`）でJS構文エラーになったが、
  Nodeでの構文チェックが検出したため画面に出す前に修正
- 凍結領域への差分: ゼロ

---

## 実機確認（iPad）— 2026-09-05

ユーザー報告: **「ipad成功です」**

T29（`Cache-Control: no-store`）でキャッシュを抜けたことで、iPad実機でレイアウト面が
正しく表示・動作することを確認。これにより T24〜T30 で「未確認」としていた
**実機表示に関する項目が解消**した。

確認できたとみなすもの:

- `/layout` がiPad実機で描画される（4区画：コピペリスト／Stream Deck／十字キー／キーボード）
- レイアウトの世代切り替え（Vol1.2 / Vol1.3）とboard切り替えがiPad上で動く
- ヘッダの2セレクタ（BOARD / LAYOUT）がiPad上で操作できる

**まだ評価を聞けていないもの**（動くことと、使い心地が良いことは別）:

- 押した感（押し込み0ms・波紋）の体感が期待どおりか
- Vol1.3（Gboard風配列）の打鍵感。数字が記号盤経由でよいか
- 十字キー（矢印版）のタップ目標の大きさ
- トラックボール区画の広さ、iframeが2本目のWSを張ることによる体感
- コピペリストの `text` が実際に狙った入力欄へ着地するか（プレビュー環境では検証不能だった項目）

---

## T31 Hubの操作画面を作る（`/settings` が実質空だった）— 2026-09-06

きっかけ: ユーザーから「Hubが確認できてない、見せて」。実際に開いたところ
**`/settings` には再読込ボタン1つしか無く、中身が空**だった。構成を知るには
JSONを手で開くしかない状態で、運用として成立していない。

### 見つけた欠陥

- **起動バナーにレイアウトが1つも出ていなかった**。`main.rs` の `println!` が
  手書きの7行で、P-005でレイアウトを足したときに更新されず取り残されていた。
  Webのランディングページは自動生成なので出ており、**両者が食い違っていた**
- 再読込の成功メッセージが `keymaps` しか数えていない（decks/layoutsも読み直しているのに）

### やったこと

1. `ws.rs` に **`connection_targets()` を新設し、接続先の表を1箇所に集約**。
   ランディングページ・`/api/formats`・起動バナーの3箇所がこれを見る。
   同じ表を複数箇所に手書きしていたことが上の欠陥の原因なので、構造として潰した
2. `GET /api/formats?token=…` を新設（**読み取り専用**。何も書かない）。
   targets / keymaps / decks / layouts を返す
3. `static/settings.html` を書き直し。接続先一覧（名前・種別チップ・URL・QRボタン）＋
   キーボード／Deck／レイアウトの一覧。QRはモーダルで1枚だけ大きく出す
   （凍結モック `screen_mock_v0.4.html`「② 設定」の行構造に従う。一覧にQRを並べない）
4. 再読込後に一覧を自動で引き直すようにした（数字が変われば成功が目に見える）

### 検証記録

- `cargo test --workspace` → **119 passed / 0 failed**
- `node --check`（settings.htmlのscript抽出）→ 構文OK
- 起動バナーに **レイアウト3種が出ることを実測**（ipad_main / ipad_trackball / ipad_v13）
- `/settings` 実測: 接続先9件・キーボード6件・Deck 2件・レイアウト3件（区画4つ×3）を表示
- QRモーダル: `layout:ipad_v13` を押下 → `/api/qr?target=layout%3Aipad_v13` が 200。
  コロンのエンコードが効いている
- 再読込ボタン → 「再読込しました。」（緑）＋ 一覧が再描画。**console.error ゼロ**
- 凍結領域への差分: ゼロ

### 補足（設計上の注意）

- `keys` 列が `—` になるのは split キーマップ（`board` ではなく `halves` で持つ）。
  壊れているのではない
- QRのURLにはトークンが入る。トークンは `main.rs` で**起動のたびに新規生成**され
  ディスクに保存されない（D8）。**QR画像を保存しても次回は使えない**ことを画面に明記した

---

## T32 P-005 段階D のデザインモック（区画エディタ）— 2026-09-06

ユーザーからの質問:「SVGでモックを作れるか。SVG / HTML / XHTML のどれが適切か」

新規追加: `brief/mockup/mock_p005_stage_d_layout_editor.html`
（**既存モックは1つも触っていない**。凍結領域への差分ゼロ）

### 手段の結論: HTML + CSS Grid。SVGは不向き

決め手は「**データが既に格子だから**」。`layouts/*.json` の
`row / col / colSpan / rowSpan` が `grid-row / grid-column: <col> / span <n>` へ
**そのまま乗る**。座標変換の算数が1つも要らない＝ズレる余地が無い。

SVGだと同じ絵を出すのに、セル寸法×番号の掛け算を全部自前で持ち、
`<text>` は折り返さず、フォーム部品は置けず（`foreignObject`＝結局HTML）、
キーボード操作と読み上げを手で付けることになる。
SVGが勝つのは「画像として書き出す」1点のみで、それはこの画面の目的ではない。

モック内にHTML版とSVG版の**同じ区画を並べて描き**、比較表を置いた。口頭の主張ではなく実物で示す。

### XHTMLについて

採用しない。理由:
- `application/xhtml+xml` で配信すると**1箇所の閉じ忘れで画面全体が出なくなる**（部分描画なし）
- 「タグとの紐づけ」はHTML5の `data-*` 属性が標準で、XHTMLに利点は無い
- D2（素のHTML+JS）に対するアーキ変更にあたる。やるならSR起票が必要
- XHTMLが必須なのはEPUB3の本文。**用途が違う**（このユーザーは技術書も作っているため混同しやすい）

### モックに入れた機能と検証

- ドラッグ移動・つまみでリサイズ・パレットから追加・区画削除
- **重なり／はみ出しをその場で赤表示**（`layout.rs` が実際に弾く条件と同じ）。不正な間は保存ボタンを無効化
- 出来上がるJSONを常時表示（実物の `layouts/layout_*.json` と同じ形）
- 検証: SEC-LEFT を右へ2マス動かす合成PointerEventを流し、
  `col 1→3` / 重なり検出2件 / verdict赤 / 保存無効 を実測。**console.error ゼロ**
- 横スクロールが出ていたのを媒体クエリで縦積みへ（実測 `scrollWidth == clientWidth`）

### 注意

- 自動操作の `left_click_drag` は**PointerEventを出さない**ため効かなかった。
  検証は合成PointerEventで行っている。実機の指・マウスでの操作感は未評価
- 保存は未実装。`layouts/` への書き込み＝不変条件6の拡張が要る（未裁定）

---

## T33 最小サイズと「減らす／減らせない」の切り分け（モックへ実装）— 2026-09-06

ユーザー要望:
1. Componentの枠を手動で調整したい
2. **Component内にスクロールバーを出さない**。入りきらないなら表示要素数を減らす
3. トラックボールのような実体がある部品は**下限を決めて**、中身が機能するようにする。今後の追加分も

### ユーザー裁定（2026-09-06）

| 論点 | 決定 |
|---|---|
| 枠の調整 | **数値入力（行/列/幅/高）＋ 部品ごとの最小サイズを手動指定**の両方 |
| あふれたDeckスロット | **ページ送りへ回す**（到達できなくならない） |
| 最小の基準端末 | **iPad Pro 12.9 横だけ**（1366×1024） |

未確認だったが私の判断で進めた点（ユーザーへ明示済み）:
**文字キーボードは「減らす」対象外**。キーを間引いたら打てないため、トラックボールと同じ下限側。

### 見つけた現状の欠陥（未修正・実装側の宿題）

- **`layout.html` は `overflow:hidden` で切り落としている。** 入りきらないスロットは
  何の表示もなく消える。スクロールより悪い（気づけない）。**いま実機で起きうる**
- `panel.html:340` は逆に `overflow:auto`。コメントに
  「押しにくいボタンを並べるより、はみ出す方がまし」と明記されており、方針が真逆
- `MIN_CELL` が layout.html=**40** / panel.html=**44** で食い違い。44（標準的なタップ目標）へ寄せる

### 設計の核: 最小は「マス数」では決まらない。実ピクセルで決まる

1マスの実寸は画面で変わる。基準端末で **1マス ≒ 114×109px**。
そこから各部品の必要pxを出し、**切り上げてマス数**にする。

| 部品 | 必要px | 最小マス | 型 |
|---|---|---|---|
| `ipad01_vol12` | 572×220 | 6×3 | floor |
| `ipad01_vol13` | 484×176 | 5×2 | floor |
| `dpad_arrows` | 132×132 | 2×2 | floor |
| `tb01`（トラックボール） | 220×220 | 2×3 | floor（**人が決めるしかない**） |
| `default`（Deck 15件） | 44×70 | 1×1 | reduce |
| `story_paths` | 200×70 | 2×1 | reduce |

**`ipad01_vol13` の最小colSpanは2**（1キー＝2列の半キーずらし）。
必要幅は `22*(44/2)=484px` であって `22*44=968px` ではない。
当初968pxと見積もったが誤りで、実データから訂正した。

### 検証記録

- `node --check` 構文OK / **console.error ゼロ**
- 最小サイズ計算がPython側の独立計算と**全項目一致**
- 下限クランプ: SEC-KEYBOARD を①つまみで極端に縮める ②数値欄に「1」を打つ
  → **どちらも 5×2 で停止**（実測）
- ページ分割: Deck default（15件）を 6×4 → 1×1 と縮め、
  16→16→14→10→2個/ページ、1→1→2→2→8ページ と変化することを実測
- 途中欠陥: `fitReport` が **Deckの宣言格子（8×2）で頭打ちにしていなかった**ため
  「4マスの区画に9行入る＝72個」と出ていた。宣言容量で clamp して修正

### 残り

- 実装側（`layout.html` / `panel.html`）はまだ旧挙動のまま。切り落とし・スクロール・
  MIN_CELL不一致の3点は**別途の修正が必要**
- Deckのページ送りUI（帯26px想定）は未実装

### T33-a 修正: ドラッグが実マウスで死んでいた（2026-09-06）

ユーザー報告「Componentの移動も可変もできない」。**開発中だからではなく、私のバグだった。**

**原因**: `move()` が毎回 `render()` を呼び、`gridEl.textContent = ""` で
**掴んでいる要素そのものを破棄していた**。要素が外れるとポインタキャプチャも解放され、
以降の `pointermove` は新しい要素（`mode` が null）へ飛ぶ。
結果、実マウスでは最初のひと動きでドラッグが死ぬ。

**なぜ最初の検証で見逃したか**: 合成PointerEventを**同じ要素参照へ1回だけ**送っていた。
外れた要素にもリスナとクロージャは残っているので、合成イベントは通ってしまう。
`document.contains(box)` を見ておらず、通ったことを「動いた」と誤読した。
実マウスは `pointermove` を連続で出すため、1回だけの検証では再現しない種類の欠陥。

**修正**: `render()` を2つに割った。
- `render()` … 骨組みを作り直す。区画の増減・レイアウト切替のときだけ
- `refresh()` … 位置・大きさ・赤表示・判定・JSONだけ更新。**要素は作り直さない**

ドラッグ中に呼ぶのは `refresh()`。`secEls`（区画id→要素）で使い回す。

**検証**:
- 連続する `pointermove` 8回で `document.contains(box)` が**全て true**、
  `col` が 1→2→3 と追従することを実測（修正前は最初の1回で false）
- **自動操作の `left_click_drag` でも成立**。SEC-LEFT が col=1 → col=6 へ移動し、
  重なり検出が働くことを実測

**前回の説明の訂正**: T32で「`left_click_drag` はPointerEventを出さないので効かなかった」と
書いたが**誤診**。イベントは出ていて、上記のDOM破棄が原因だった。

### T33-b 重なりの扱いを裁定（2026-09-06）

ユーザー質問:「重なったらエラーでいいか。それともtab式／レイヤー式にするか」

**裁定: エラー（重なりは許さない）。tab式・レイヤー式は採らない。**

理由:

1. **この装置は見ないで押す道具**。指の下にあるものが指を動かさずに変わる仕組みは、
   Deck・キーボードの価値である位置の記憶を壊す
2. レイヤー式は「押したいボタンを押す前に1回クリック」が要る。
   **1タップ＝1コマンドというDeckの前提が崩れる**
3. `layout.rs` は既にロード時に重なりを弾いている。tab/レイヤーは検証・描画・
   隠れた部品のWS接続とレイヤー状態の扱いを作り直すことになる
   （隠れたキーボードのレイヤー状態が裏でずれる問題も付く）
4. **「同じ場所に別のものを出したい」はboardセレクタが既に満たしている**。
   画面まるごとの差し替えで、iPad実機で確認済み。区画ごとのtabより指の記憶を壊さない

### 「赤くするだけ」では足りなかった

重なったまま指を離せてしまい、**下に潜った区画が見えなくなる**（ユーザー提示の
スクリーンショットで SEC-LEFT が SEC-CENTER の下に隠れていた）。

**方針: 不正な形はコミットさせない。**
ドラッグ中は赤く見せて理由を伝えるが、確定はさせず直前の状態へ戻す。
これなら「Aを一旦どかしてBを入れる」も途中経過が赤いだけで妨げられない。

`commitOrRevert()` を1つ置き、**4経路すべてを同じ扱い**にした
（ドラッグ／数値入力／手動下限の引き上げ／部品の追加）。
ドラッグだけ厳しく数値入力は緩い、という不整合を作らない。

**検証（すべて実測）**:

| 操作 | 結果 |
|---|---|
| 重なる位置へドラッグ→離す | ドラッグ中 col=5 で赤 → 離すと col=1 へ復帰＋メッセージ |
| 空きへドラッグ→離す | col=10 へ移動して確定。判定は緑 |
| 列に「4」を打つ（重なる） | col=1 のまま。差し戻しメッセージ |
| 幅に「9」を打つ（重なる） | colSpan=3 のまま。差し戻しメッセージ |
| 空きが足りない部品を追加 | 区画数 3→3。「6×3ぶん無いため追加しませんでした」 |

console.error ゼロ。

### T33-c モックの機能棚卸し（2026-09-06）

ユーザー要望: ①空きが無いときのエラー ②置いた部品を「置ける部品」へ戻す
③足りない機能の洗い出し

#### キャッシュ事故（3回目）

ユーザー提示のスクリーンショットが**重なったまま確定した状態**だった。
現行版で再現を試みたが、どの操作列でも不正状態に到達しない
（全消し→8種追加／一部消して追加を6周／手動下限を4→6→9→12：いずれも不正なし）。

原因は配信側。`keydeck-mock` が素の `python -m http.server` で、
**Last-Modified しか返さない**ためブラウザが古いページを保持していた。
実測: `#flash` も `BUILD` も存在せず、修正前のHTMLが配信されていた。

対処2つ:
1. `brief/mockup/serve.mjs` を新設し、**全応答に `Cache-Control: no-store`**。
   `.claude/launch.json` の `keydeck-mock` をこれに差し替え（`/` でモック本体へ）
2. 画面右上に**版（`rev4 2026-09-06`）を表示**。
   「古い画面を見ているか」を推測ではなく事実で切り分けられるようにした

Hub本体は T29 で no-store 済みだったが、モック用サーバーだけ抜けていた。

#### 追加した機能（rev4）

- 空きが無いときの通知を**パレットの真上の帯**へ（判定行は格子の下で視線から遠い）
- パレットに**「配置済み」チップ**。部品プールとして読めるようにした
- 削除を**「← 置ける部品へ戻す」**に。戻した部品名を通知に出す

検証（実測）: 空きなし追加→区画数4のまま＋通知 ／
「戻す」→区画3件へ、`dpad_arrows` のチップが消えて通知が出る ／ console.error ゼロ。

#### 棚卸しの結果（未実装。ユーザーの選択待ち）

**A. 実務で困る**
1. **`component.ref` を差し替えられない** — 最頻出操作（vol1.2→vol1.3）なのに
   削除して置き直すしかなく、位置と大きさを失う。JSONでは1行
2. **レイアウトの複製・新規作成が無い** — リポジトリの運用方針そのもの
   （「既存を編集せず複製して新しい世代を作る」layout_ipad_v13.json に明記）を支えていない
3. **Undo が無い** — 削除は即時。「元に戻す」は全体を初期状態に戻す粗さ
4. **区画idを変えられない** — 調査結果: idは `layout.html` の `data-section-id` に
   使われるだけで、`layout.rs` は重複のみ検査。**改名は安全**

**B. あると良い**
5. 中身のプレビュー（今は色の付いた箱だけ。最小サイズは計算で保証しているが体感は別）
6. キーボード操作（矢印キーで1マス移動）
7. `description` の編集（実物は改行入りの長文。複数行対応が要る）
8. `grid` 12×9 の変更（基準端末を1つに決めた今は不要）

**C. Hub側の不足（実装へつなぐ前に必要）**
9. **`/api/formats` が `minColSpan`/`minRowSpan` を返さない** — 最小サイズ計算に要る。
   モックはハードコードした表で代用している
10. **`/api/formats` が surfaces（トラックボール）を返さない** — targets/keymaps/decks/layouts
    の4つだけ。トラックボールをパレットに出せない
11. 保存の口が無い（不変条件6の拡張が未裁定）

**D. 入れないほうがよい**
12. 自動詰め（空白は指の逃し場所として意味がある）
13. tab／レイヤー（T33-b で裁定済み）
14. 部品そのものの新規作成（VIAL型エディタ=T9の領分。混ぜると画面の責務が壊れる）

### T33-d 部品へ戻す操作を「放り込み」に（2026-09-06）

3秒長押し案はユーザーが中止。採用は**ドラッグ中だけ現れる放り込み口**。

**なぜこちらが良いか**: 掴んでいる間しか現れないので誤爆しにくく、
いま行っているドラッグをそのまま使える。長押しは「押している間の進み具合」を
別途見せないと壊れて見えるうえ、待ち時間が操作を止める。

**実装（rev6）**
- 「置ける部品」パネルに膜（`.dropveil`）を重ね、**移動ドラッグ中だけ**表示
- 膜は `pointer-events: none`。ポインタは掴んだ区画にキャプチャされているため、
  **当たり判定は座標で行う**（膜側にイベントは飛んでこない）
- 膜の上に来たら濃くする（`.over`）。放せば効くことが見て分かる
- ボタンからの削除と放り込みは `returnToPalette()` を共有。2つの入口で挙動が食い違わないように

**狭い画面の問題と対処**: 幅900px以下ではパレットが格子の上へ回るため、
格子を見ている間はパレットが画面外にある（実測 `top=-275`、見えているのは下端58pxだけ）。
掴んだ時点でパレットの可視高さを測り、90px未満なら膜を**画面上端へ貼り付ける**
（`.pinned`）。当たり判定は膜の実位置を見ているので、貼り替えるだけで追従する。

**検証（すべて実測）**

| 経路 | 結果 |
|---|---|
| 掴む前 | 膜は非表示 |
| 区画を掴む | 膜が出る |
| パレット上へ運ぶ | `.over` が付く |
| そこで離す | 4件→3件・通知あり・膜は片付く |
| **つまみでリサイズ中** | **膜は出ない**（大きさ変更で消える事故を防ぐ） |
| リサイズをパレット上で離す | 4件→4件（消えない） |
| パレットを通過して格子へ戻して離す | `.over` が外れ、4件→4件。判定は緑 |
| 狭い画面で帯に放り込む | `.pinned` 付き・`.over` 付き・4件→3件 |

console.error ゼロ。

### T33-e 役目を終えた部分の削除（2026-09-06・rev7）

ユーザー指示「sectionの下のSVGと、この画面にいるものはもういらなければ削除しておいて」。

**削除したもの（1021行 → 924行、97行減）**

| 対象 | 消した理由 |
|---|---|
| 比較パネル（HTML版＋SVG版＋比較表） | SVG/HTML/XHTMLの判断は済み。**根拠は DEVBOARD T32 に記録済み**なので、画面に置き続ける必要が無い |
| `.compare` / `.cols2` / `.tablewrap` / `table` `th` `td` `td.y` `td.n` | 比較パネル専用。他から使っていない |
| `renderMini()` と呼び出し、`#miniHtml` | 比較パネル専用 |
| `.nums` | 読み取り専用だった旧プロパティ欄の名残。`.numedit`（入力欄）に置き換わった時点で死んでいた |
| `#verdict.warn` | 通知を `#flash` へ移した時点から誰も付けていない |
| 冒頭コメントの「右下のパネルに示す」 | パネルを消したので嘘になる。DEVBOARD T32 への参照に書き換え |

**削除後の検査（機械的）**
- `getElementById` が参照するid **11個 / HTMLに在るid 11個 → 参照できないid ゼロ**
- 定義済みCSSクラスのうち**未使用のもの ゼロ**
- `compare` `cols2` `tablewrap` `miniHtml` `renderMini` `.nums` `verdict.warn` `<table` `<svg` の残骸 **すべて0件**

**削除後の通し確認（機能が壊れていないこと）**

| 確認 | 結果 |
|---|---|
| 重なる移動 → 差し戻し | OK |
| 下限クランプ（幅に1を入力） | 5×5 で停止 |
| ページ分割（Deck 3×2） | 14個/ページ・全2ページ |
| パレットへ放り込み | 4→3件 |
| 空きなしで追加 | 4→4件（拒否） |
| 横スクロール | なし |

`node --check` OK / console.error ゼロ。

---

## T34 モックをVol1.1で凍結／見た目の規則を制定（2026-09-06）

### Vol1.1 を凍結

ユーザー指示「現在の物をVol1.1として保存。他の開発でも使うかも」。

- `brief/mockup/mock_layout_editor_vol1.1.html` … **凍結。編集禁止**
- `brief/mockup/mock_p005_stage_d_layout_editor.html` … Vol1.2。以降の開発はこちら

既存のVol運用（キーマップと同じく「編集せず複製して次の世代」）に合わせた。
凍結版の冒頭に凍結の旨と収録機能を明記し、版表示も `Vol1.1 凍結` に変えてある。

### 中身の描画は流用できる。**Rustは一切不要**

ユーザー質問「Vol1.2からComponentの中身に現在の機能のレイアウトを使えるか。
Rustの本番のほうがよければ保留で」→ **モック側で進めてよい。Rust変更ゼロ。**

調査結果:
- `static/layout.html` の `renderKeyboard`（514行〜）は**通信に一切触っていない**
  （`socket`/`send` の参照ゼロ）。読むのは `config.keymaps` と `layerStates`、
  発火は `bindKey` に分離済み。つまり**描画部分はほぼ純粋**
- 中身のデータは `/api/formats` を拡張しなくても手に入る。エディタが
  `layout.html` と同じく WS の `surface.config` を受ければ、keymap/deck の全体が来る

**ただし条件が1つ**: `renderKeyboard` / `renderDeck` / `renderLabel` / `resolveDisplay` を
`static/components.js`（素のJSファイル。D2に反しない）へ**切り出して共有する**こと。
モックへコピーしてはいけない。

理由: **同じ表を2箇所に手書きして食い違う事故を、今日この repo で実際に起こしている**
（接続先一覧が `main.rs` と `ws.rs` で食い違い、レイアウト3種がコンソールに出ていなかった。
T31で `connection_targets()` に集約して解消）。描画を複製すれば、エディタのプレビューが
実機と違う絵を出すようになる＝プレビューが嘘になる。

### 見た目の規則を制定: `brief/ref_component_visual_rules_v1.md`

**色は「その部品がHubへ送るメッセージ種別」で決める**（`protocol.rs` の `ClientMessage`）。

| 色 | メッセージ | 部品 |
|---|---|---|
| 青 `--kind-key` | `key.press` | 一枚キーボード、十字キー |
| 紫 `--kind-cmd` | `deck.press` | Stream Deck、コピペリスト |
| 緑 `--kind-state` | `surface.state` | トラックボール |

見た目の分類と通信の分類がずれない。「十字キーは青か緑か」は
`key.press` を送るかで決まる（送るので青）。
新種別が3つのどれにも当てはまらないなら、**色より先に不変条件1の裁定が要る**。

アイコンは3分類に対応するインラインSVG。`currentColor` で描き色はCSSから与える。
絵文字は使わない（端末で字形が変わり iPad と PC で揃わない）。
**色だけに意味を載せない**（色が見分けにくい人・モノクロでも通る）。

> 比較パネルのSVGは削除したが、アイコンにSVGを使うのは矛盾しない。
> SVGが向くのは動かさない絵であり、アイコンはまさにそれ（T32の結論どおり）。

### 見つけた衝突（未修正）

`static/settings.html` の接続先チップが `layout` に**青**を使っている。
青は `key.press`（キーボード）に割り当てたので、同じ色が別の意味を持っている。
接続先は部品種別ではなく画面の入口なので、灰系の中立色へ寄せる必要がある。

### 検証

`node --check` OK / console.error ゼロ /
旧名 `--kb` `--deck:` `--tb` `.dot` の残骸 **すべて0件** /
パレット7個・区画4個にアイコンが出ることを実測。

### T34-a 入口の色を灰へ（2026-09-06）

ユーザー裁定「入り口は灰色で頼む」＋「画面が白いのは苦手。黒めなら何でもいい」。

`static/settings.html` を修正。**色を持つのは部品種別だけ**にした。

| 出てくるもの | 部品種別か | 色 |
|---|---|---|
| レイアウトの区画の `kind` | そう | 3色（青/紫/緑） |
| 接続先（kb / deck / ipad / trackball / panel / layout） | 違う＝**画面の入口** | 中立の灰 |
| Deckの描き方（grid / list） | 違う＝**描き方** | 中立の灰 |
| キーマップの種別（split / single） | 違う＝盤の形 | 中立の灰 |

`.chip.layout` / `.chip.deck` / `.chip.trackball` を廃止し、
`.chip.kind-keyboard` / `.chip.kind-deck` / `.chip.kind-trackball` に置き換え。
クラス名に `kind-` を付けたのは、**部品種別にしか付かない**ことを名前で分かるようにするため。

**検証（実測の背景色）**
- 接続先9件すべて `rgb(42,51,82)`（中立の灰）
- Deckの描き方 grid / list ともに中立の灰
- 区画の部品種別のみ keyboard=`rgb(36,54,92)` / deck=`rgb(58,43,77)` / trackball=`rgb(22,63,58)`
- 背景 `rgb(16,21,38)`（暗いまま）
- console.error ゼロ

明るさの方針（白い画面にしない）を `brief/ref_component_visual_rules_v1.md` §2 に明記。

---

## T35 画面構成の変更: トップ=レイアウト編集 / QR=専用ギャラリー（2026-09-07）

ユーザー指示:
- QRはトップではなく専用ページへ。**ギャラリー形式**にして簡易図や実機スクショを載せ、
  カード右上のQRボタンでそのカードがQRに切り替わる
- **トップはレイアウトを変えられる画面**にする

### ルーティング

| URL | 変更前 | 変更後 |
|---|---|---|
| `/` | QRを縦に9枚（`ws.rs` 内で生成） | **`static/editor.html`（レイアウト編集）** |
| `/connect` | （無し） | **`static/gallery.html`（QRギャラリー）** |
| `/shots/*` | （無し） | `static/shots/` の実機スクショ配信 |

`index_page`（サーバー生成のQR一覧HTML・約2,100文字）を削除。gallery.html が置き換えた。
起動バナーも「▼PCで開く（トップ/QRギャラリー/設定）」「▼端末で開く」に整理した。

### editor.html — モックからの移植

`brief/mockup/mock_layout_editor_vol1.1.html`（凍結）で確かめた設計をそのまま実装へ。
**決め打ちのデータを全部外し、WS `surface.config` から作る**。

- `surface.config` は `/layout` 面（実機で使う面）と**同じ入口**。だからエディタが
  見ている構成と実機の構成が食い違わない。追加APIは不要（Rust側の新規実装ゼロ）
- 最小サイズは実データから計算: キーボードは board の列数・行数と**一番細いキーのcolSpan**、
  Deckは grid とスロット数。トラックボールだけは中身から寸法が出ないので既定値220×220px
- `surface.config` は surfaces を配らないため、**トラックボールのidは既存レイアウトの
  参照から拾う**（参照されていないものはパレットに出せない。既知の制約）
- **保存ボタンは常に無効**。不変条件6の拡張が未裁定であることを画面とtitle属性に明記した。
  形が正しくても押せない（押せない理由を1つに保つため）

**検証**: 実データから出した最小サイズが、モック時代のハードコード値と**全項目一致**。

| 部品 | 最小 | 型 |
|---|---|---|
| ipad01_vol12 | 6×3 | floor |
| ipad01_vol13 | 5×2 | floor |
| dpad01 / dpad_arrows | 2×2 | floor |
| tb01 | 2×3 | floor |
| default | 1×1 | reduce |
| story_paths | 2×1 | reduce |

レイアウト3件・区画4件・パレット7件を実測。

### gallery.html — カードの表と裏

- 表 = 実機スクショ、無ければ**Hubが読み込んでいるJSONから描いた簡易図**。
  「それらしい絵」を手で描くと構成変更で古くなって嘘になるので、実データから描く
- 裏 = QRとURL。カード右上のボタンで切り替わる。**QR画像は開くまで読み込まない**
  （9枚ぶんを最初に取りに行かない）
- 実機スクショは `static/shots/<target>.png` に置くだけで差し替わる

### 移植中に踏んだ欠陥（4件）

1. `QrQuery` の定義が `index_page` ブロックの中にあり、削除で巻き添えになった → 復帰
2. 構成が届く前に `clone(SOURCE[layoutId])` を呼び、`undefined` を JSON.parse して落ちた
   → 空のmodelで初期化し、届いてから作る
3. `log()` を定義せずに使っていた（モックには無かった）→ D9書式で追加
4. WSメッセージのタグ名を `t` と書いたが正しくは **`type`**（`#[serde(tag = "type")]`）
   → 修正。あわせて `error` メッセージも拾うようにした

### ギャラリーで踏んだ欠陥（2件）

5. **`[hidden]` が `.shot { display:block }` に負け**、スクショが無いカードに
   壊れた画像とalt文字が出た。`settings.html` で同じ罠を注記していたのに再発
   → `.shot[hidden] { display:none !important }`
6. 置いていないスクショを毎回取りに行き、**404が9件**コンソールに並んだ
   → `/api/formats` が `shots`（実在するファイル名）を返すようにし、
     クライアントは**実在するものだけ**読む。当てずっぽうに取りに行かない

### 検証

- `cargo build` OK / `cargo test --workspace` **119 passed / 0 failed**
- エディタ: 実データ読み込み・最小サイズ一致・保存ボタン無効を実測
- ギャラリー: カード9枚・簡易図9枚・**`.shot` 要素0個**（404を出す原因が消えた）・
  QRボタンで裏返り `/api/qr?target=layout%3Aipad_v13` が200で表示されることを実測
- `docs/HUB_MANUAL.md` の接続手順を新構成に更新

### T35-a ヘッダの整理と、部品共有の規則（2026-09-07）

**ヘッダから基準値を削除**（ユーザー指示）。`TARGET` / `TAP` の値はコードに残っており、
最小サイズの計算にそのまま使われている。根拠が要るのは「なぜこの大きさで止まるのか」を
見る場面だけなので、**最小サイズの箱の中へ移した**。

実測: ヘッダ = `KeyDeck — レイアウト編集 / 保存は未対応 / [レイアウト選択] / 元に戻す /
保存 / 接続中 / QRギャラリー→ / 設定→`。`#targetInfo` は存在しない。

### 部品の共有について（ユーザーの問いへの回答）

問い:「Componentだから全体に変更が影響する物と、独立したものを Ver1.1・Ver2.1 は
別々のものとして取り扱うか？」

**答え: 別々のものとして扱う。ただし「版」ではなく「別のid」として。**
規則を `brief/ref_component_versioning_v1.md` に制定した。要点:

1. **部品は参照で共有される。編集は必ず伝播する。** 分けたいなら複製して別id
2. **レイアウトが部品の中身を上書きする仕組みは作らない。**
   役割の分離（レイアウト=配置／部品=中身）が壊れ、
   「`ipad01_vol12` はどんな盤面か」がレイアウトを知らないと答えられなくなる
3. 直す=誤字や明らかな誤り（全部に効いてほしい）／複製=配置やキーの増減（既存を壊しうる）。
   **迷ったら複製**。複製の代償はファイル1つ、編集の代償は気づかない破壊

現状の共有関係（実データ）:

| 部品 | 使っているレイアウト |
|---|---|
| `ipad01_vol12` | ipad_main, ipad_trackball |
| `dpad_arrows` | ipad_main, ipad_v13 |
| `default` | 3つ全部 |

**実装した対策**: 区画を選ぶと、その部品が他のレイアウトでも使われていれば警告を出す。
気づかずに直すのが一番危ないため、編集の前に目に入る位置へ置いた。

実測: SEC-KEYBOARD（`ipad01_vol12`）選択時に
「他の 1 件でも使われています: ipad_trackball」と表示。

**未実装**: 複製ボタン（この区画だけ新しいidへ差し替える）／部品そのものの編集画面。
ギャラリーのカードから飛んで中身を微調整する流れはまだ無い。

**見つけた揺れ**: keymapIdは `vol12`（ドット無し）、モックのファイル名は `vol1.1`（ドット有り）。
idはドット無しに統一する方針を規則書へ明記した。

### T35-b SR-004起票とパレット行の折り返し修正（2026-09-07）

**SR-004**: 設計書v0.3の **D15「`/` を設定画面に置換」** に対し、実装は `/` を
レイアウト編集にした。D15の意図（QR縦並びランディングの廃止・トップを操作画面に）は
満たしており、D15が求めた一覧行の構造とQRモーダルは `/settings` と `/connect` に
実装済み。違うのは「トップに置く操作画面が設定ではなく編集になった」点のみ。
凍結領域からの逸脱なので `brief/spec_return_log.md` へ記録した（裁定待ち）。

**パレット行の折り返し**: 幅が足りないと「最小 6×3」が2行に割れて行が崩れていた
（ユーザー提示のスクリーンショットで発生）。縮むのを名前だけにし、右側の数字は
`white-space: nowrap` で折り返させないようにした。
実測: 全7行が高さ33px一定・横溢れなし。名前には `title` を付けて省略時も読める。

---

## T36 レイアウト保存（P-005 段階D）— Hubで編集して実機へ反映（2026-09-07）

### トークンは再発行されない

ユーザー質問「Hubで変更をしたら、トークンが再発行になりますか？」→ **なりません。**
トークンは `main.rs` の起動時に1回だけ作られる（D8）。保存はHubを再起動しないので変わらない。
したがって**保存後の新トークンQRポップアップは不要**（条件が成立しない）。
繋がったままの端末には、保存後の再配信で**自動で反映される**。

### 不変条件6を拡張（ユーザー裁定）

`CLAUDE.md` の書き込み許可に **`layouts/`** を追加。防御条件を条文に明記した:

- 書き先は `layouts/layout_<layoutId>.json` のみ。**ファイル名はクライアントから受け取らない**
- `layoutId` は `[a-z0-9_]{1,64}` のみ（`ws::layout_id_is_safe`）。
  **パスを組み立てる前の唯一の関門**
- 受け取った本文はそのまま書かず、Layoutとして**解釈し直し整形したもの**を書く
- 書く前に `.bak`、書いた後に**全体を読み直して検証**。失敗したら巻き戻す
- 検証は `startup::load_startup_data` を再利用（保存専用の緩い検証を作らない）

### `POST /api/layout/save` の防御（実測）

| 試したこと | 結果 |
|---|---|
| `layoutId: "a/b"` | 422 `LAYOUT_SAVE_REJECTED`（ファイル名になる前に拒否） |
| `layoutId: "../evil"` | 422 同上 |
| 区画の重なり | 422 `LOAD_LAYOUT_INVALID`「A と B が (row 1, col 2) で衝突」 |
| 存在しない `ref` | 422。**書いた後に読み直して落ち、`rolledBack: true` で巻き戻し** |

4回の不正保存のあと `layouts/` に**残骸ゼロ**を実測。

### 実機まで通した（本番経路）

`/layout?id=ipad_v13` を開いたタブを繋いだまま、エディタで SEC-RIGHT を 3×4 → 3×3 に変更して保存。

- 端末側は**再読込なしで追従**（`SEC-RIGHT` の grid-row が `1/5` → `1/4`。
  `performance.getEntriesByType("navigation").length === 1` でリロードしていないことを確認）
- `.bak` が作られ、本体が更新されることを実測

### 🚩 見つけたデータ損失（重大・修正済み）

**1回目の保存で `description` が消えた。** ファイルが 1734 → 895 バイトになり、
`layout_ipad_v13.json` の運用申し送り**373文字**（Volの切り替え手順・「編集せず複製する」方針）
が失われた。`.bak` から復元済み。

原因: エディタの送信本文に `description` が無く、`Layout.description` は
`#[serde(default)]` なので空文字として解釈され、そのまま書かれた。

**二重に塞いだ**:
1. エディタが `description` を持ち回って送る
2. Hub側でも、本文の説明が空で既存ファイルに説明があれば**引き継ぐ**。
   エディタは配置しか編集しないので「空＝指定しなかった」とみなす。
   説明を消したいときはJSONを直接編集する

再発防止テスト2件を追加（`save_carries_over_the_existing_description` /
`real_layout_descriptions_survive_a_save_round_trip`）。
修正後に再実測: **description 373文字が保たれ、配置だけ更新**された。

なお `Layout` は全階層に `deny_unknown_fields` が付いているため、
**構造体が持たないフィールドはそもそもファイルに存在できない**＝往復での取りこぼしは
原理的に起きない。今回消えたのは「構造体は持っているが送っていなかった」項目だった。

### 検証

- `cargo test --workspace` **126 passed / 0 failed**（保存の防御5件＋説明文2件を新規追加）
- テストのパス解決を修正: `cargo test` の作業ディレクトリはクレート直下で、
  実行時のワークスペース直下と違う。実物を読むテストは `CARGO_MANIFEST_DIR` から解決する
- テスト後、`layouts/` は元の状態へ復帰済み（3件とも説明文つき・SEC-RIGHT rowSpan 4）

---

## T37 部品の描画を1本化し、エディタに実物のプレビューを出す（2026-09-07）

ユーザー要望「現在のレイアウトでは、正しく部品を配置できてるかわからないので、
部品のデザインをわかるようにしてほしい」。

### `static/components.js` を新設（描画の唯一の実装）

実機の面（`layout.html`）とエディタ（`editor.html`）が**同じ関数で描く**。
別々に描いていると必ずズレ、ズレた瞬間にエディタのプレビューは嘘になる。
このリポジトリでは同じ形の事故が実際に起きている（接続先一覧が `main.rs` と
`ws.rs` に手書きで散り、レイアウト3種がコンソールに出ていなかった。T31で解消）。

- `renderKeyboard` / `renderDeck` / `resolveDisplay` / `renderLabel` / `fitDeck`
- **CSSも持つ**（見た目が2箇所に分かれてもズレるため）。`.kd-surface` に閉じ、
  色は `var(--panel, 既定値)` の形でホスト側の定義が勝つ
- 2つのモード: `interactive: true`（実機。押下はホストのコールバックへ）/
  `false`（プレビュー。`.kd-preview` で `pointer-events` を殺す）
- **送信そのものは書かない。** 何を送るかはホストが決める（不変条件1）

### `layout.html` を共有版へ移行

描画関数を削除して委譲に置き換え（6,242文字）、**重複CSSを26件削除**。
呼び出し側の書き方（`renderKeyboard(body, keymapId)`）は変えていないので、
`rerenderKeyboards` 等の既存経路は無改修。

**移行前後の照合（実機で通っている面なので数値で突き合わせた）**

| | 移行前 | 移行後 |
|---|---|---|
| キー数 | 41 | 41 |
| スロット数 | 17 | 17 |
| キーの背景 | `rgb(28,35,56)` | 同じ |
| 角丸 / 文字 | 10px / 22px | 同じ |
| 盤面の列数 | 3 | 3 |

スクリーンショットも一致。`function resolveDisplay` `renderLabel` `bindKey`
`fitDeck` `DECK_GAP` `MIN_CELL` の残存 **0件**。

### エディタに実物のプレビュー

区画の中に、実機と同じ関数で中身を描く。

- 実測: `ipad_v13` でキー41・スロット17、`ipad_trackball` でキー59・スロット17＋丸1
  （どちらも `/layout` と同数）
- 4区画すべてに `.kd-preview` が付き、押せない
- **ドラッグの邪魔をしない**: プレビュー中央で `elementFromPoint` が返すのは `.sec`。
  プレビューの真上から掴んで1マス下へ移動できることを実測（row 1 → 2）。
  重なる先へ運んだ場合は従来どおり差し戻される
- 小さい区画は文字を消して形だけ見せる（`tiny-labels`）。
  「正しく置けているか」に要るのは形と比率であって文字ではない

### 途中で直した欠陥

**見出しがキーに埋もれて読めなくなった。** 影だけでは足りなかったので、
`.sid` / `.sref` に不透明の座布団を敷いた。

### 検証

`cargo build` OK / `cargo test --workspace` **126 passed / 0 failed** /
`node --check` は components.js・layout.html・editor.html すべてOK / console.error ゼロ。

### T37-a 「切り替わらない」の原因究明とQRボタン（2026-09-07）

ユーザー報告「変更をした。MainのQRを読み込んだ。しかし十字キーはトラックボールへ
切り替わらなかった。失敗です」。

**保存は正しく動いていた。読んだQRが別のレイアウトだった。**

ディスクの事実:

| レイアウト | 右上の区画 | 更新時刻 |
|---|---|---|
| `ipad_v13` | `SEC-NEW1` = **trackball · tb01** | 12:15（保存された） |
| `ipad_main` | `SEC-RIGHT` = keyboard · dpad_arrows | 9/5 19:41（未更新） |

編集したのは `ipad_v13`、読んだQRは `ipad_main`。保存経路に欠陥は無い。

### ただしこれは製品側が作った罠

編集画面と接続用QRが**別のページ**にあり、
「いま編集しているのはどれか」と「いま読もうとしているQRはどれか」を
突き合わせる手段が無かった。ギャラリーには9枚のカードが並ぶだけで、
どれが直前に編集したものかを示していない。

**対処（ユーザー要望と一致）**: 編集画面のヘッダ、レイアウト選択のすぐ隣に
**QRボタン**を置いた。出るのは**いま選んでいるレイアウトのQR**なので、
取り違えようがない。

実測:
- ボタン→中央にモーダル。見出し「レイアウト: ipad_v13」、URLも一致、QR画像は読み込み成功
- **カードの中を押しても閉じない** / **画面外を押すと閉じる** / **Escでも閉じる**
- レイアウトを `ipad_main` へ切り替えて開き直すと、見出しもURLもQR画像も切り替わる
- `[hidden]` が `display:flex` に負ける罠は今回も先回りして打ち消してある
  （settings.html・gallery.html で既に踏んでいる）

### 残っている使いにくさ（未対応）

- 部品を入れ替えると区画idが `SEC-NEW1` になる。元の `SEC-RIGHT` という名前が失われる。
  棚卸しA-4（区画idの改名）が効くところ
- ギャラリー側にも「直前に編集したレイアウト」の印は無いまま

---

## B-001 Fn（mo）を押したまま切断すると、レイヤーが残り続ける（2026-09-17）

技術書の執筆中、第6章の画面を撮るために「押した接続を先に閉じる」手順を踏んだところ見つかった。
**読本屋（技術書）側の問題ではなく、KeyDeck 本体＝操作系の不具合として切り分ける。**

```
対象アプリ    KeyDeck（C:\00_master\DevApps\APP_KeyDeck01）
対象面        /ipad（ipad_layer_state）。/layout 面の keymap ごとの状態も同じ作りなので同様と推定（未確認）
見つけた経緯  技術書 初版の第6章の撮影（2026-09-15）→ 2026-09-17 に切り分けのため再現
```

### 再現手順

1. Hub を起動する（`cargo run -p proto-hub`、port 8770）
2. WebSocket で `/ws?token=<token>&surface=ipad` に接続し、`{"type":"key.press","keyId":"K101","edge":"down"}` を送る（K101 = Fn = `{"t":"mo","layer":1}`）
3. **`up` を送らずに接続を閉じる**（実機では、Fn を押したまま画面を閉じる・スリープする・Wi-Fi が切れる）
4. 新しく `/ipad?token=<token>` を開く

### 期待する挙動

押していた端末が居なくなった時点で、その端末が押していた `momentary` は取り消され、
新しく開いた端末はレイヤー0で始まる（＝押しっぱなしのキーを切断時に離す `release_held_keys` と同じ考え方）。

### 実際の挙動（2026-09-17 実測）

新しく開いた端末が **レイヤー1のまま** 始まる。

```
（切断後に新しく開いた画面）  キー = Fn | F1 | F2 | F3 | F4 ／ バッジ = Layer 1
（Fn を1回押して離したあと）  キー = Fn | 1 | 2 | 3 | 4 ／ バッジ = 英数
```

Hub のログ:

```
key press ... key_id="K101" edge=Down layers=0   outcome=layer-change   momentary: [1]
client disconnected client_id=0
client connected    client_id=1                  ← 新しい端末。レイヤー1が配信される
key press ... key_id="K101" edge=Up   layers=0,1 outcome=layer-change   momentary: []
```

### 影響の切り分け

- **表示と発火は一致している。** 新しい端末にも `surface.config` でレイヤー状態が配られるため、
  画面には F1〜F10 と出て、押せば F1〜F10 が出る。**「1 と表示されているのに F1 が出る」という黙った食い違いではない**
- したがって「再接続後に、画面と違うキーが飛ぶ」形の誤入力は起きない
- 残る害は、**利用者が気づかずに数字段を打つと F キーが飛ぶ**こと（見た目は F 表示なので気づけるが、
  指の位置で打つ人は踏む）。復帰は Fn を1回押して離すだけ
- 押しっぱなしのキー（`key.hold`）は `release_held_keys` が解放するので、この件の対象外

### 判定

**修正対象。** ただし緊急ではない（黙った誤入力にならず、Fn 1回で復帰するため）。
KeyDeck を配布・公開する前には直す。

### 修正案（未着手）

押しっぱなしのキーと同じ形にする。**接続ごとに「その端末が有効にした `momentary` レイヤー」を台帳に持ち、
切断時にその分だけ取り消す。** 面の状態を丸ごと 0 に戻すのは不可。
同じ面を別の端末が開いていて、そちらが Fn を押している場合に巻き添えになる。

### 検証記録

```
2026-09-17  再現・実測（キーは一切発火させず、画面の表示とログだけで確認）
            scratchpad/repro_fn_disconnect.mjs（一時スクリプト。リポジトリには置いていない）
            cargo test --workspace 139 passed / 0 failed（2026-09-15 実行、この件では未変更）
```

---

## B-002 `layout.switch` / `app.launch` の入れ子 `fire` が許可リストに載らない（2026-09-17）

技術書の監修（別AIによる指摘のみのレビュー）で見つかり、こちらで裏を取った。
**T21 で一度直したはずの穴（`tg.fire` の内側が許可リストに載らない）が、後から足した2つのアクションで再発している。**

```
対象アプリ    KeyDeck
対象          crates/proto-hub/src/state.rs の canonical_command_id
              crates/proto-hub/src/startup.rs の all_actions / all_actions_with_source
現物への影響  keymaps/layers/appswitch_layer0.json の A1（YouTubeへ）
```

### 何が起きているか

`canonical_command_id` が内側の `fire` へ潜るのは `TgFire` だけ。

```
Action::TgFire { fire, .. } => canonical_command_id(fire),   ← 潜る
Action::AppLaunch { id, .. } => Some(format!("app:{id}")),   ← 自分のidだけ。fireは見ない
（LayoutSwitch は match の _ => None に落ちる）               ← 何も載らない
```

`all_actions` も最上位の `action` しか辿らないため、入れ子の `fire` は起動時の許可リストに一度も現れない。

一方 `fire_action` は、`layout.switch` / `app.launch` を処理したあとに内側の `fire` を**実行しようとする**。
内側は `key` / `chord` / `text` のいずれか（`check_nested_fire` がロード時に制限している）で、
これらは発火の直前に許可リストと照合されるため、**そこで弾かれる**。

### 現物での実害

`appswitch` の A1 は `{"t":"layout.switch","id":"ipad_youtube","fire":{"t":"chord","keys":["WIN","1"]}}`。
`WIN+1` を持つ定義は**リポジトリ全体でこの1か所だけ**なので、許可リストに `chord:WIN+1` は存在しない。

```
期待   ボードが ipad_youtube に切り替わり、同時に Win+1 が PC へ飛ぶ
実際   ボードは切り替わるが、Win+1 は飛ばない
       ログ: action resolved but is absent from the startup allow-list: chord:WIN+1
```

**症状は T21 のときと同じ「画面は変わるのに PC が変わらない」。** 防御が正しく働いた結果として機能が壊れている。

### 裏の取り方（2026-09-17）

コードと現物データからの確定。**実機で A1 を押す試験はしていない**
（もし私の読みが誤っていて許可リストに載っていた場合、Win+1 が実際に飛んで利用者の画面が切り替わるため）。

```
grep -rn '"WIN", *"1"' keymaps decks   → appswitch_layer0.json の1件のみ
state.rs の canonical_command_id       → TgFire だけが再帰。LayoutSwitch は _ => None
startup.rs の all_actions              → 最上位の action のみを列挙
```

### 修正案（未着手）

1. `canonical_command_id` を `LayoutSwitch { fire, .. }` / `AppLaunch { fire, .. }` でも内側へ潜らせる
   （`AppLaunch` は `app:{id}` と内側の両方が要る。片方だけだと今度は起動側が弾かれる）
2. 起動時の列挙（`all_actions` / `all_actions_with_source`）も入れ子まで辿る
3. **テストで固定する。** 「入れ子 `fire` を持つ3種のアクションすべてについて、内側が許可リストに載る」を1本
   （`tg.fire` だけを見るテストでは、次に入れ子アクションを足したときにまた同じ穴が開く）

### 補足 — token の権限範囲（不具合ではなく、記録しておくべき事実）

`POST /api/layer/save` は token だけで通り、受け取った `action` を書いたあとに
`load_startup_data` で**許可リストを作り直す**（`s.command_registry = loaded.command_registry;`）。

したがって token を持つ者は「レイヤーを保存する → 許可リストに載る → 押す」の2手で、
**いまボードに置かれていないキー・ショートカット・任意の文字列も PC へ送れる**。
これは編集機能として意図された動作だが、**token は「利用者の鍵」ではなく「編集権限つきの鍵」**である。
技術書 初版の第2章がここを取り違えて書いていたため、改訂版で直す（`00_勉強フォルダー/20_書籍_初版/改訂メモ_v2候補.md`）。

- 2026-09-18 ポータブル版の作り直し: `cargo test --workspace` 139 passed → `RUSTFLAGS="-C target-feature=+crt-static" cargo build --release -p proto-hub --target-dir target/portable-static`（VCRUNTIME140.dll依存なしを確認）→ `D:\00_WorkSpace\APP_KeyDeck01_portable\` へ exe＋static/keymaps/decks/surfaces/layouts/apps/schemas を配置（.bak除外）。旧9/5版は `D:\00_WorkSpace\_old\` へ移動。SSD直下に `D:\00_START_KeyDeck.bat` を追加。起動確認は未実施（ユーザーが出発前に実行）


## 2026-09-22 トラックボール Vol2（シンプル版）— Vol1.1 を凍結して作り替え

**ユーザー指示**: 「今のをVol1.1として保存して、シンプル設計のトラックボールを新調」。

- 凍結: 作り替える直前の `static/trackball.html` を `static/trackball_v1_1.html` に複製（冒頭に凍結の注記のみ追加）。`/trackball_v1_1` で単独で開ける（`ws.rs` にルート1行）。boardには埋め込まれない
- Vol2（`static/trackball.html`・boardの区画はすべてこちら）
  - 見た目: 青い球＋十字ライン。模様の点は既定0。設定の保存キーを `trackball02` に分けた（Vol1.1 の値を引き継がない）
  - 設定アイコンは球の画面の右上に重ねるだけ（帯・外枠なし）。領域の案内表示・下半分の帯・状態表示は出さない。※最初は右に56pxの帯を置いたが、その幅だけ球が中心から左へずれたため同日撤去（ユーザー指摘）
  - 操作は1本指だけ: 動かす＝移動／タップ＝`tap1-left`／ダブルタップ＝`dtap1`／タップ→すぐ触れて0.2秒置く or 動かす＝`tapdrag1`（左ボタン押しっぱなし、離すと離す）。2本目の指は無視（タップにも数えない）
  - 抜いたもの: 右クリック・Ctrl+C（2本長押し）・Ctrl+V（2本タップ）・スクロール（`tb01-scroll` を送らない）
- Hub 側 `surfaces/trackball.json` は無変更（Vol1.1 がそのまま動くように。Vol2 は既存のジェスチャー名だけを使う）
- 同日の前段: `tapdrag1` の追加、および Vol1.1 の不具合修正（右の帯のタップ直後に左へ触れると右クリックが左クリックに化けていた＝タップ確定時に sessionSide を読み直していた）
- 検証: 実マウス操作でタップ／ダブルタップ／タップ→ドラッグ／ただのドラッグ／右の帯のタップ／歯車の開閉／パネル外タップで閉じる、を確認。置いたまま0.2秒・2本指タップ・1本長押しは合成イベントでタイミングのみ確認。`cargo test --workspace` 全通過。**iPhone 実機は未確認**

- 2026-09-22 追記: **✋つかむ（トラックボール Vol2）**。タップ→押しっぱなしは実機で範囲スクショが一度も成功せず（間合い頼み）、外部の相談（reports/consult_20260922_trackball_drag.md）でも A案＝トグルを推された
  - 右下の ✋ を押すと（当初は歯車の下。同日ユーザー指定で右下へ） `grab1` down（`surfaces/trackball.json` に `grab1`＝mouse.button.hold left を追加）、もう一度で up。つかみ中は球で動かすだけ（タップ・ダブルタップ・タップ→押しっぱなしを出さない）。指を離して持ち直しても押されたまま。ふちが赤く光り「つかみ中」表示
  - 画面を閉じる/隠れる（pagehide・visibilitychange）→ up を送る。WS切断 → 画面の表示だけ解く
  - **Hub 側の穴を修正**: 切断時に押しっぱなしを離す `release_held_keys` はキー（key.hold）しか見ておらず、トラックボールで押したマウスボタンは切断しても押されたままだった。`state.rs` に `held_mouse`（note_mouse_hold / take_held_mouse）を足し、`handle_surface_gesture` で Windows へ送れた押下/解放を記録、切断時に離すようにした。テスト2件（押したまま切断→離す／自分で離したものは二重に離さない）
  - 検証: 実マウス操作で つかむ→ドラッグ2回（持ち直し）→タップ（クリックが出ない）→離す、離した後のタップ＝左クリック、を確認。WS切断は偽の接続で起こし、表示が解けることを確認。pagehide/visibilitychange での解放は未確認。`cargo test --workspace` 全通過（87＋42＋8＋7）。**iPhone 実機・範囲スクショでの成功は未確認**

- 2026-09-22 追記: **スクロール用ホイール（wheel_scroll）**。トラックボールとは別の部品
  - 新しい動作 `mouse.wheel {dir: up|down}`（proto-keymap の Action）。ホイール1段＝WHEEL_DELTA 120 を1回。量は固定で端末からは渡せない（`amount` 等の余計な欄は読み込みで拒否）。許可リストID `mouse.wheel:up/down`（state.rs `canonical_command_id`）、`fire_action` の送信経路、`/api/schema` の動作一覧、deck の検証、adapter の送出に追加
  - ダイヤル設定 `jog` に `shape`（dial/wheel、既定 dial）と `detentPx`（6〜80、wheel 用、既定18）を追加。`detentDeg` は省略可（既定15）に。既存のダイヤル JSON は無変更で読め、書き出しても欄は増えない
  - 見た目（components.js `renderWheel`）: 縦長の溝つきの筒。上下にこすると detentPx ごとに CW（指を下へ）/CCW（指を上へ）を1回押し、ダイヤルと同じ目盛り音を鳴らす。溝は指に密着（重みなし）
  - エディタ: ホイールの下限を端末ごとに持たせた（iPad 60×130px、スマホ 44×100px → どの端末も 1列×2行）。パレット名「ホイール」
  - 配置: `iphone7_portrait` の右端6〜7行目（SEC-WHEEL）
  - 検証: 単体ページで実マウス操作。下へ約58px→CW×3、上へ約58px→CCW×3、触れただけ→0。エディタで iPhone 縦 board が「重なり・はみ出しなし」。テスト（`mouse.wheel` の読み込み・`jog.shape` の既定と書き出し・実データで `mouse.wheel:up/down` が許可リストに載る）追加、`cargo test --workspace` 全通過。**音が鳴るか・実機でスクロールするかは未確認**

- 2026-09-22 追記: **アプリ切り替えの3ボタン（deck `apps`・アイコン付き）**。youtube01 の Win+5/6/8 の進化版
  - `decks/deck_apps.json`（3列×1行）: Chrome=WIN+5、Codex=WIN+6、Claude=WIN+8。Win+数字はタスクバーの並び順で決まるので、並びを変えたら action を直す
  - アイコン `static/icons/app_{chrome,codex,claude}.png`（256px）。この PC のアプリ本体から取り出した: Chrome=`VisualElements\Logo.png` を余白を詰めて、Codex=ストア版の白い絵柄をアプリ定義の色 #3143FF の角丸に載せて、Claude=ストア版 `Square150x150Logo.png`
  - Hub に `/icons`（ServeDir static/icons）を追加。それまでアイコン画像を配る道が無かった
  - **不具合修正**: deck のアイコン画像に大きさの指定が無く、元の寸法のまま出てボタンからはみ出し、真ん中だけ拡大されて見えていた（アイコン付きボタンを実際に置いたのはこれが初めて）。アイコンのあるボタンだけ `has-icon` を付け、画像をボタンに収めて下にラベルを残すようにした。アイコンの無い既存のボタンは見た目が変わらない
  - 配置: `iphone7_portrait` の8行目・左3マス（SEC-APPS）
  - 検証: 実際の描画処理で iPhone 縦の3マス（282×62px）と iPad の3マスに描き、アイコン・ラベルが収まることを目視。実マウスで3つを押して A1/A2/A3 が出ること。実データのテストで deck `apps` が読め、WIN+5/6/8 が許可リストに載ることを追加。**実機でアプリが前に出るかは未確認**
  - 同日追記: deck `apps` を4ボタンに（先頭にフォルダー＝WIN+1。アイコンは explorer.exe から256pxで取り出した `static/icons/app_explorer.png`）。Codex のアイコンの背景を青→黒（暗いボタンに溶けないよう薄い縁取り）。区画 SEC-APPS を 8行目の4マスへ
- 2026-09-22 追記: **トラックボール Vol2 に右クリック「R」**。右端の真ん中（歯車=右上 と つかむ=右下 の間）。押すと `tap1-right`（surfaces/trackball.json に既存。Hub の設定は変えていない）。設定パネルを開いたままでも押せる。検証: 実マウスで押して tap1-right が1回出ることを確認
- 2026-09-22 追記: **1マスのラジアルボタン（keymap `radial_edit`）**
  - keymap に `radial: { label }` を足すと、キーボード部品が1つのボタンとして描かれる（proto-keymap `RadialConfig`。盤面に N/E/S/W の4キーが必須、jog との併用は拒否）。押すと指の位置を中心に上下左右4項目が開き（body 直下に position:fixed で重ねる＝周りの区画の上に出る）、指を置いた点から22px以上倒した方向を白く光らせ、離すとそのキーを down→up。真ん中で離す・pointercancel は何も送らない。画面の端では項目が切れないよう表示の中心だけ内側へ寄せる（方向の判定は指の位置のまま）
  - キー編集画面（editable）では普通の4キーの盤面のまま（中身を差し替えられるように）。keymap の保存は JSON の board だけ差し替えるので radial は消えない
  - 中身（layer0）: 上=コピー、右=貼り付け、下=元に戻す、左=切り取り（仮。キー編集で差し替え可）
  - エディタ: 下限は 44×44px（どの端末でも1×1）。配置は `iphone7_portrait` の6行目・1列目（SEC-RADIAL）
  - **エディタの不具合修正**: Deck の下限に常にページ送りの帯（26px）を足していたため、1ページしかない Deck（apps の4ボタン、1行）が「最小サイズ 1×2 を下回る」と誤って赤くなっていた。帯は2ページ以上のときだけ足すようにした
  - 検証: 実マウスで上/右/下/左へはらい N/E/S/W の down→up が1回ずつ、動かさずに離すと何も送らない、を確認。開いた状態は合成イベントで作って撮影（倒した方向が白く光る・周りの区画の上に出る）。離して決定＝E、外へ倒して真ん中へ戻して離す＝取消 も確認。全 board の判定（ipad_youtube の SEC-CLICK-L＝mouse_left の件のみ残る）。`cargo test --workspace` 全通過。**iPhone 実機は未確認**
- 2026-09-22 追記: `iphone7_portrait` を 4×8 → **5×8** に（ユーザー指示）。1行目＝範囲スクショ(2)・右クリック(2)・ラジアル(右端)、2〜6行目＝トラックボール(横いっぱい)、7〜8行目＝アプリ4ボタン(左4列・縦2)・ホイール(右端・縦2)。40マスすべて使用。1マス約75×62px。エディタの判定は「重なり・はみ出しなし」
- 2026-09-23 追記: **二層ラジアルメニュー（階層型パイメニュー・keymap `radial_edit2`）**。ユーザーが見せたゲームUI（外周＝大分類・内周＝細かい操作）の再現
  - `RadialConfig` に `sectors`（4/6/8、既定4）と `rings`（1/2、既定1）を追加。方向idは Rust の `RADIAL_DIRS_4/6/8`（8方向は N NE E SE S SW W NW）、内周は方向のうしろに `2`（`N2` など）。`radial_key_ids(sectors, rings)` が盤面に要るidを出し、読み込みで揃っていなければ落とす。既定のままなら JSON に欄は増えず、既存の `radial_edit`（4方向・1層）は無変更で読める
  - 見た目（components.js `renderRadial` を書き直し）: 扇形を SVG で描き、名前だけ HTML を上に重ねる（SVG の text は折り返せないため）。中心＝取消の丸、内周は外周より一段暗い。選ばれている扇は白抜き。**決まる名前はメニューの上（余白が無ければ下）に大きく出す**（指が扇を隠すため）。画面が狭ければ全体を縮める
  - 決め方は2つの物差しだけ: 角度＝どの扇か／距離＝中心は取消・内周・外周。外へ出しすぎても最後の扇を保つ
  - **暴発の修正**: 画面の端のマスだとメニューは内側へ寄って開くので、押した指は最初からどれかの扇の上に乗る。そのまま決まると「触っただけで発動」になるため、12px 動かすまでは何も選ばない（GRACE）。動かす向きは常に画面の内側なので、端のマスでも全部の扇に届く
  - 中身が空の扇は暗く出して「空き」と表示し、離しても何も送らない
  - 中身（radial_edit2 layer0・16項目）: 上 コピー/切り取り、右上 貼り付け/書式なし、右 元に戻す/やり直し、右下 保存/別名保存、下 検索/置換、左下 全選択/削除、左 閉じる/タブ復活、左上 新規/開く
  - エディタ: パレット名を「ラジアル二層 8×2」「ラジアル 4方向」のように出す。下限は 44×44px のまま（選択肢は区画の外へ広がるので、必要な大きさは項目数と関係しない）
  - 配置: `iphone7_portrait` の1行目・右端（SEC-RADIAL）を `radial_edit` → `radial_edit2` に差し替え。`radial_edit` はそのまま残す
  - 検証: iPhone 7 縦と同じ 375×497px の格子に実際に置き、実マウスのドラッグで 外周上＝N、内周上＝N2、内周左下＝SW2、外周右上＝NE が down→up 1回ずつ。動かさず離す＝何も送らない、中心で離す＝何も送らない。離した瞬間の見た目を写して、白く光る扇・上下に出る名前・取消の丸を確認。メニューの後始末（body に残らない）も確認。テスト（`sectors`/`rings` の既定と読み込み、`radial_key_ids` の網羅と不正値、実データで radial_edit2 が16キーで読め CTRL+SHIFT+V / CTRL+Y / CTRL+SHIFT+S / CTRL+H / CTRL+SHIFT+T / CTRL+O / DEL が許可リストに載る）追加、`cargo test` 全通過。**iPhone 実機は未確認**

## V2.1（2026-09-23〜）— Deck編集の追加とUIの作り替え

- **View_Ver1.1 として凍結**: `static/editor_v1_1.html`（レイアウト編集）・`static/keys_v1_1.html`（キー編集）・`static/components_v1_1.js`。URL は `/editor_v1_1` `/keys_v1_1`。**描画も同じ日の写しを指す**ので、生の components.js を変えてもこの2画面は変わらない（trackball_v1_1 はライブの components.js を読んでいて、そこだけ凍結が甘かった。今回はそれを踏まえた）。ナビに「保存版（View_Ver1.1）」の項を足した
- **Deck編集（`/deckedit`・`static/deckedit.html`）**: タイルのラベル・地色・アイコン・動作を画面から変える。左に盤面（実機と同じ `renderDeck` で描く）、右に「選んだタイル」だけを出す2列。Deckの新規作成・削除・ページ追加はしない（ファイルの増減を伴うため）
  - 1ページのタイルは格子の枚数ちょうどに揃え、**空きも1枚（動作 none）として並びに残す**。残さないと、途中を空にしたとき後ろが前へ詰まって位置がずれる。保存時は末尾の空きだけ落とす
  - 格子を小さくするとき、あふれるタイルの枚数を先に数えて止める（黙って消さない）
- **`POST /api/deck/save`**（ws.rs `deck_save_handler`）: 書き先は `decks/deck_<id>.json` に固定。`deckId` はレイアウトidと同じ `[a-z0-9_]{1,64}` で、**パスを組み立てる前の唯一の関門**。`.bak` 退避 → 書く → **全体を読み直す** → 落ちたら巻き戻す、はレイアウト保存と同じ手順。説明文は本文に無ければ既存を引き継ぐ（画面はタイルしか触らないため）。エラーコード `DECK_SAVE_REJECTED` / `DECK_SAVE_FAILED`
- **タイルの地色 `slot.color`**（deck.rs）: `#rrggbb` だけを通す。CSSへそのまま入る値なので、読み込みが唯一の関門（`is_hex_color`）。省略した既存のDeckは無変更で読め、書き出しても欄は増えない。見た目だけの項目で、何が起きるかには関わらない
  - components.js の `renderDeck` が地色を塗り、`.tinted` で文字を白＋影にする（色の上では細い灰色の文字が読めないため）。**空きタイルには塗らない**（「置ける場所」と「置いてある物」を色で分ける）
- **`GET /api/icons`**: `static/icons/` の画像名を名前順で返す**読み取り専用**API。Deck編集がアイコンを選ぶための一覧
- **部品カタログを components.js へ集約**: `buildActionParts(config, keymapId)` / `actionTabs` / `layerName`。キー編集と Deck編集が同じ一覧を出すため、写しを置かない（AGENTS.md「分裂させない」）。keys.html からは101行を削ってこれを呼ぶだけにした
- **不具合修正（既存）**: キー編集の部品一覧に `HOME` `END` `PGUP` `PGDN` `DELETE` が並んでいたが、5つとも `VK_DICTIONARY` に無く、**選んで保存すると LOAD_VK_UNKNOWN で弾かれていた**。HOME/END/PGUP/PGDN を辞書とアダプタ（0x24/0x23/0x21/0x22）に足し、`DELETE` は辞書にある `DEL` に直した。アダプタの網羅テストが辞書の追加を検査している
- 検証: 本物のHub（8770・ユーザーが使用中）には触らず、`static/` と WS の surface.config を出すだけの偽Hubを別ポートに立てて実マウスで操作。Deck編集で apps を開く→タイルを選ぶ→地色を選ぶ→保存、まで通し、**送られたJSONを取り出して本物の Rust の読み込みへ通した**（説明文が残り、色が付いたタイルだけに増え、他は無変更）。`cargo test` 全通過（91＋46＋8＋7）。**実機は未確認。Hub の再起動が要る（Rustを変えたため）**
- 2026-09-23 追記: **ゲーム部門**（ナビの2つ目の部門。PC部門と道具＝編集画面は共通で、ここからは**ゲーム用の中身**へ直接入る。Loupedeck の Profile と同じ考え方。画面を増やさないので直す場所も1か所のまま）
  - `keymaps/keymap_game_action.json`（3列×2行）: 上段 決定・調べる E／ジャンプ SPACE／攻撃 左クリック、下段 ダッシュ SHIFT／しゃがむ CTRL／取消 ESC。ジャンプ・ダッシュ・しゃがむは `key.hold`（押している間だけ）
  - `decks/deck_game.json`（4列×2行・**地色つき**）: スクショ／録画（WIN+ALT+R＝Xbox Game Bar）／全画面 F11／窓の切替 ALT+TAB／ミュート／音量±／メニュー ESC。役目ごとに色を分け、押し間違えると困る録画だけ赤を強くした
  - 盤面3枚（移動は既存の `dpad01`＝WASD を流用、視点は `tb01`、武器切替は `wheel_scroll`）
    - `game_ipad`（iPad Pro 12.9 横・12×9）: 左上 道具Deck／左下 移動／中央 トラックボール（縦いっぱい）／右上 ホイール／右下 アクション。108マス全部使用
    - `game_iphone7_land`（iPhone 7 横・10×4）: 左 移動／中 トラックボール／右上 アクション／右下 道具Deck。40マス全部使用。**ホイールは入らない**（置くと1列も余らないため、iPad と縦画面にだけある）
    - `game_iphone7_port`（iPhone 7 縦・5×8）: 上3行 トラックボール（横いっぱい）／中3行 移動＋アクション／下2行 道具Deck＋ホイール。40マス全部使用
  - 検証: 3枚ともレイアウト編集で「✓ 重なり・はみ出しなし」。iPhone 7 縦の実寸（375×553）で実際に描き、地色つきタイル8枚・アクション6個・十字キー・ホイール・トラックボールが収まることを目視。実データのテストに `key.hold:SPACE/SHIFT/CTRL`・`mouse.click:left`・`chord:WIN+ALT+R`・`key:F11` などが許可リストに載ることを追加、`cargo test` 全通過。**実機は未確認**
- 2026-09-23 追記: `dpad01`（十字キー）の既定割当をWASD（vk W/A/S/D）→**矢印キー**（vk UP/LEFT/RIGHT/DOWN、D102/D201/D203/D302）に変更。`keymaps/layers/dpad01_layer0.json`と`keymaps/keymap_dpad01.json`の`description`もWASD前提の注記から矢印キー前提の文へ更新（vkは`dpad_arrows_layer0.json`と`VK_DICTIONARY`で実在確認済み）。labelとaction.t(`key.hold`)は無変更、D202は空きのまま。`cargo test --workspace` = **152 passed, 0 failed**（hub-core 7 + proto-adapter-win 8 + proto-hub 91 + proto-keymap 46）。既存テストの削除・弱体化なし。dpad01は`startup::discover_and_load_picks_up_arbitrary_new_keymap_files_without_hardcoding`が実データを動的に読み込む経路でロード確認済み（Hubサーバー自体は起動していない）
  - 同日追記: 他の説明文に残っていた「dpad01（WASD）」も矢印キーへ合わせた（`keymap_dpad_arrows.json`・`dpad_arrows_layer0.json`・`keymap_game_action.json`・`layout_game_ipad.json`・`layout_game_iphone7_land.json` の description のみ。vk・配置は無変更）
- 2026-09-23 追記: **レイアウト編集の新View `/editor_v2`（`static/editor_v2.html`）** を追加。旧 `/`（editor.html）は無変更で残す。メニューの PC部門に「レイアウト編集 V2（新View）」を登録（`components.js` NAV_TREE）、ルートは `ws.rs` に1行（ServeFile。保存は既存 `/api/layout/save` を通るので書き込み口は増えていない）。変えたのは左の「置ける部品」だけ: 幅190→300px・画面に貼り付けて中だけスクロール／種類チップ（件数つき）＋「未配置のみ」／種類別の見出し・未配置が先／カードに中身の実物ミニプレビュー（2.5倍で描いて0.4倍に縮小）＋id＋最小サイズ＋中身の文字（キーのラベル・Deckのボタン名）／検索は中身の文字にも当たり、当たった所に印／乗ると詳細（要る縦横比で描いた実物・説明文全文）／**カードを格子へドラッグして落とした場所に置ける**（緑=置ける・赤=重なり、重なりは置かずに知らせる）。クリックで空きへ置く従来動作も残す
  - 検証: `cargo test --workspace` = 152 passed。画面は稼働中Hubが旧ビルドのため、実データ（keymaps/decks/layouts）から組んだ模擬 surface.config を差し込んだ写しで確認: game_soranokiseki を開いて部品22件（キーボード14・リスト4・ダイヤル3・トラックボール1）表示／「esc」検索で youtube01 だけが残る／リスト絞りで4件／未配置のみで19件／game_action を重なる場所へドラッグ→拒否の知らせ、列を12にして空きへドラッグ→ (2,10) 3×3 に配置。**実Hubでの確認はHub再起動後に未実施**

## 2026-09-25 Deckの取り込み＋アイコン保存（外のアプリ→KeyDeck）— Claude Code（Opus 5.5）

- 発端（ユーザー要望）: kanban-note01（PWA）で作ったキャラクターの **ID・名前・四角アイコン** を、ボタン1つでHubのDeckに出したい。PWAから直接Hubへ送るのはCORS（Hubは別オリジンを許可していない）と混在コンテンツ（HTTPSのPWA→LANのHTTP）で不可のため、**ファイル受け渡し＋Deck編集の「取り込み」**にした
- **不変条件6に `static/icons/<name>.<ext>` を追加（2026-09-25・ユーザー裁定）**。CLAUDE.md に条件を明記
- **`POST /api/icon/save?token=&name=`**（新規 `crates/proto-hub/src/icon.rs`＋ws.rs `icon_save_handler`）: 本文は画像バイト列。name は `[a-z0-9_]{1,64}`、拡張子は先頭バイトで決める（PNG/JPEG/WebP）。**SVGは拒否**（同一オリジンで配られスクリプトを埋め込めるため）。1MiBまで。`.bak` 退避→書く→読み直して一致確認→不一致なら巻き戻し。エラーコード `ICON_SAVE_REJECTED`（422）/ `ICON_SAVE_FAILED`（500）。`/api/schema` の `ids.iconName` に規則を載せた
- **Deck編集 `/deckedit` に「取り込み」ボタン**: 形式 `keydeck.deck_import.v1`（`{format, deck, icons}`。slot の `iconRef` が icons のキーを指す）。画像を `/api/icon/save`、Deckを既存の `/api/deck/save` へ。参照先の無い iconRef・形式違いは**画像を1枚も書く前に**止まる。既存deckIdは上書き確認。保存後に届く surface.config でそのDeckを開く（応答より先に届く競合に備え、保存前に印を付ける）
- 外向け資料: `C:\00_CreatorCompass\KeyDeck\できること.md` を版2.2へ（§5 書き込み場所5か所・CORS不可の理由・取り込み形式、§7 実機未確認）
- 検証: `cargo test --workspace` = **158 passed, 0 failed**（hub-core 7 + proto-adapter-win 8 + proto-hub 97 + proto-keymap 46。新規6件: icon.rs 5件＝形式判定・name関門・保存とURL・上書き時の.bak・SVG/不正名/空/1MiB超を**ディレクトリすら作らず**拒否、ws.rs 1件＝token無し/不一致は401）。`cargo build` 警告0。取り込みの画面側は、fetch と WebSocket を差し替えた jsdom 上で deckedit.html を実際に動かし確認: 画像2枚→`/api/icon/save?name=chara_mia|chara_hero`、保存されたDeckの icon が `/icons/*.png` に置き換わり `iconRef` が残らない、deckSel が新Deckへ移り盤面にラベルが出る／iconRef欠落→通信0回で停止／形式違い→停止
- **未確認**: 本物のHubでの通し（AGENTS.md の規則とユーザー使用中の8770を止めないため、Hubは起動していない）。**Hubを再起動してから** `/deckedit` → 取り込み で確認すること

## 2026-09-28 Note Story 用の盤面 `note_story`（REQ-20260928-001）— Claude Code（Opus 5.5・統括チャット）

- 依頼: `C:\00_CreatorCompass\KeyDeck\依頼\REQ-20260928-001.json`（Note Story のメインチャットから）。本文の日本語はPCのキーボード、iPad は「話者・タグ/柱/記号・行の移動」だけを受け持つ盤面。**データの追加だけで、Rust の本体・書き込み口・通信は増やしていない**
- 盤面 `layouts/layout_note_story.json`（iPad 横・12×9・108マス全部使用）: 上段 話者Deck `note_cast`（8×4）／行の移動 `note_move`（2×4）／ホイール `wheel_scroll`（既存・2×4）、下段 書く道具 `note_tools`（6×5）／定型文 `note_palette`（4×5・list）／ふち `note_edge`（2×5）
- 新しい部品
  - `keymap_note_move`（1×3）: ALT+UP／CTRL+ENTER／ALT+DOWN
  - `keymap_note_tools`（3×2）: F13 [Char]／F14 [Scene]／F15 [演出]／text「柱：」／text「備考：」／F16 2カラム
  - `keymap_note_edge`（2×3）: CTRL+SLASH／CTRL+Z／ESC／F17 キー確認／`app.launch note_story`／`layout.switch`（id 省略＝既定へ）。**B-002 のため fire は付けていない**（起動と盤面移動は別ボタン）
  - `deck_note_cast`（5×2）: C1〜C9＝ALT+1〜9、C0＝ALT+0。**Note Story の書き出し（`keydeckExport.ts`）と同じ並び・同じ格子**にした置き場所の見本。Note Story で Deck名 `note_cast` として書き出し→Deck編集「取り込み」で人物名・アイコン付きに上書きされる（盤面を書き換えなくてよい）
  - `deck_note_palette`（list・10件）: 柱：／備考：／ナレーション：「」／[bg_town_night]／[bg_room]／# ／// ／{}／[]／()。すべて text
- `apps/apps.json` に `note_story`（`C:\Program Files\Google\Chrome\Application\chrome.exe` ＋ `--app=https://kanban-note01.vercel.app`）。exe の実在は確認済み。シェルは経由しない（不変条件7のまま）
- ナビ（components.js `NAV_TREE`）に「執筆部門（Note Story）」を追加: 盤面／話者Deck編集／定型文Deck編集／書く道具キー編集。`components_v1_1.js`（凍結）は触っていない
- 話者ラジアル（依頼の任意項目）は**入れなかった**。人物Deckはアイコンと名前が見えるが、ラジアルは開くまで誰が何番か見えず、取り込みで人物が入れ替わっても追従しない。両方置くと同じ ALT+n が2か所になるだけなので、まず Deck 1本で使ってもらう
- 検証: `cargo test --workspace` = **158 passed, 0 failed**（`startup` の実データテストに、note_story と5部品の存在・ALT+0〜9・ALT+UP/DOWN・CTRL+ENTER・CTRL+SLASH・F13〜F17 の許可リスト掲載を追加）。本物の Hub を再起動し読み込み成功（keymaps=22 decks=6 layouts=10 apps=2・許可リスト188）。レイアウト編集で「✓ 重なり・はみ出しなし」、端末画面 `/layout?id=note_story` を iPad 横の実寸 1366×1024 で描いて目視（押してはいない＝本物のHubなのでPCへキーが飛ぶため）
- **未確認**: iPad 実機・実際の Note Story（Chrome アプリ窓）での通し、取り込み（09-25 分）の実機通し。F13〜F17 は Note Story の次のデプロイまで Note Story 側で「押して登録」が要る
- 点検: データ追加のみ（Rust は startup のテスト1か所、static は NAV_TREE の1項目）のため keydeck-guardian は回していない

## 2026-09-29 実験: 1マス多重操作の判定ページ `/lab/multigesture`（REQ-20260929-001 段階1）— Claude Code（Opus 5.5・統括チャット）

- 依頼: `C:\00_CreatorCompass\KeyDeck\依頼\REQ-20260929-001.json`（Sub_KanbanNote 経由）。SpaceMouse の操作（パン・ズーム・回転・傾け・サイドボタン）を1アイコンにどこまで積めるか。**利用者裁定（2026-09-29）: 段階1＝見分けられるかを測るページだけを作る。部品化（段階2）は結果を見て決める**
- 依頼の形のまま（パンもズームも縦ドラッグ）では見分けられないため、指の本数と時間で分けた: 1本ですぐ動かす＝パン4方向／2本でつまむ＝ズーム／2本でひねる＝回転／0.5秒長押し→動かす＝傾け4方向／端の帯（22%）を動かさず短くタップ＝左右ボタン。2本指はつまみ量とひねり量を各しきい値で割った比の大きい方で決める
- `static/lab_multigesture.html`（新規・素のHTML+JS）: お題モード（取り違え表・混ざった組の多い順・正答率・判定までの時間の中央値）／自由モード／操作の種類ごとのオンオフ／マスの大きさ4種／しきい値7つ／JSON 書き出し。**WebSocket も fetch も使わない**。記録は端末の localStorage のみ
- `ws.rs` にルート1行 `/lab/multigesture`（ServeFile）。何も送らないページなので、書き込み口・通信は増えていない。ナビに「実験」部門を追加
- 検証: `cargo test --workspace` = 158 passed。本物の Hub から配信を確認。**PC の実マウス**でパン→・パン↑・左ボタン・右ボタン・なしの5回すべて正しく判定。2本指と長押しはマウスで作れないため**合成 PointerEvent** で判定ロジックだけ確認（ズームイン/アウト・回転↻↺・ひねり＋わずかな広がり＝回転・0.6秒押し→傾け←）。途中で、ブラウザが知らない pointerId に `setPointerCapture` が例外を投げて2本目が落ちる不具合を見つけ、try/catch で直した
- **未確認**: iPad での実測（受け入れ基準の本体）。返事は `依頼\REQ-20260929-001.reply.md`
- 2026-09-30 22:31 keydeck-guardian: P-008 段階A（e870e66以降の作業ツリー）点検 **PASS**。`cargo test --workspace` = 172 passed（hub-core 7 + adapter 8 + proto-hub 109 + keymap 48、+14）。凍結領域差分なし（brief/proposals の裁定追記のみ）。起動中Hub(8770・変更前バイナリ)で静的5ページ200・token無しWS/API 401。新バイナリでの実機通しは未確認 → `reports/guardian_20260930_2231.md`

## 2026-09-30 P-008 段階A 端末スロット — Claude Code（Opus 5.5・統括チャット）

- 設計: `brief/proposals/P-008_device_slots.md`（利用者の裁定: devices.json は手書き／既定は押した端末だけ・`"to": "all"` を必要なボタンに明示して付ける／レイヤーは端末ごとにキーマップを分ければ独立／新しい編集画面は Hub の中に作る／表示レイアウトは Hub に保存＝`views/` の書き込み裁定済み・未実装・不変条件6へは実装時に追記）
- 実装: 新規 `device.rs`（`devices/devices.json`・最大3台・id `[a-z0-9_]{1,32}`・無ければ空＝従来どおり）、`Action::LayoutSwitch` に `to`（`"all"` のみ）、接続ごとの device・端末ごとのいまの盤面（メモリのみ）・`broadcast_to_device`・`connection_url("device:<id>")`、`check_device`（未知の device は WS 確立前に 403 `WS_DEVICE_UNKNOWN`）、起動バナーに端末ごとの入口、`layout.html` が `?device=` と今の盤面を WS に載せる。見本 `devices/devices.example.json`。`/api/schema` の layout.switch に `to?`、`できること.md` 版2.5
- 検証: `cargo test --workspace` = **172 passed**（+14）、build 警告0。本物の Hub を一時の devices.json（見本の写し）で起動し、起動ログ `device slots loaded devices=3`・バナーに3台の入口。PC のブラウザ2枚（ipad／android1、どちらも note_story）で **android1 の「既定へ戻る」を実マウスでクリック → android1 だけ game_iphone7_port、ipad は不変**（ログ `this device only device="android1" sent=1`）。`device=android9` の WS は 403、`android2` は 101。確認後に devices.json は消した（端末の実物が未回答）
  - 注: guardian 報告の「起動中の Hub は変更前のバイナリ」は誤り。変更後にビルドした Hub を起動し、上の確認はそれで行った
- 未確認: 実機、`to:"all"` を付けた実物のボタン
- 既知の注意（guardian）: 端末の画面のセレクトで盤面を手で変えると、再接続まで Hub の「いまの盤面」が古い／reload・保存で消した盤面の id が端末の入口 URL に残り得る／token を持てば登録済みのどの端末でも名乗れる（名札であって認証ではない）
- **以前からの穴（guardian が発見・統括が再現）**: `/api/qr` は token を確かめずに token 入り URL の QR を返す（token なしで 200）。LAN 内の誰でも QR を読めば token が手に入る。直すには呼び出し元10画面に token を渡す必要があり、凍結中の `editor_v1_1.html` も含むため利用者の判断待ち

## 2026-09-30 `/api/qr` に token を必須にした（guardian が見つけた以前からの穴）— Claude Code（Opus 5.5）

- 穴: `/api/qr` は token を確かめずに token 入り URL の QR を返していた（token なしで 200 を再現）。同じ Wi-Fi の誰でも QR を読めば token（編集もできる鍵）が手に入った
- 利用者裁定 A（2026-09-30）: token を必須にし、QR を出す**11画面**すべてに token を渡す。**凍結中の `editor_v1_1.html`・`trackball_v1_1.html` も1行だけ足す（凍結の例外。該当行にコメントで明記）**
- 変更: `ws.rs` の `QrQuery` に token・`qr_image` の入口で `token_ok`（無い・違うは 401 `WS_TOKEN_INVALID`）。JS で作る画面（editor / editor_v2 / editor_v1_1 / gallery / kb / settings）は URL に `&token=` を足した。HTML に直接書いていた画面（ipad / layout / panel / trackball）は `data-qr` にして、token を読んだ直後に src を入れる。trackball_v1_1 は token の行に1行で上書き
- 検証: `cargo test --workspace` = **173 passed**（+1 `qr_requires_token`）。再起動した本物の Hub で token なしの `/api/qr` が 401。ipad / layout / panel / trackball / trackball_v1_1 の QR 画像が token 付きで 225px に読み込まれる。QR ギャラリーでカードの「QR」を実マウスで押して token 付きで表示。deck / kb-left / layout:note_story を token 付きで取ると 200

## 2026-09-30 統合編集画面（studio）段階1 — 新しいトップ `/` — Claude Code（Opus 5.5）

- 利用者: 「昔の UI は一掃していいから、新しいモックの UI で再スタート」。土台は `00_勉強フォルダー/41_統合編集画面_VIA型モック_v0.4.html`（v0.4.1）。**旧画面は消さず**、新しい画面にまだ無い機能（区画の並べ替え・キーの位置・Deck の色/アイコン/取り込み）のために歯車の「旧画面」から開けるようにした（旧レイアウト編集は `/` → `/editor`）
- 端末: `devices/devices.json` を置いた（iPad Pro＝tablet 横・既定 note_story ／ Pixel 6a＝phone 縦・既定 game_iphone7_port。iPhone は後で3台目）
- Hub: `GET /api/devices`（P-008 段階B の一部・読み取り専用・token 必須）、`state.device_connections`、`/` を `static/studio.html` に、`/editor` に旧画面。起動バナーの文言
- `static/studio.html`（新規・素の HTML/JS・components.js を使う）: 端末を1台／並べて（タブレット中央・携帯左右）、実寸で描いて縮小（iPad 1366×980・Pixel 412×840）。キー・ダイヤルの CW/CCW・Deck のタイルを押して選ぶ→左の部品棚（buildActionParts）か下の棚から入れる／表示名を変える。空欄・▽素通し（L0 では不可）。元に戻す Ctrl+Z／やり直す Ctrl+Y。保存はキー＝`/api/layer/save`・Deck＝`/api/deck/save`（既存の口だけ）→ 各端末に黒＋その端末専用 QR。LAYER タブ、携帯は L0・L1 を積む表示、フォーカス（狭い画面では自動で切る）、NAV の並び替え、表示レイアウト・NAV の並び・質感は**このブラウザに保存**（Hub の views/ は次の段階）。色相は `/api/theme`、再読込は `/api/reload`
- 検証: `cargo test --workspace` = **174 passed**（+1 `/api/devices`）、警告0。本物の Hub で2台（Pixel 6a・iPad Pro）が実物の盤面で描かれる。**実マウス・実キーボード**で: [Char] を選ぶ→表示名を打つ→Ctrl+Z で戻る→Ctrl+Y→保存 → Hub が配り直した構成に新しい名前、QR（token 付き）が出る。確認後にファイルを git から戻して `/api/reload`。[Scene] に部品「A」→Ctrl+Z で F14 に戻る。Deck のタイル（note_cast C3）を選べる
- 直したこと: 縦リストの Deck に正方形用の fitDeck を当てて幅48pxに潰れていた／800px 幅でフォーカスが図を点にしていた（空きが 420px 未満ならフォーカスしない）
- 未確認・未実装: 実機、キーテスター（端末で押したキーを光らせる）、トラックボールのジェスチャー編集、区画の編集、表示レイアウトの Hub 保存（views/・不変条件6へは実装時に追記）

## 2026-10-01 B-002 修正＋ Note Story 起動の Hooks（REQ-20261001-001）— Claude Code（Opus 5.5）

- **B-002 修正**: 起動時の許可リスト作りで入れ子 fire を全部たどる（`startup.rs` の `with_nested`。tg.fire・layout.switch・app.launch）。テスト `nested_fire_of_every_kind_reaches_the_allow_list` で3種すべてを固定
- **読み込みの決まりを広げた（利用者裁定 2026-10-01）**: app.launch の fire に限り、fire を持たない layout.switch を許す（proto-keymap `check_nested_fire`）。盤面移動は画面の話で PC へは何も送らない。入れ子は2段まで
- **塞いだ穴**: Deck の app.launch の fire を何も検査していなかった（deck.rs）。キーボード側と同じ規則に
- **読み込み時の確認を追加**: layout.switch の移る先が実在する盤面か（入れ子も含む）
- データ: `apps.json` の note_story を PWA の `--app-id`（chrome_proxy.exe・Default）に。`appswitch` に A3「Note Story 起動」（app.launch＋fire layout.switch note_story）、board を3列に
- 実測: `--app-id` で2回起動すると Note Story の窓が 1→2→3 と増える（manifest に launch_handler が無い）。案は Note Story 側の `launch_handler: { client_mode: "focus-existing" }`（返事に記載）
- 検証: `cargo test --workspace` = **178 passed**（+4: B-002・fire の決まり・Deck・未知の盤面）。本物の Hub で iPad を名乗る画面の A3 を実マウスで押す → `app launched` → `layout switch; this device only device="ipad"`、iPad だけ note_story へ、Pixel は不変
- 同時に: 利用者のテスト保存で変わっていた `game_action` GA2（ジャンプ→N）と `note_tools` T1（[Char]→Y）を 22e8bc0 の内容へ戻した（実データのテストが GA2 の変化を検出した）
- 未確認・未決: iPad 実機。「既定へ戻る」の行き先（iPad の既定が note_story なので戻っても note_story のまま。返事で A/B を質問）
