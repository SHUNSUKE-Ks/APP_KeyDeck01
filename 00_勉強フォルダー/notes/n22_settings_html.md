# n22 — `static/settings.html`（351行）

## 役割

Hubの操作画面。「いま何が読み込まれているか」「どこへ繋げるか」を確認する
ための画面（T9、`/settings?token=…`）。頭コメント22〜24行目に明記の通り、
**この画面は書き込みを一切しない**（`GET /api/formats`と`POST /api/reload`の
2本のみ。`/api/reload`もディスクを読み直すだけで何も書かない）。

## 読みどころ

- 156〜217行目 `log` / `setStatus` / `el(tag, className, text)`（小さなDOM生成
  ヘルパー） / `table(headers, rows, host)`（一覧表を組み立てる共通関数） /
  `firstLine(text)` / `openQr` / `closeQr` … 一覧行＋QRモーダルというUIパターン
  の部品（頭コメント17〜19行目の見た目の正と対応）。
- 223〜236行目 `loadFormats()` … `fetch("/api/formats?token=...")`を呼んで
  Hubの構成を取得する。n14の`formats_handler`（272行目）が返すJSONを
  そのまま受け取る側。
- 237行目〜 `render(data)` … 受け取ったデータ（targets/keymaps/decks/layouts）を
  一覧表として描画する。
- 319行目 `fetch("/api/reload?token=...", {method:"POST"})` … 再読込ボタンの
  実体。n14の`reload_handler`を叩き、n09の`load_startup_data`を再実行させる。

## なぜこうなっているか

- 「読み取り専用」に徹しているのは、この画面の目的が「今の構成を疑わずに
  確認できる」ことだから。書き込み機能（layer/keys編集等）は別の専用画面
  （n23 keys.html、n21 editor.html）に分離されており、責務が画面ごとに
  はっきり分かれている。
- `targets`が`connection_targets()`（n05/n14参照）由来であることを頭コメントで
  明言しているのは、「ランディングページ・起動バナーと必ず同じ並びになる」
  という一貫性を保証するため（n05で見た過去のバグの再発防止）。

## 理解度チェック

1. なぜ設定画面には「保存」ボタンが無いのか？
2. 再読込ボタンを押したとき、ディスク上のファイルは変更されるか？
3. `targets`の並び順がこの画面と起動バナーで必ず一致するのはなぜか？
