# keymaps/layers/ — レイヤー別JSON置き場（v0.3 D13）

- 1レイヤー=1ファイル。書式は `schemas/layer.schema.json`。
- Export: `GET /api/keymap/{keymapId}/layer/{n}/export`
- Import: このフォルダへファイルを置いて（または上書きして）Hub再起動。
- T7でSonnetが作るもの:
  1. 既存 `keymap_default.json` / `keymap_writing01.json` のlayers配列をここへ分割移行
     （`default_layer0.json` 等。マニフェスト側は`layerFiles`参照に書き換え、旧インライン形式は廃止）
  2. `ipad01_layer0.json` / `ipad01_layer1.json`（T8。配置の正=brief/ref_ipad_keyboard_parts_v1.md）
- 検証は全ファイル読込後に結合して従来どおり（Layer0必須・L0にtrans禁止・vk辞書・mo/tg/tg.fire参照先）。

## ipad01_vol12 のレイヤー構成（T21時点）

| # | ファイル | 役割 | 入り方 |
|---|---|---|---|
| 0 | `ipad01_vol12_layer0.json` | 基盤 | 常時 |
| 1 | `ipad01_vol12_layer1.json` | Fn（F1〜F12） | K101 `mo` 押している間 |
| 2 | `ipad01_vol12_layer2.json` | 記号盤 | K513 `tg` |
| 3 | `ipad01_vol12_layer3.json` | 日本語モード表示 | K511 `tg.fire`（同時にPCへALT+GRAVE） |

**レイヤー番号は「大きい方が勝つ」（D3）。** layer3はlayer2より強いので、
layer2が定義済みのキーをlayer3に足すと記号盤が壊れる。layer3を編集するときは
`ipad01_vol12_layer3.json` の description に書いた注意書きを先に読むこと。
