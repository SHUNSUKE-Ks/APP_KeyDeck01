# n18 — `static/deck.html`（308行）

## 役割

Deck単独面（Stream Deck風ボタン集合のみを表示する画面、T4）。
`/deck?token=…`で開く。

## 読みどころ

- 133〜200行目 `log` / `setStatus` / `render(deck)` / `setSlotsEnabled` /
  `sendPress(slotId)` … n01のDeck構造（grid/pages/slots）を描画し、
  タップで`deck.press`を送る。n07の`ClientMessage::DeckPress`の送信元の1つ。
- 200〜242行目 `connect()` … WS接続。`deckId`を指定しないため、
  n06の`DEFAULT_DECK_ID`（省略時のデフォルトDeck）を暗黙に使うことになる
  （実際にどのdeckを開くかはHub側のws.rsの`handle_deck_press`で解決される、
   n14参照）。
- 270〜291行目 `handleServerMessage` … `surface.config`の中の`decks`
  （n07の`SurfaceConfig.decks`、`BTreeMap<String, DeckSetlist>`）から
  対象のDeckを取り出して`render`する。

## なぜこうなっているか

- 単独でDeckだけを表示する需要（Stream Deck機のような専用デバイスとして
  1台をDeck専用にする）に応えるための最小構成。panel.html（n19）はこれに
  キーボードを組み合わせた発展形で、**Deck部分のロジックはdeck.htmlのものを
  そのまま移植している**（panel.htmlの頭コメントに明記）。

## 理解度チェック

1. `deck.press`を送るときに`deckId`を省略すると、Hub側はどのDeckを対象にするか
   （n06/n14を参照して答えられるか）？
2. `decks`が複数存在する場合（P-005段階A以降）、このファイルはどうやって
   「どのDeckを描くか」を決めているか（URLクエリの有無を確認する）？
