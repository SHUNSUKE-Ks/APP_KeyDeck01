# 設計指示 v0.3 — 設定画面＋レイヤー別JSON＋iPad一枚キーボード（ipad01）

作成: FABLE（2026-07-20） ／ 実装: **Sonnet** ／ 検証: **Opus** ／ 曖昧さ発見時: `brief/spec_return_log.md` へSR起票して停止
入力: ユーザー要望（UpNote 2026-07-20）→ **配置の正は `brief/ref_ipad_keyboard_parts_v1.md`**（スクショはテキスト化済み。画像は開かない）
前提: v0.2はOpus検証で全合格（`reports/report_keydeck_v0.2_verification.md`）。本書はその上への増築。v0.2のD1〜D12は引き続き有効。

## 0. ゴールの受け入れ基準（G8..G11）

- **G8（設定画面＋単一QRポップアップ）**: `GET /` が設定画面になり、フォーマット一覧（分割左/分割右/ipad01/Deck）を表示。各行のQRボタン→**ポップアップで単一QRをでかでか表示**（1画面1QR＝誤読対策の本命）。ポップアップを閉じれば一覧に戻る
- **G9（レイヤー別JSON）**: キーマップは「マニフェスト＋`keymaps/layers/*.json`（1レイヤー=1ファイル）」に分割。`GET /api/keymap/{keymapId}/layer/{n}/export` でレイヤー単位ダウンロード可。Import=layersフォルダへ配置＋再起動（D11踏襲）。既存default/writing01も新形式へ移行
- **G10（ipad01面）**: `GET /ipad` でVer1.1裁定どおりの一枚キーボードが**画面下部に固定**表示され、ブラウザ（iPad 12.9横持ち相当の横長画面）から**文字が打てる**。上部ヘッダに「取り消し」「進む」ボタン（CTRL+Z / CTRL+Y）
- **G11（回帰）**: 既存の split(left/right)・Deck・レイヤー同期・エラー系がそのまま動作。`cargo test --workspace` 全pass（40+）

## 1. 設計判断（D13..D17。再解釈禁止。曖昧はSR起票）

- **D13 レイヤー別JSON（スキーマv2）**: キーマップ本体はマニフェスト化 —
  `keymaps/keymap_<id>.json` = `{ keymapId, kind: "split"|"single", halves|board, layerFiles: ["layers/<id>_layer0.json", …] }`。
  レイヤーファイル = `{ "layer": n, "keys": { keyId: {label, action} } }`。
  検証（Layer0必須・trans禁止 on L0・vk辞書・mo/tg参照先）は**全ファイル読込後に結合して**従来どおり実施。既存2キーマップ（default/writing01）も移行し、**旧インライン形式は廃止**（正は1箇所）。`schemas/keymap.schema.json`をv2に更新＋`schemas/layer.schema.json`新設
- **D14 ipad01の配置**: `brief/ref_ipad_keyboard_parts_v1.md` のA＋Bが正。裁定詳細:
  - kind="single"。`board.rows` はキーごとに幅係数 `w`（省略時1.0）を持つ: `[{ "id":"K101", "w":1.5 }, …]`。keyId は `K`+行+列2桁（例 K101）。幅は見た目の近似でよい（ぴったり再現不要。**後でユーザーがJSONを編集して調整するのが目的**）
  - 赤指定キー = `{"t":"none"}`（空ボタン・薄表示）。青指定: customFn01 = **MO(1)**（既存意味論）、🌐位置 = `CTRL`、全角位置 = **英数⇄日本語** = `{"t":"chord","keys":["ALT","GRAVE"]}`（MS-IME既定のAlt+半角/全角相当。効かない環境だった場合のみSR起票）
  - Spaceは幅を削る（例 w=3.0。周囲の空ボタンで行幅を埋める）
  - Layer1はプレースホルダ最小構成（数字段にF1〜F12等）。**テスト範囲は「文字が打てる」まで**（ユーザーが後で改造する土台）
  - vkは既存D4辞書の最近似でよい。JIS固有記号（@`や:*等）の物理位置ズレはMVP許容・SR不要
  - 画面: kb.htmlを流用拡張（`/ipad`ルート）。キーボードは`position:fixed; bottom:0`、ヘッダに undo(CTRL+Z)/redo(CTRL+Y)。レイヤー同期・切断表示・D9ログは既存のまま
- **D15 設定画面**: `/` を設定画面に置換（v0.2のQR3枚並びランディングは廃止）。一覧行=［フォーマット名｜kind｜QRボタン｜URL文字列］。QRボタン→モーダルポップアップに**そのフォーマット1枚だけ**のQR（`/api/qr`流用）を大きく表示。素のHTML+JS（D2維持）
- **D16 クリップボードboard（copy10件リスト）**: **保留**（ユーザー明記「初期未実装でもいい」）。future_ideas扱い。先回り実装禁止
- **D17 IDパターン**: keyIdの正規表現を `^[A-Z][0-9]{2,3}$` に拡張（L11/R45/K101が全部通る）。schemasも合わせて更新

## 2. タスク分割（T7..T10。この順で。実装は全てSonnet）

| T# | 内容 | 完了の目印 |
|---|---|---|
| T7 | **スキーマv2**: proto-keymapにマニフェスト＋layerFiles読込を実装（結合後の検証は従来関数を再利用）。既存default/writing01を`keymaps/layers/`へ移行。schemas更新。単体テスト追加（マニフェスト読込/レイヤーファイル欠損=LOAD_JSON_SYNTAX/結合後検証） | `cargo test` 全pass＋既存キーマップが新形式でロード |
| T8 | **ipad01**: `keymaps/keymap_ipad01.json`＋layers 0/1をD14どおり作成。kind="single"のレンダリング（w対応・下部固定・ヘッダundo/redo）を`/ipad`で配信。Hubのkeymaps一覧へ登録（activeとは独立に、surfaceごとに使うkeymapを選べるようにする: kb=active、ipad=ipad01固定でMVP可） | ブラウザで文字入力・レイヤー表示切替 |
| T9 | **設定画面＋QRポップアップ**: D15の`/`置換。`/api/keymap/{id}/layer/{n}/export`追加（G9） | G8/G9の手動確認 |
| T10 | **Opus検証**: V7..V10独立再実行→検証シート→DEVBOARD | G8..G11判定確定 |

## 3. 触ってよいファイル

| 区分 | 対象 |
|---|---|
| 変更可 | crates/proto-keymap・proto-hub、static/、keymaps/（layers/新設・移行含む）、schemas/、README、DEVBOARD |
| **禁止** | `crates/hub-core/`（vendored凍結）、`crates/proto-adapter-win/`（v0.2合格品。変更不要のはず。必要ならSR）、`brief/`配下 |
| **禁止** | 本フォルダ外への書き込み、PWA/フレームワーク/ビルドツール、アップロードAPI、クリップボードboardの先回り実装（D16） |

## 4. 検証項目（V7..V10。Opusが独立再実行）

- **V7**: `cargo test --workspace` 全pass＋新形式keymapのロード確認（default/writing01/ipad01の3つ）
- **V8**: `/ipad` で「文字が打てる」E2E（既存のWS注入方式で可）＋undo/redoボタン=CTRL+Z/Y発火＋赤指定キーがnone/薄表示
- **V9**: `/` 設定画面→QRボタン→**単一QR**ポップアップ表示。`/api/keymap/writing01/layer/0/export` がlayer.schema妥当なJSONを返す
- **V10**: 回帰 — split左右のレイヤー同期（v0.2 V3相当）とDeck発火が引き続き動作

---
**Sonnetへ: T7から本書と `ref_ipad_keyboard_parts_v1.md` の通りに。**
