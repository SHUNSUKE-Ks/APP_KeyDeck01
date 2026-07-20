# keymaps/layers/ — レイヤー別JSON置き場（v0.3 D13）

- 1レイヤー=1ファイル。書式は `schemas/layer.schema.json`。
- Export: `GET /api/keymap/{keymapId}/layer/{n}/export`
- Import: このフォルダへファイルを置いて（または上書きして）Hub再起動。
- T7でSonnetが作るもの:
  1. 既存 `keymap_default.json` / `keymap_writing01.json` のlayers配列をここへ分割移行
     （`default_layer0.json` 等。マニフェスト側は`layerFiles`参照に書き換え、旧インライン形式は廃止）
  2. `ipad01_layer0.json` / `ipad01_layer1.json`（T8。配置の正=brief/ref_ipad_keyboard_parts_v1.md）
- 検証は全ファイル読込後に結合して従来どおり（Layer0必須・L0にtrans禁止・vk辞書・mo/tg参照先）。
