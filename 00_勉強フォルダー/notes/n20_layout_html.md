# n20 — `static/layout.html`（725行）

## 役割

レイアウト面（P-005段階B/C）。`layouts/layout_*.json`の区画割りを**そのまま
描く実機用の1枚**。`/layout?id=<layoutId>&token=…`。頭コメント（3〜29行目）に
「三層（section/Component/slot）」の説明があり、n12（layout.rs）の型定義と
1対1で対応する。

## 前提

- n12（layout.rs）とn15（components.js）を先に読んでいること。このファイルは
  `components.js`を実際に呼び出す側の代表例。

## 読みどころ

- 432〜436行目 `renderComponent(body, component, effectiveRef)` … `component.kind`
  による分岐（`"keyboard"` → `renderKeyboard`、`"deck"` → `renderDeck`、
  `"trackball"` → `renderTrackball`）。**これがn12のコメントで言われていた
  「部品種別を増やすときに触る2箇所」のうちの1箇所**（もう1箇所は
  `crates/proto-hub/src/layout.rs`の`ComponentKind`）。
- 509〜549行目 `renderKeyboard` / `renderDeck` / `renderTrackball` … 前2つは
  n15の`KDComponents`（`renderKeyboard`/`renderDeck`）を呼んでいるはず
  （実際に該当行を開いて`KDComponents.`という呼び出しがあるか確認する）。
  `renderTrackball`（22〜25行目のコメント参照）は**`static/trackball.html`を
  `<iframe ...&embed=1>`で埋め込む**というやり方。「検証済みのコードを
  無改変のまま使う」という判断で、WS接続が1本増えるという代償を払っている
  （コメントに明記）。
- 259〜275行目 `isTextKeyboard` / `effectiveKeymapId(section)` … 文字入力用
  キーボードかどうかの判定、実際に使うkeymapIdの決定（設定で上書きできる
  仕組みがあるらしいことが`loadPref`/`savePref`(242〜248行目)から読み取れる）。
- 560〜577行目 `sendKeyPress(keymapId, keyId, edge)` / `sendDeckPress(deckId, slotId)`
  … n07の`ClientMessage::KeyPress`/`DeckPress`で**`keymapId`/`deckId`を明示的に
  付けて送る**唯一の面（他の面は省略して面ごとのデフォルトに頼る）。
  複数キーボード・複数Deckを1画面に同時に置けるのはこのおかげ。
- 578〜674行目 `connect()` / `handleServerMessage` … `surface=layout`で接続し、
  `surface.config`から`keymaps`（複数形）・`layouts`・`layer_states`
  （n07のOptionフィールド）を受け取る。既存面と違うデータの受け取り方を
  している点に注目。
- 675行目〜 `rerenderKeyboards(keymapId)` … `layer.state`を受けて、
  該当するkeymapIdの盤面だけを再描画する（複数盤面が独立して更新される
  仕組み）。

## なぜこうなっているか

- 「HTMLを1行も触らずにJSONだけで配置を変える」というP-005の目的を実現する
  ために、このファイルは**特定の配置を一切ハードコードしない**。全ての
  位置情報は`layouts/*.json`から来る。
- トラックボールだけiframe埋め込みという「間に合わせ」の統合方法を取って
  いるのは、Core/View/Sinkに分離された既存実装（n26）を壊さず、かつ
  すぐに動くものを優先したトレードオフ。将来の課題として「共有モジュール化」
  がコメントに明記されている＝**技術的負債を隠さず書き残す**姿勢。

## 理解度チェック

1. `layout.html`はなぜ`keymapId`/`deckId`を毎回明示的に送る必要があるのか
   （他の面が省略できる理由と対比して）？
2. トラックボールをiframeで埋め込む設計の「代償」は何か？
3. 部品種別を1つ増やすとき、このファイルのどの関数とn12のどの型を
   同時に変更する必要があるか？
