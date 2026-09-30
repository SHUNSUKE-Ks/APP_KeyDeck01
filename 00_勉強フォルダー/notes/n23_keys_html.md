# n23 — `static/keys.html`（1143行）

## 役割

キーの「中身」編集画面（`/keys?keymap=<id>&layer=<n>&token=…`）。頭コメントの
「やる/やらない」の区別が重要: **ラベル・アクション・レイヤー割当は変えるが、
キーの「位置」（board上のrow/col）は変えない**（位置編集はCLAUDE.md不変条件6の
書き込み許可外で、裁定が要ると明記）。

## 前提

- n01・n02（keymap/layerの形とレイヤー解決）を読んでいること。
- 頭コメント23〜27行目の「継承の見え方」を先に理解しておく: あるレイヤーを
  編集していても、そこに定義の無いキーは下のレイヤーの値が**薄く**見える
  （D3のtransフォールスルー）。「このレイヤーが自分で決めているキー」だけが
  実線表示。

## 読みどころ

- 235〜272行目 `snapshot()` / `undo()` / `updateUndoBtn()` / `clearHistory()` …
  **undo機能**。編集操作のたびに状態のスナップショットを取り、取り消せるように
  している（メモリに記載のあった`KEYDECK_LOGIC_001`のundoStackがこれに該当。
  技術トピックカードとして既に登録されているロジックなので、余裕があれば
  `C:\00_CreatorHub\knowledge\TechTopicCards\`側のカードも見比べると理解が
  深まる）。
- 297〜387行目 `buildParts()` / `layerName(layer)` / `loadRecent()` /
  `pushRecent(part)` … 上部の「部品エリア」（ラベル・アクションの組み合わせの
  候補一覧）と、最近使った部品の履歴。
- 397〜503行目 `renderParts()` / `bindPartDrag` / `renderHint()` … 部品を
  ドラッグしてキーへ入れる操作（頭コメント13〜21行目にある「選んでから押す」
  「消す操作だけドラッグ」という操作方針の実装）。
- 526〜669行目 `renderBoard()` / `showGhost` / `moveGhost` / `hideGhost` /
  `keyUnder` / `clearDragMarks` / `showDropCell` / `hideDropCell` … 盤面の描画と、
  ドラッグ中の見た目のフィードバック（ゴースト表示・ドロップ先ハイライト）。
- 669〜944行目 `bindKeyEdit` / `shownDef(keyId)` / `standardKeySize` /
  `newKeyId(row, col)` / `boardKey(id)` / `cellAt` / `keyAtPoint` /
  `boardConflicts` / `paintConflicts` … キー1個をクリックしたときの編集UIと、
  競合（同じ位置に複数キー等）の検出・表示。
- 1042行目 `fetch("/api/keymap/board/save?...")` / 1062行目
  `fetch("/api/layer/save?...")` … **2種類の保存**が別々のAPIを叩くことに
  注目。頭コメント29〜33行目の通り、保存先は
  `POST /api/layer/save → keymaps/layers/<keymapId>_layer<N>.json`
  （既存レイヤーの上書きのみ）。この画面から新しいレイヤーの追加は
  できない（マニフェストの書き換えを伴うため）。
- 1093〜1100行目 `connect()` … `surface=layout`で接続（レイアウト系の画面と
  同じ入口を共有している）。

## なぜこうなっているか

- 「位置」と「中身」を別の画面・別のAPI（`board/save`と`layer/save`）に
  分けているのは、CLAUDE.mdの不変条件6が定める書き込み範囲の細分化と対応
  している。1つの巨大な「なんでも編集画面」にせず、権限の範囲ごとに
  画面を分けることで、何が変更可能で何が不変条件の対象かが画面構成
  そのものから読み取れる。
- undo機能を持たせているのは、盤面という「間違えると気づきにくい」対象を
  編集する画面だからこそ、安全に試行錯誤できるようにするため。

## 理解度チェック

1. この画面から新しいレイヤー自体を追加できないのはなぜか？
2. 「継承しているキー」と「このレイヤーが自分で決めているキー」の違いを
   自分の言葉で説明できるか？
3. 位置編集（並び替え）がこの画面に無いのは、単なる未実装か、それとも
   意図的な制約か？CLAUDE.mdのどの記述が根拠になるか？
