# n11 — `crates/proto-hub/src/deck.rs`（313行）

## 役割

Deck（Stream Deck風のボタン集合）の型定義・ロード・検証。1〜2行目のコメントの
通り、Actionの型自体は`proto_keymap`のものをそのまま再利用している
（「正は1箇所」の原則がここでも守られている）。

## 読みどころ

- 33〜36行目 `Grid { cols, rows }` … Deckの格子サイズ。
- 40〜47行目 `Slot { slot_id, label, icon, action }` … ボタン1つの中身。
  `action`は`proto_keymap::Action`そのもの＝keymapのキーと全く同じ語彙で
  「何をするか」を表現できる。
- 51〜54行目 `Page { id, slots }` … Deckは複数ページを持てる（ページ送りで
  スロット数を超えるボタンを扱える）。
- 56〜65行目 `DeckRender`（`Grid` / `List`）… **正方形スロットの格子**か
  **横いっぱいの縦リスト**かの見た目切り替え。省略時は`Grid`（デフォルト実装で
  既存JSONに影響を与えない設計、58行目のコメント）。
- 69〜79行目 `DeckSetlist { deck_id, description, grid, render, pages }` …
  n01の`decks/*.json`の実例と照らし合わせる。
- 81〜93行目 `impl DeckSetlist`:
  - `find_slot(slot_id)`（82行目）… 全ページを横断してslotIdからスロットを探す。
    ws.rsの`handle_deck_press`から呼ばれる。
  - `actions()`（90行目）… 全スロットのActionを1つのイテレータにする。
    起動時の許可リスト構築（`command_registry`）や、keymapId参照の検証
    （Deckのslotが`KeymapSwitch{id}`を持つ場合、そのidが実在するか）に使われる。
- 95行目〜 `load_deck_from_path` / `load_deck_str` … ファイル読み込み→JSON構文
  チェック→スキーマ検証→中のvkが辞書内かチェック、という流れ
  （proto-keymapのkeymapロードと同じ設計思想）。

## なぜこうなっているか

- DeckのActionがproto-keymapの型そのものであることは、「キーボードのキーで
  できることは、Deckのボタンでも同じだけできる」ことを意味する。型を共有する
  ことで、新しいAction種別を1箇所（proto-keymap）に足すだけで、キーボードにも
  Deckにも同時に反映される。
- `render`をenumにして省略時デフォルトを`Grid`にしているのは、見た目の選択肢を
  増やしても既存のJSONファイルを1つも書き換えずに済むようにするため
  （後方互換性を「書かなくても動く」形で確保する典型パターン）。

## 理解度チェック

1. Deckのボタンにキーボードと同じ`{"t":"key","vk":"A"}`という中身を書けるのは
   なぜか？
2. `render`を省略したDeck JSONは、既存の挙動と比べて何が変わるか（変わらないか）？
3. `find_slot`が「全ページを横断して探す」ということは、slotIdはDeck全体で
   一意である必要がある、と言えるか？
