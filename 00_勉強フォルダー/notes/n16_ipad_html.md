# n16 — `static/ipad.html`（487行）

## 役割

iPad一枚キーボード面。今の主力の実機画面。頭コメント（1〜24行目）にある通り、
見た目の正は`brief/mockup/screen_mock_v0.4.html`、配置の正は
`surface.config.keymap.board`（D24グリッド式）。**このファイルは`components.js`
より前に書かれた古い世代**で、独自に描画コードを持っている（kb.htmlと同系統の
「骨格は共通・盤面ごとに手書き」という作り方。n20/n21が後発の
`components.js`共有パターン）。

## 前提

- n01のkeymap_ipad01_vol13.jsonの`board`と、n02の`resolve()`を読んでいること。
- WS接続先は`/ws?token=…&surface=ipad`固定（n06の`SurfaceKind::Ipad`と対応）。

## 読みどころ

- 198〜228行目 `resolveDisplay(keyId)` / `renderLabel` … 現在のレイヤー状態から
  そのkeyIdの表示ラベルを決める。n02の`resolve()`のロジック（大きいレイヤー番号
  優先・transフォールスルー）とほぼ同じ考え方を、表示専用にJS側でも行っている
  （実際の発火判定はHub側だが、見た目のプレビューはクライアント側で先取りする
  ため）。
- 229〜276行目 `render()` … `board`を1回丸ごと描画する。
- 285〜318行目 `bindKey` / `sendPress(keyId, edge)` … pointerdown/upで
  `key.press`メッセージを組み立てて送信する。**ここがn07の`ClientMessage::KeyPress`
  の送信元**。
- 319〜366行目 `connect()` … `new WebSocket("/ws?surface=ipad&token=...")`
  （324〜326行目）。
- 367〜393行目 `diagnoseDisconnect` / `scheduleReconnect` … 切断時の原因診断・
  再接続。
- 394〜441行目 `handleServerMessage(data)` … `switch`文で`surface.config` /
  `layer.state` / `error`（n07の`ServerMessage`3種そのもの）を振り分ける。
  ここを読むとn07で見た型が実際にどう使われるか具体的に分かる。
- 170〜196行目 `log` / `setStatus` / `setConnectionBadge` … `[KD][ERR][コード]`
  形式のログ（D9）と接続状態バッジ（D26）。他の面（kb/deck/panel/trackball）にも
  ほぼ同じ形の関数がある＝**このファイルを読めば残りの面の共通パターンが
  ほぼ分かる**、という意味で最初に読む価値が高い。

## なぜこうなっているか

- iPad面が「専用keymap固定・独立layer_state」（n06参照）である設計に対応して、
  このファイルはURLに`half=`のような分割指定を持たず、常に同じ盤面を描く
  シンプルな構造になっている。
- `components.js`共有以前の設計のまま残っているのは、「動いているものを
  無理に統一しない」という現実的な判断（INDEXにも「今の主力」と書かれている
  通り、実績のあるコードを壊すリスクを取っていない）。

## 理解度チェック

1. `handleServerMessage`の`case "layer.state":`では何を再計算する必要があるか
   （ヒント: ラベルの見た目が変わる）？
2. このファイルの`resolveDisplay`とn02の`resolve()`は、同じレイヤー解決の
   考え方をどこで・なぜ二重に持っているか？
3. `sendPress`が送るメッセージの形は、n07のどの型と対応するか？
