# n17 — `static/kb.html`（361行）

## 役割

分割キーボード面（2台を同期させて使う、このプロジェクト最初期の画面T4）。
`/kb?half=left|right&token=…`というURLで、左半分・右半分をそれぞれ別の
デバイスで開く運用を想定している。

## 前提

- n16（ipad.html）を先に読んでおくと、`log`/`connect`/`handleServerMessage`
  などの共通パターンが既知になっているので読みやすい。
- n06の`SurfaceKind::Split`（分割/Deckの共有state）と対応する面であることを
  意識する。

## 読みどころ

- 166〜262行目: `log` / `setStatus` / `resolveDisplay` / `render` /
  `setKeysEnabled` / `bindKey` / `sendPress` … ipad.htmlとほぼ同じ構造。
  差分は`half`パラメータで左右どちらの`Half`（n02の`Halves`/`Half`型）を
  描くかだけ、という点に注目して読むと速い。
- 268〜314行目 `connect()` … WS接続先は`surface`パラメータを付けない
  （n06のデフォルト＝`SurfaceKind::Split`扱い）。
- 324〜345行目 `handleServerMessage` … `surface.config`/`layer.state`/`error`の
  3分岐。iPad面と違い、**分割の2台が同じ`layer.state`配信を受けて同期する**
  （n06のHubStateが持つ共有`layer_state`のおかげ。G2の分割同期）。

## なぜこうなっているか

- 「左右で別デバイス」という要件のために、盤面全体ではなく`half`だけを
  切り出して描く必要があった。これがkeymapの型に`Halves`/`Half`という
  概念がある理由（n02参照）。
- iPad面より前に作られたため、ヘッダのQRボタン(D25)・状態ボタン(D26)などの
  後発の装飾が無い、素朴な作りのまま残っている（比較すると、機能追加が
  どの順で積み上がってきたかの歴史が見える）。

## 理解度チェック

1. 左右2台のkb.htmlが同じレイヤー状態を見ているのはなぜ可能か（n06のHubStateの
   どのフィールドのおかげか）？
2. `half=left`と`half=right`で、送信する`key.press`のkeyIdの集合はどう違うか？
3. なぜこの面には`keymapId`を明示的に送るフィールドが無いのか（n07の
   `ClientMessage::KeyPress`の`keymap_id`が省略時にどう扱われるか、n06と
   合わせて考える）？
