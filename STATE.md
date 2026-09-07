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
- **2026-09-05 ✅ 実機確認**: ユーザーより「ipad成功です」。T29のキャッシュ修正で抜けた。
  **表示・動作は確認済み。使い心地（押した感・Vol1.3の打鍵感・十字キーの大きさ・textの着地）は未評価**
- **2026-09-05 追加(9)**: レイアウト面のヘッダに**切替セレクタ2つ**を追加（T30・未コミット）。
  BOARD=登録boardを丸ごと差替／LAYOUT=文字入力キーボードの配列だけ差替。**Hub側の変更ゼロ**
  （surface.configが全部配っているので端末の状態だけで完結）。選択はlocalStorageに保存
- **2026-09-05 追加(8)**: 端末が古い画面を出し続ける真因は**HTTPキャッシュ**だった（T29）。
  tower-httpの`ServeFile`は`Cache-Control`を付けないため、iOS Safariが独自判断で使い回し、
  **端末を再起動しても消えない**。ルータ全体に `Cache-Control: no-store` を付けて解決
- **2026-09-05 追加(7)**: 「PCでは変わるのに端末が変わらない」の原因は**端末が古いtokenを掴んだまま**だった（T28）。
  レイアウトごとにQRを出すようにし（ランディングページのカードが9枚に）、端末側は
  `/api/ping` で401を検出して**「tokenが古い」と明示**するようにした。`cargo test` = 119 passed
- **2026-09-05 追加(6)**: 十字キーを矢印版へ／押した感（押し込み0ms＋波紋）を実装／
  キーボードVol1.3（Gboard風・22列スタガード）を新設（T27・未コミット）。`cargo test` = 119 passed
- **2026-09-05 追加(5)**: P-005 段階B/C完了（T24・未コミット）。`layouts/`新設・`/layout`面・
  十字キー(`key.hold`＋切断時の強制解放)・トラックボールの部品化(iframe埋め込み)・`docs/HUB_MANUAL.md`。
  `cargo test` = 119 passed（113→+6）
- **2026-09-05 追加(4)**: P-005 段階A完了（T23・未コミット）。Deckを複数ロードできるようにし
  （`decks/deck_*.json` のディレクトリスキャン）、`deck.press`に`deckId`を追加、`render:"grid"|"list"`を新設。
  `deck_default.json`は縦2段（8×2）に、`deck_story_paths.json`（コピペリスト）を新規追加。
  `deck.grid.rows`が一度も使われていなかった欠陥も修正。`cargo test` = 113 passed（108→+5）
- **2026-09-05 追加(3)**: 実機指摘でipad01_vol12の配置を1列左へ寄せた（T22・未コミット）。
  Bkspと英数⇄日本語は1マス、旧p/旧？の位置は空白。keyIdも位置に合わせて付け替え（layer0とlayer2の両方）。
  凍結モックと食い違うため **SR-003** を起票（上記「未回答」0-a）
- **2026-09-05 追加(2)**: 実機iPad確認で見つかった不具合を修正（T21・未コミット）。
  「英数⇄日本語を押すとPCのIMEは切り替わるのに画面表示が切り替わらない」＝K511がただの`chord`で
  Hub側に状態が残らなかったのが原因。**D29 `tg.fire`**（レイヤー切替＋発火を1打鍵で）を新設し、
  日本語モード表示レイヤー(layer3)とバッジ長押し(H03=表示だけ直す)を追加。
  `cargo test --workspace` = 108 passed（99→+9）
- **2026-09-05 追加**: Stream Deck v2。見た目モック（`brief/mockup/screen_mock_streamdeck_v0.8.html`）＋
  提案書（`brief/proposals/P-003_streamdeck_v2.md`）＋**Ver1-a実装まで完了・未コミット**。
  新規`static/panel.html`（分割面。上=Deck四角スロット／下=13列キーボード、WSは1本・プロトコル追加ゼロ）、
  `static/deck.html`の四角スロット化（CSS中心）、`/panel`ルート＋QR target追加。
  `cargo test --workspace` = 99 passed（95→+4）。凍結領域への差分ゼロ。詳細はDEVBOARD末尾
- 気づいた副産物: `prototypes/capture_sequence/capture_sequence.py`という未追跡ファイルが作業ツリーにある。今回のセッションでは作成した記憶がなく、由来未確認（Browser pane検証時の副産物の可能性）。中身を確認してから要否判断すること

## 未回答・判断待ち【最重要】
0. **【新規・2026-09-05】P-003 Stream Deck v2**（`brief/proposals/P-003_streamdeck_v2.md`）。
   **Ver1-a（分割面 `/panel` ＋四角スロット化）は実装・検証まで完了・未コミット**。
   裁定済み: §7（`app.launch`/`shell.open`/`app.close` を防御A〜G条件付きで採用）・§11（段階分割、Ver1-aから着手）。
   **次に判断が要るのは Ver1-b に進んでよいか**（`multi` / `multi.toggle` / `chord.toggle` / `deck.page` / `web.open`）。
   Ver1-c（PC操作系＋登録簿）着手時に残りの未裁定4点を確認する:
   ①`web.open` の url 直書きという非対称を許すか ②`random` を採用するか（G5決定性の唯一の例外）
   ③登録画面をT9（VIAL型GUI）と同居させるか ④`icon` に絵文字を認めるか
0-00000. **【2026-09-05】キーボードは世代を分けて運用する**（T27）。
   現行= `ipad01_vol12`（凍結・数字段/Tab/Ctrl/Shiftあり）／新規= `ipad01_vol13`（Gboard風の日本語配列）。
   **既存盤面は編集せず複製する**運用（過去に「直したらレイアウトが崩れた」ため）。
   切替は `/layout?id=ipad_main`（Vol1.2）と `/layout?id=ipad_v13`（Vol1.3）でURLのみ。
   ⚠️ `IPAD_KEYMAP_ID = "ipad01_vol12"` が state.rs にハードコードされているので、**vol12は改名・削除できない**。
   十字キーは矢印版（`dpad_arrows`）を既定に変更済み
0-0000. **【2026-09-05・新規】技術書化の題名裁定待ち**（`docs/BOOK_REQUEST_CODEX.md` §0）。
   ユーザーは「Tauri＋Rust＋customkeyboard」としてCODEXに技術書化を依頼したいとのことだったが、
   **このリポジトリにTauriは1行も無い**（Cargo.toml/Cargo.lockに0件）。実体は axum+tokio+SendInput と
   素のHTML+JS。CLAUDE.mdはこの構成を確定アーキテクチャと定めD2でフレームワーク導入を禁じている。
   3案を提示済み: A=題名を実態に合わせる（推奨・今すぐ書ける）／B=最終部にTauri章を足す（先に実装が要る）／
   C=Tauriで作り直す（CLAUDE.md違反。提案書と裁定が要る）。**Aで進めるのが最も速く誠実**
0-000. **【2026-09-05】P-005 段階B/C 完了（T24・未コミット）**。
   `layouts/layout_*.json` で部品（keyboard/deck/trackball）を自由配置できるようになった。
   十字キー（`dpad01`・`key.hold`）とトラックボール部品を実装し、**手動入れ替えを実証済み**。
   **Hub操作マニュアル = `docs/HUB_MANUAL.md`**（コードを書かずJSONだけで変える手順）。
   `cargo test` = 119 passed。**次に効く未確認: 実機iPadでの操作感**。
   残る段階D（配置GUI）は不変条件6の条文更新（`layouts/`を書込許可に追加）が必要
0-00. **【2026-09-05・新規】音声Component（将来）— 実装以前に不変条件1に当たる**。
   ユーザー要望「iPad側で音声入力し、文字起こしをPCへペーストする」。**明示的にフューチャー扱い**。
   技術より先に効くのは governance: この部品は**クライアントが任意の文字列をHubへ送る**必要があり、
   CLAUDE.md 不変条件1「任意vk/文字列を受けるWS APIを追加しない」に正面から当たる。
   D28（トラックボール）が「上下限つき数値のみ」で例外を切ったのと同じ形の**正式な裁定が要る**。
   もう1つ: ユーザーはReactNative+Expoの経験ありで「iPadのローカルアプリでもよい」と言っているが、
   CLAUDE.mdは「React Native不使用＝確定アーキテクチャ。提案なしに変更してはならない」と定めている。
   iPad SafariのWeb Speech API対応状況は**未検証**（着手時に実機で確認すること）
0-000. **【2026-09-05】提案書の採番が別セッションと衝突した**。当初P-004で起票した
   「レイアウトと部品の三層化」を **P-005 へ改番**（別セッションが1分差で `P-004_ssd_field_deck.md` を
   同番号で起票していたため）。**`brief/proposals/` の採番は並行セッションで衝突する**ので、
   番号を取ったらすぐ空ファイルを置くこと。なお `P-004_ssd_field_deck.md` は別セッションの担当で、
   **P-005 段階Bと領域が重なる**（向こうの案1「/ipadのkeymap固定を外す」は、P-005のlayouts/が
   sectionごとにkeymapを指定できるようにすることで自然に解決する）。着手前に突き合わせること
0-0. **【2026-09-05・新規】P-005 レイアウトと部品の三層化**（`brief/proposals/P-005_layout_components.md`）。
   ユーザー要望「iPad面の空きに十字キーとコピペリストを置きたい／Deckは縦2段／スロット数をHub設定で変えたい／
   PCのStream Deckのように配置をパズルのように変えたい」＋設計質問「section/Component/スロットで大丈夫？」への回答。
   **§2/§3.1/§3.3/§4/§5は裁定済み。段階A（`grid.rows`の欠陥修正・Deck縦2段・コピペリスト・
   Deck複数ロード）は実装・検証まで完了（T23）。次は段階B（`layouts/*.json` ＋ `/layout` 面）**。
   調査で判明した既存の欠陥2件のうち①は段階Aで修正済み、②は段階Cで対応:
   ①~~`deck.grid.rows`は宣言だけで一度も使われていない~~ → 段階Aで修正
   ②`send_key()`がpress直後にreleaseするため押しっぱなしができない（十字キーに`key.hold`が要る）
0-a. **【2026-09-05】SR-003の裁定待ち: 新しい「配置の正」をどこに置くか**。
   ユーザー指示でipad01_vol12の配置を1列左へ寄せた（T22）。CLAUDE.mdが配置の正と定める
   `brief/mockup/screen_mock_v0.4.html` は凍結領域なので触っておらず、row2-5で実装と食い違っている。
   案A=実装（`keymap_ipad01_vol12.json`のboard）を正としモックは歴史資料として残す／
   案B=`screen_mock_v0.5.html` を新規作成して正を移す。**当面は案Aとして運用中**
0-b. **【2026-09-05】T21・T22の実機再確認待ち**。「英数⇄日本語」の表示切替を修正した。iPadは
   **ページ再読込だけでよい**（URL・tokenは変わらない。ただしブラウザキャッシュに注意＝上記「落とし穴」）。
   確認してほしいのは4点: ①配置が1列左に寄り、q/a/zが数字の1の真下に揃っているか（Bkspと英数は1マス）
   ②押すとバッジが「英数」⇄「日本語」に変わり盤面の地色が変わるか
   ③日本語のときSpaceに「変換」・Enterに「確定」が付き、「、。？」と「日本語」が大きく出るか
   ④タスクバー等でIMEを変えて表示がずれたとき、バッジ長押し(600ms)で表示だけ直せるか
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
  - **2026-09-05時点: Hubを起動したまま session を終えている**（`/panel` の実機確認用。不要なら閉じてよい）
- **2026-09-05 追加**: `static/*.html`を編集しても、ブラウザが古い版をキャッシュから返すことがある
  （`/panel`の検証中に実際に発生し、修正が効いていないように見えた）。検証時はURLにクエリを足して読み直すこと
- **2026-09-05 追加(2)・重要**: **D5の起動時許可リストは入れ子アクションを見つけられない。**
  `canonical_command_id`はtop-levelしか見ないため、`tg.fire`の中に入れたchordが許可リストに載らず、
  実行時に「absent from the startup allow-list」で**発火だけ**が拒否された（症状は
  「画面は切り替わるのにPCが切り替わらない」＝元の不具合の左右反転で、しかもHubログにしか出ない）。
  P-003 Ver1-b/1-cで`multi`/`logic`など入れ子アクションを足すときは、許可リスト構築を必ず同時に更新すること。
  回帰テスト2本あり（`canonical_command_id_descends_into_tg_fire` /
  `real_data_startup_allows_the_ime_toggle_chord_behind_tg_fire`）
- **2026-09-05 追加(8)・重要**: **「サーバは正しいのに端末が古い」を見たら `curl -D -` で配信ヘッダを見る。**
  `Cache-Control` が無い静的配信はブラウザに好き放題キャッシュさせている状態。
  tower-httpの`ServeFile`は既定で付けない（2026-09-05にルータ全体へ`no-store`を追加して解決）。
  **原因を1つ直して直らなかったら、別の原因が重なっていると考えること**（tokenとキャッシュの2つが重なっていた）
- **2026-09-05 追加(7)・重要**: **「PC側は変わったのに端末が変わらない」は、まずtokenを疑う。**
  Hub再起動のたびにtokenが変わる（D8）ため、端末が古いURLのままだと接続が拒否され続け、
  画面には前の内容が残るので「変わらないだけ」に見える。1秒で切り分けられる:
  `curl -o /dev/null -w "%{http_code}" "http://localhost:8770/api/ping?token=<端末のtoken>"`
- **2026-09-05 追加(6)**: **`animationend` だけでDOMを片付けてはいけない。**
  タブが非表示のあいだCSSアニメは進まず`animationend`が永久に来ないため、要素が溜まり続ける
  （波紋の実装で実際に9個溜まった）。`setTimeout`のフォールバックを必ず併用すること
- **2026-09-05 追加(4)**: **SendInputの着地はプレビュー環境では確認できない。**
  ページ内に`<input>`を作ってfocusしても（`document.hasFocus()`がtrueでも）文字は入らない。
  Hub側のログでエラー0件＝送出成功までは確認できるが、「意図した欄に入るか」は実機で見るしかない。
  検証で`text`アクションを撃つと**どこか別のウィンドウに文字が入る**ので、むやみに発火しないこと
- **2026-09-05 追加(3)**: **Bashツールのヒアドキュメントはバックスラッシュを畳む。**
  Pythonスクリプト内で `"\n"` と書いても Python には `
`（実改行）として渡り、JSONへ生の改行を
  書き込んでファイルを壊した（実際に1回やった）。`chr(92)` で組み立てるか、Write/Editツールを使うこと
- **2026-09-05 追加(2)**: **レイヤー番号は大きい方が勝つ（D3）**ので、ipad01_vol12でlayer3(日本語)に
  layer2(記号盤)と同じキーを足すと記号盤が壊れる。layer3を触る前に
  `keymaps/layers/ipad01_vol12_layer3.json`のdescriptionを読むこと

## 履歴
- `DEVBOARD.md`（時系列の全検証記録・決定事項ログ、本体）
- `reports/report_keydeck_v0.2_verification.md`（Opus独立検証シートの書式見本）
- `brief/spec_return_log.md`（SR-001裁定済み。今回のT15〜T19は設計書側の事前警告により新規SR起票なしで実装完了）
