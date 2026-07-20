# 設計指示 v0.5 — キーボードフォーマットをPC側で自由に変更できる仕組み

作成: FABLE（2026-07-20） ／ 実装: Sonnet ／ 検証: guardian＋Opus ／ 曖昧さはSR起票

## 原則（これが本設計の全て）

**フォーマットの正は常にJSONファイル**（`keymaps/keymap_<id>.json`＝盤面グリッド ＋ `keymaps/layers/<id>_layer<n>.json`＝キー割当）。
GUI・API・将来のエディタは全て「このファイルを読み書きする道具」にすぎない。コード変更・再ビルドは永久に不要（G4で実証済み）。
これにより: どのAI・どのツール・手作業のどれで編集しても結果は同じ場所に集まり、Git履歴＝フォーマット変更履歴になる。

## 編集手段の3段階（AはすでにできるB→Cの順に実装）

### 段階A（実装済み・今日から使える）: JSON直編集
1. PCで `keymaps/layers/ipad01_vol12_layer0.json` 等をエディタで編集（ラベル・action・レイヤー）。盤面の形は `keymap_ipad01_vol12.json` の board（row/col/colSpan/rowSpan）
2. Hub再起動（start_hub.cmd）→ 反映。不正JSONは起動時にcode＋causeで拒否されるので壊れたまま動くことはない
3. 書式の辞書: `schemas/keymap.schema.json`・`layer.schema.json`、action種はD4/D20（key/chord/mo/tg/trans/none/text/keymap.switch/reset）

### 段階B（今回実装する最小完成形）: 再起動不要の反映＋フォーマット自動発見
- **B1 keymapsディレクトリスキャン**: Hub起動時に `keymaps/keymap_*.json` を全ロード（現在の3つ固定を廃止）。新フォーマットはファイルを置くだけで設定画面・ランディングに自動で並ぶ
- **B2 `POST /api/reload`**（token必須）: ディスクからkeymaps/decksを再読込→検証→成功時のみ差替→全クライアントへ`surface.config`再配信。失敗時は現行構成を維持しerrorフレーム（D9）。設定画面に「再読込」ボタン
- 運用: PCでJSON編集→ブラウザで再読込ボタン→端末に即反映（Hub再起動不要）

### 段階C（将来・T9）: VIAL型GUIエディタ
- モック済み（screen_mock_v0.4.html ②）。上=対象キーボード・下=割当パレット・マクロタブ枠
- 書込はD22経路のみ: スキーマ検証→既存ファイルを`.bak`→`keymaps/layers/`配下へ書込→reload。任意パス書込API禁止（CLAUDE.md）

## 拡張規則（新フォーマットを足すとき）

1. `keymap_<新id>.json`＋`layers/<新id>_layer*.json` を作る（既存をコピーして改名が最短）
2. kindは split（halves）か single（board grid）。新kindが必要になったらSR起票→設計判断
3. Vol凍結: 使える状態になったフォーマットは `keymap_<id>_volNN.json` として複製凍結し、作業は次Volで（D10の一般化）

## 実装タスク（仕上げフェーズF。この順で）

| F# | 内容 |
|---|---|
| F1 | **ズレ点検・修正**: `/ipad`実装とモックv0.4①の差分を列挙し（配色トークン値・グリッド配置・ヘッダ構成）、実装側をモックに合わせて修正。差分一覧をDEVBOARDに記録 |
| F2 | B1 keymapsディレクトリスキャン（設定画面・ランディング・QR targetが自動追従） |
| F3 | B2 `/api/reload`＋設定画面の再読込ボタン |
| F4 | guardian点検→`cargo test`全pass→コミット。README「フォーマットの変え方」節を段階A/Bの手順で更新 |

禁止: hub-core/brief変更・アップロードAPI・PWA/フレームワーク（CLAUDE.md）。検証: reload後に既存接続の表示が新JSONに変わること＋不正JSONでreloadしても現行構成が生き残ること。
