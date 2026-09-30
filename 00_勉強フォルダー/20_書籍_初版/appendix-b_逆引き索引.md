# 付録 B　逆引き索引

**やりたいこと・知りたいことから、章・ファイル・関数へ飛ぶための表です。**
開発を引き継ぐ人や AI は、ここから入ってください。
ファイルの場所はリポジトリの中の相対パスです。行番号は付録 C にあります。

---

## 使う・変える

| やりたいこと | 章 | 開くファイル | 見る関数・項目 | 先に確認すること |
|---|---|---|---|---|
| キーの意味を変える | 3, 6 | `keymaps/layers/<名前>_layer<N>.json` | `keys.<位置>.action` | そのキーマップを他のボードでも使っていないか（9） |
| キーの位置を変える | 3 | `keymaps/keymap_<ID>.json` | `board.keys` | 同上。凍結した版なら複製する（14） |
| Fn のような「押している間だけ」の層を足す | 3, 6 | レイヤーの JSON | `{"t":"mo","layer":N}` | 番号の大きい層が勝つ |
| 決まった文字列を入力するボタン | 7 | レイヤーまたは Deck の JSON | `{"t":"text","string":"…"}` | 着地先の入力欄は確かめられない（付録 D） |
| ショートカットのボタン | 7 | 同上 | `{"t":"chord","keys":["CTRL","S"]}` | キー名は辞書にある 92 種類だけ |
| 押しっぱなしのキー（十字キーなど） | 8 | レイヤーの JSON | `{"t":"key.hold","vk":"UP"}` | — |
| ボードの部品を取り替える | 9 | `layouts/layout_<ID>.json` | `sections[].component` | 参照先の ID が実在するか（4） |
| 区画の大きさ・場所を変える | 9, 11 | 同上、または編集画面 | `row` `col` `colSpan` `rowSpan` | 重なり・はみ出しは拒否される |
| ボードを切り替えるボタン | 5, 12 | レイヤーの JSON | `{"t":"layout.switch","id":"…"}` | 切り替え先のボードが実在するか |
| アプリを起動するボタン | 12 | `apps/apps.json` とレイヤーの JSON | `{"t":"app.launch","id":"…"}` | `.exe`・絶対パスだけ。登録は PC で |
| トラックボールのジェスチャーの出口を変える | 10 | `surfaces/trackball.json` | `gestures` | — |
| ダイヤルの手ざわりを変える | 10 | ダイヤルのキーマップの JSON | `jog`（`detentDeg` 5〜90、`weight` 0〜95） | `CW` と `CCW` が盤面に必要 |
| テーマの色を足す | — | `crates/proto-hub/src/state.rs` と `static/components.js` | `THEMES` と `:root[data-kd-theme=…]` | **両方同時に** 足す。片方だけだと色が変わらない |

## 調べる・直す

| 知りたいこと | 章 | 開くファイル | 見る関数 |
|---|---|---|---|
| Hub が起動しない | 4 | コンソールの出力 | `load_startup_data` のエラー一覧 |
| 押したのに何も起きない | 5, 6, 7 | Hub のコンソール | `handle_key_press` の `key press … outcome=` の行 |
| 画面は変わるのに PC に入力が出ない | 7, 13 | Hub のコンソール | `absent from the startup allow-list` の行、`canonical_command_id` |
| PC のキーが押されたままになった | 8 | `crates/proto-hub/src/ws.rs` | `release_held_keys`、`note_key_hold` |
| レイヤーが戻らない | 6, 8 | 同上 | `resolve` の `Mo` の枝。切断しても `momentary` は残る（8） |
| 保存に失敗する | 11 | 編集画面の表示、Hub のコンソール | `layout_save_handler`、`layout_id_is_safe` |
| iPad が古い画面を出し続ける | 13 | `crates/proto-hub/src/ws.rs` | `router` の `Cache-Control: no-store` |
| 端末に何を送ってよいか | 2 | `crates/proto-hub/src/protocol.rs` | `ClientMessage` |
| 使える操作・部品・読み込み済みの中身の一覧 | 13, 14 | Hub 起動中の `GET /api/schema?token=<token>` | `schema_handler` |

## 境界に触れる前に

| しようとしていること | 読む章 | 必要なこと |
|---|---|---|
| 端末から新しい種類のメッセージを受け付ける | 2, 10 | 不変条件1。位置の形で表せないなら条件を書き、利用者の裁定 |
| 書き込める場所を増やす | 11 | 不変条件6。利用者の裁定 |
| 起動できるものの条件を緩める | 12 | 不変条件7。**緩めない** |
| `crates/hub-core/` を変える | 1 | 変更禁止 |
| テストを消す・弱める | 14 | 禁止 |
| 機能を足した | 14 | 同じ作業の中で `/api/schema` の辞書と能力の一覧を直す |
