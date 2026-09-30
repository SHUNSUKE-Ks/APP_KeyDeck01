# n19 — `static/panel.html`（699行）

## 役割

分割面（上=Stream Deck／下=キーボード）を1画面にまとめたT20/P-003 Ver1-a。
頭コメント（1〜24行目）が非常に重要な設計意図を語っている:

> キーボード部分はipad.htmlの描画をそのまま移植、Deck部分はdeck.htmlの描画を
> そのまま移植。WSは1本だけ。**この面はプロトコル変更ゼロで成立する**
> （`surface.config`が元々keymapとdeckの両方を1メッセージで配っているため）。

## 前提

- n16（ipad.html）とn18（deck.html）を先に読んでいること。このファイルは
  その2つの「合成」として読むのが最速。

## 読みどころ

- 244〜342行目 `log` / `setStatus` / `setConnectionBadge` / `renderDeck()` …
  ipad.html/deck.htmlと対応する関数群。`renderDeck`が上段のDeck部分。
- 342〜399行目 `fitDeckGrid` / `renderPageDots` / `setSlotsEnabled` /
  `sendDeckPress(slotId)` … Deckのページ送りドット表示など、deck.htmlより
  少しリッチになっている部分（複合画面ならではの追加UI）。
- 400〜511行目 `resolveDisplay` / `renderLabel` / `renderKeyboard()` … 下段の
  キーボード部分。ipad.htmlの`render()`に相当。
- 512〜587行目 `sendKeyPress(keyId, edge)` / `connect()` … WS接続は
  `/ws?surface=ipad&token=…`（キーボード部分はipad面と同じ扱い）。
  Deckの発火は`surface`に依存しない`deck.press`をそのまま使う
  （頭コメント17〜19行目）。
- 588〜653行目 `handleServerMessage` … `surface.config`から`keymap`と
  `decks`の両方を1回で受け取り、上下それぞれをrenderする。**新しいメッセージ
  種別を1つも追加していない**ことをここで確認できる。

## なぜこうなっているか

- 「新規発明を最小に保つ」（頭コメント12行目）という方針が徹底されている実例。
  複合画面を作るからといって新しいプロトコルや新しい描画ロジックを作らず、
  既存の部品を並べただけ、という設計判断。n15のcomponents.js以前の世代の
  ファイルだが、「既存コードの移植で新画面を作る」という考え方自体は
  components.jsの思想の先駆けになっている。

## 理解度チェック

1. なぜこの画面を作るのにサーバ側のプロトコル（protocol.rs）を1行も
   変更する必要が無かったのか？
2. 上段（Deck）と下段（キーボード）は、それぞれ別のWebSocket接続を持つか、
   1本を共有しているか？
3. `--split-deck`/`--split-kb`というCSS変数（頭コメント37行目）は何のために
   あると推測できるか？
