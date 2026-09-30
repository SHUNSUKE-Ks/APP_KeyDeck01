# n12 — `crates/proto-hub/src/layout.rs`（285行）

## 役割

「画面の区画割り」（`layouts/layout_*.json`）の型定義・ロード・検証。
n01で見たJSONの「三層」（section/Component/slot）のうち、section と Component の
実体がここにある（P-005 段階B）。

## 読みどころ

- 3〜19行目の頭コメントが設計意図の要約。「画面=1枚の手書きHTML」だったものを
  「画面をグリッドに区切り、各区画に部品を1つ割り当てる」というJSONに
  持ち上げた、という経緯が書かれている。
- 48〜59行目 `ComponentKind`（`Keyboard` / `Deck` / `Trackball`）… 48行目コメント
  「増やすときはここと`static/layout.html`の描画分岐を同時に足す」に注目。
  **部品種別を1つ増やすときに触る場所が正確に2箇所だと明記されている**
  （n20で対応する側を見る）。
  - `Keyboard`（52〜54行目）: `ref`はkeymapId。**十字キーも「小さいboardを持つ
    keyboard」でしかなく、新しい部品種別ではない**という一文が重要
    （n01のlayout_ipad_mainで十字キーがkeyboardとして表現されていたのはこのため）。
- 61〜68行目 `Component { kind, reference }` … `reference`（JSON上は`ref`）が
  実在するかは、このファイル単体では分からず**startup側で全データロード後に
  検証する**（65行目のコメント）。単体ファイルの検証と、複数ファイルを
  横断する参照検証を分けている設計。
- 70〜81行目 `Section { id, row, col, col_span, row_span, component }` …
  keymapの`Board`/`BoardKey`と全く同じ形（row/col/colSpan/rowSpan）。
  n01のコメントにあった「D24と同じ形」がここで確認できる。
- 91〜96行目 `LayoutGrid { cols, rows }` … 画面全体の格子サイズ。
- 98〜107行目 `Layout { layout_id, description, grid, sections }` 本体。
- 109行目〜 `load_layout_from_path` / `load_layout_str` … 読み込み→検証。
  n01の`layout_ipad_main.json`のコメントにあった「はみ出し」「重なり」
  「参照先の実在」チェックがこのファイルのどこかで行われている
  （実際に読むときは`mod tests`の193行目以降のテスト名を先に見ると、
  どんな異常系がテストされているか一覧できて理解が早い）。

## なぜこうなっているか

- gridの形をkeymapのBoardと合わせているのは、「区画の配置」も「盤面上のキーの
  配置」も本質的に同じ問題（矩形をグリッドに置く）だから。同じ形にしておくと、
  `static/layout.html`や`static/components.js`の描画コードを使い回せる
  （実際に`board`も`section`も同じCSS Gridの考え方で描かれる）。
- 参照先の実在チェックをこのファイル単体でやらずstartup側に任せているのは、
  layoutファイル単体では「参照しているkeymapId/deckId/surfaceId」の実在を
  判断できない（他のファイルを読まないと分からない）ため。責務の境界が
  ファイルの読み込み順序と一致している。

## 理解度チェック

1. `layouts/layout_ipad_main.json`のSEC-RIGHTの`component.ref`に存在しない
   keymapIdを書いたら、どの段階で・どんなエラーコードで弾かれるか
   （n09のstartup.rsと合わせて考える）？
2. なぜ十字キーは新しい`ComponentKind`ではなく`Keyboard`として実装されているのか？
3. 部品種別を1つ増やすとき、触る必要がある2箇所とはどこか？
