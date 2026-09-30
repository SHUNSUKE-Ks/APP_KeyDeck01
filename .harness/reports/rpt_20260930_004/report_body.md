# /api/qr に token を必須にした（以前からの穴）

## 完成状態
- `/api/qr` は token が無い・違うと 401。QR を出す11画面はすべて token を付けて取りに行く
- 凍結中の `editor_v1_1.html`・`trackball_v1_1.html` は1行だけ足した（利用者裁定 A・凍結の例外。行にコメントあり）

## 変更内容と判断
- 穴: token を確かめずに token 入り URL の QR を返していた。同じ Wi-Fi の誰でも QR から token（編集もできる鍵）を得られた。P-008 段階A の guardian 点検で見つかり、統括が再現した
- Hub: `QrQuery` に token、`qr_image` の入口で `token_ok`
- 画面: JS で作る6画面は `&token=` を足す。HTML に直書きの4画面は `data-qr` にして token を読んだ直後に src を入れる

## 検証結果
- `cargo test --workspace` = 173 passed（+1 `qr_requires_token`）
- 再起動した本物の Hub で: token なし 401、5画面の QR が token 付きで読み込まれる、QR ギャラリーを実マウスで開いて表示

## 成果物
- `crates/proto-hub/src/ws.rs`、`static/` の11画面、`DEVBOARD.md`（commit 4688e23）

## 次の行動
- なし（穴は塞いだ）。token は「名札」ではなく鍵なので、今後 token 入りの URL や画像を返す口を足すときは必ず token を確かめる
