# n07 — `crates/proto-hub/src/protocol.rs`（153行）

## 役割

Hub⇄ブラウザ端末間でやり取りするWebSocketメッセージの「ワイヤー形式」を
定義するモジュール。1行目のコメントの通り、**Hub→clientは`surface.config` /
`layer.state` / `error`の3種類しかない**（設計決定D6）。ファイルは短いが、
このプロジェクトの通信仕様そのものなので重要度は高い。

## 前提

- n01で見たJSON（keymap/layer/deck/layout）がここでもう一度「WSでどう送るか」の
  形として登場する。中身は同じでも「誰向けの表現か」が違うことに注意
  （ディスク上の形＝proto-keymap/deck.rs/layout.rsの型、通信の形＝ここ）。

## 読みどころ

- 12〜59行目 `ClientMessage` enum（クライアント→Hub。**ここが不変条件1の実物**）:
  - `KeyPress { keymap_id, key_id, edge }`（17行目）… 「位置ID」だけを送る。
    `keymap_id`はP-005段階Bで追加されたが、これも「どの盤面か」という位置情報
    でしかないのでルール違反にはならない、とコメントで明言されている。
  - `DeckPress { deck_id, slot_id }`（28行目）… Deckの1ボタン分の位置ID。
  - `SurfaceState { surface_id, delta, spin, active }`（37行目）… D28で追加された、
    連続値を出す面（トラックボール）用。`spin`/`active`は**Hubが読み捨てる**
    フィールド（`#[allow(dead_code)]`が付いている＝将来の3D面向けに形だけ用意）。
  - `SurfaceGesture { surface_id, gesture_id, edge }`（50行目）… タップ・長押し等の
    離散ジェスチャー。
  - **共通点**: どの列挙子も「id」か「量」しか運ばない。「それが何をするか」は
    一切含まれない＝解釈は必ずHub側のJSON（keymap/surface）が行う。
- 61〜83行目 `DeltaWire` / `SpinWire` / `EdgeWire` … 上記メッセージの部品。
  `DeltaWire`のdx/dyは`f64`で受け取り、丸め・クランプはws.rs側で行う（61行目コメント）
  ＝**このファイルはバリデーションをしない**、単なる形の定義だという点に注意。
- 94〜119行目 `LayerStateWire` … Hubからクライアントへ配る「今有効なレイヤー番号」。
  `for_keymap()`（116行目）でどの盤面のレイヤー状態かを明示できる
  （P-005段階Bで複数盤面を1画面に置けるようにするため）。
- 121〜138行目 `SurfaceConfig` … `surface.config`メッセージの中身。
  `keymap`・`decks`は常に送るが、`keymaps`（複数形）・`layouts`・`layer_states`は
  `/layout`面専用で`Option`（132〜137行目）。**既存の面（kb/ipad/panel）はこれらの
  フィールドを見ないので無影響**、という後方互換の作り方に注目。
- 140〜153行目 `ServerMessage` … Hub→clientの3種類そのもの。

## なぜこうなっているか

- ワイヤー形式を専用モジュールに独立させているのは、「プロトコルの形」と
  「サーバの処理ロジック(ws.rs)」「ドメインの型(proto-keymap/deck.rs)」を
  分けて、通信仕様だけを見たいときにここ1ファイルで完結させるため。
- 新しい面（`/layout`）を足すのに**既存メッセージ型を壊さず、`Option`フィールドを
  足すだけ**で対応している。これにより古い面(kb.html等)のコードは一切変更不要で
  動き続ける。

## 理解度チェック

1. `SurfaceState`の`spin`/`active`は今何のために存在しているか（今すぐ使うためでは
   ないとしたら、何のためか）？
2. `ClientMessage`のどの列挙子も「意味」を運ばないと言えるのはなぜか。もし
   `KeyPress`が`vk`のような文字列を直接運べる形だったら、何が危険になるか？
3. `SurfaceConfig`の`keymaps`（複数形）と`keymap`（単数形）は何が違うか？
