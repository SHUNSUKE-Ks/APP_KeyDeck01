# 設計指示 v0.2 — APP_KeyDeck01（Stream Deck面＋2台分割レイヤーキーボード面の原型）

作成: FABLE（2026-07-19） ／ 実装: **Sonnet** ／ 検証: **Opus** ／ 曖昧さ発見時: `brief/spec_return_log.md` へSR起票して停止
v0.1からの変更: 独立アプリ化（APP_ControlDeckから分離）、ログ/例外設計（D9）、キーマップState default/writing01（D10）、DeckセットリストImport/Export＋スキーマ（D11）、骨組みファイルをFABLEが事前作成（各ファイル内コメントが実装指示の一部）

## 位置づけ（正の宣言）

- **CODEXのAPP_ControlDeckとは別アプリ**。相互に参照も書き込みもしない
- 要件の参照元: PM削減前のv0.1要件（APP_ControlDeck/brief/初期案_v0_1/。読み専用の歴史資料）。MIDIは捨てる
- `crates/hub-core` は検証済みcrateの**vendoredコピー（2026-07-19時点スナップショット・凍結）**。変更禁止
- アーキ: PWA廃止。頭脳（キーマップ解決・レイヤー状態・許可リスト・SendInput・エラー整形）は全てRust Hub。ブラウザは押した位置IDを送り、受けた表示を描くだけ

## 0. ゴールの受け入れ基準（G1..G7）

- **G1（タイプ実証）**: `cargo run -p proto-hub` → ブラウザ2窓 `/kb?half=left` `/kb?half=right` → タップでメモ帳に実文字（左右にまたがる "hello"）
- **G2（レイヤー同期）**: 左でMO(1)保持→**両窓**の表示がLayer1に変化、離すと戻る。TG(2)はトグル。反映500ms以内
- **G3（Deck面）**: `/deck` の消音タップでWindowsミュート実トグル。音量±・メディアキーも発火
- **G4（データ駆動）**: `keymaps/`・`decks/` のJSON編集→再起動だけで配置・ラベル・レイヤーが変わる（コード変更ゼロ）
- **G5（回帰ガード）**: proto-keymap単体テスト12件以上pass、解決結果は決定的
- **G6（State切替）**: Deckの切替ボタンで keymap_writing01 ⇄ keymap_default が切り替わり、両kb窓へ即反映。resetでdefaultに戻る
- **G7（診断可能性）**: 誘発した各エラー（不正JSON・未知vk・token不一致・未知keyId）につき、**原因コード＋cause付きのログがちょうど1行**、Hubコンソールとブラウザコンソールに出る（D9書式）

## 1. 設計判断（D1..D11。Sonnetは再解釈禁止）

- **D1 独立ワークスペース**: 本フォルダ単独でcargo workspace。外部への依存はvendored hub-coreのみ
- **D2 端末=素のブラウザ**: PWA機構・フロントフレームワーク・ビルドツール禁止。静的HTML+素のJS各1枚
- **D3 レイヤー意味論（VIAL簡約）**: Layer0が常に最下層。`MO(n)`=押下中のみ有効、`TG(n)`=トグル。有効レイヤーの番号最大が優先、`trans`は下層へフォールスルー。レイヤー状態は**Hubが一元保持**し`layer.state`を全クライアントへブロードキャスト（2台同期の仕組み）
- **D4 アクション型**: `{"t":"key","vk":…}` / `{"t":"chord","keys":[…]}` / `{"t":"mo","layer":n}` / `{"t":"tg","layer":n}` / `{"t":"trans"}` / `{"t":"none"}` / `{"t":"keymap.switch","id":…}` / `{"t":"keymap.reset"}`。vk辞書: A-Z, 0-9, F1-F24, ENTER, ESC, TAB, SPACE, BKSP, DEL, UP/DOWN/LEFT/RIGHT, CTRL/SHIFT/ALT/WIN, COMMA, PERIOD, SLASH, SEMICOLON, QUOTE, MINUS, EQUALS, LBRACKET, RBRACKET, BACKSLASH, GRAVE, VOL_UP, VOL_DOWN, MUTE, MEDIA_PLAY, MEDIA_NEXT, MEDIA_PREV。**辞書外はロード時に起動拒否**（LOAD_VK_UNKNOWN）
- **D5 許可リストの正**: 起動時ロードしたJSON群に現れるアクション集合のみ実行可。クライアントは**位置ID（keyId/slotId）だけ**送信し解決はHub側 — 任意キー送信は構造的に不可能。実行経路はhub-core::CommandServiceを通し冪等・拒否を流用
- **D6 押下プロトコル**: kb→Hub `{"type":"key.press","keyId":…,"edge":"down"|"up"}`（key/chordはdown発火・up無視、MO/TGはdown/up使用）。deck→Hub `{"type":"deck.press","slotId":…}`。Hub→client: `surface.config`（接続直後＋keymap切替時: グリッド・ラベル・現在レイヤー・activeKeymapId）／`layer.state`／`error`（D9書式）
- **D7 SendInput**: `proto-adapter-win`に汎用実装（vk辞書→VKコード表、chordは修飾↓本体↓↑修飾↑、入力キュー直列）。本線adapter-windowsとは無関係
- **D8 トークン**: 起動時生成・stdout1回表示・URLクエリ・定数時間比較（本線D4の簡略流用）
- **D9 ログ・例外設計（新規）**:
  - **書式**: ブラウザ `[KD][<CHK|ERR>][コード] メッセージ {文脈}` を console.info / console.error。Hub側はtracingで `chk=/code=/cause=/context=` フィールド付き1行
  - **検証チェックポイント**: 実装タスクごとに `T1-1` 形式のID（各骨組みファイル内コメントに配置済み）。正常通過時 `[KD][T3-2][OK] …` を出す
  - **エラーコード**: `LOAD_JSON_SYNTAX` / `LOAD_SCHEMA_INVALID` / `LOAD_VK_UNKNOWN` / `LOAD_LAYER_REF_INVALID` / `WS_TOKEN_INVALID` / `WS_PARSE` / `KEY_UNKNOWN_ID` / `KEY_RESOLVE_NONE` / `ADAPTER_SENDINPUT_FAIL` / `KEYMAP_SWITCH_UNKNOWN` / `DECK_UNKNOWN_SLOT` / `INTERNAL`。追加はSR起票
  - **例外方針**: ランタイム入力起因でpanic禁止。各crateはthiserror的なエラーenum→Hubで捕捉→**原因コード＋causeを`error`フレームでクライアントへ返し**、ブラウザ側は受信したerrorフレームを必ずconsole.errorする（=エラーの原因がブラウザのコンソールで読める）。起動時ロード失敗のみ「原因を全部printして終了」
- **D10 キーマップState（新規）**: `keymaps/keymap_default.json`（初期値・リセット用、**今後も原則不変**）と `keymaps/keymap_writing01.json`（今後改造していく作業用）。**当面は同一内容**。Hubは両方を起動時ロードし `activeKeymapId` を保持（初期値: writing01）。切替はDeckのアクション`keymap.switch`/`keymap.reset`。切替時はレイヤー状態を0にリセットし`surface.config`再配信
- **D11 DeckセットリストImport/Export（新規）**: セットリスト=`decks/*.json`（スキーマ `schemas/deck.schema.json` でロード時検証）。**Export**: `GET /api/deck/export` が現在のセットリストJSONをダウンロード返却（読み取りのみで安全）。**Import**: `decks/` へファイルを置いて再起動（MVPではアップロードAPIを設けない — 書き込み面を増やさない）。keymapも同様に `schemas/keymap.schema.json` で検証

## 2. タスク分割（T1..T6。この順で。実装は全てSonnet）

各ファイルの骨組み（コメント＝詳細指示）は作成済み。**コメントの指示とチェックポイントIDを削らず実装で置き換えること。**

| T# | 内容 | 完了の目印 |
|---|---|---|
| T1 | `proto-keymap`: D3/D4の型・JSONロード＆スキーマ検証・レイヤー解決エンジン・テスト12件以上（チェックポイント T1-1..T1-4） | `cargo test -p proto-keymap` 12+ pass |
| T2 | `proto-adapter-win`: 汎用SendInput（T2-1..T2-2） | メモ帳smoke成功 |
| T3 | `proto-hub`: axum/WS・状態保持＆同期・D9エラー整形・D10切替・D11 export（T3-1..T3-5） | 起動して接続URL表示 |
| T4 | `static/kb.html`・`static/deck.html`: 描画・送信・errorフレームのconsole.error（T4-1..T4-3） | 2窓でG1/G2手動確認 |
| T5 | README整備＋Android実機手順＋G4確認 | G4/G6手動確認 |
| T6 | **Opus検証**: V1..V6→`reports/`へ検証シート→DEVBOARD記録 | G1..G7判定確定 |

## 3. 触ってよいファイル

| 区分 | 対象 |
|---|---|
| 変更可 | 本フォルダ配下すべて（骨組みの実装置換・テスト追加・DEVBOARD追記） |
| **禁止** | `crates/hub-core/`（vendored凍結）、`brief/`配下の設計書 |
| **禁止** | 本フォルダの外（**APP_ControlDeckには一切触らない**） |
| **禁止** | PWA機構・フロントフレームワーク・ビルドツール・アップロードAPI・任意vk受信・MIDI |

## 4. 検証項目（V1..V6。Opusが独立再実行）

- **V1**: `cargo test`（12+件）＋`cargo build`クリーン
- **V2**: G1手動E2E（2窓→メモ帳"hello"）
- **V3**: G2レイヤー同期（MO/TG、500ms以内）＋G6 State切替・reset
- **V4**: G3 Deck実発火＋`/api/deck/export`の返却JSONがスキーマ妥当
- **V5**: G7エラー誘発4種（不正JSON・未知vk・token不一致・未知keyId）→各1行・コード＋cause付きでHub/ブラウザ両コンソールに出ること
- **V6**: 本フォルダ外の無変更（git status）

---
**Sonnetへ: T1から本書と各ファイル内コメントの通りに。**
