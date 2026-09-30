# P-007 端末から画像をアップロードして Gallery に足す

起票: 2026-09-27（利用者の依頼をメインチャットが依頼書にした。**依頼書のみ。実装はあとでまとめて**）
関係アプリ: APP_KeyDeck01（端末・Hub）／ APP/kanban-note01（Note Story の Gallery）

## 対象アプリ/画面
- 誰が: 利用者本人
- どこで: KeyDeck を表示している端末（iPad・スマホのブラウザ）で撮った／持っている画像を、その場で送る
- どこへ: Note Story（kanban-note01）の Gallery。作品（Title）と登場人物（Character）で絞って見られるようにする

## 課題
- 立ち絵の参考・ロケハン写真・手描きのラフなどは、スマホや iPad で撮ることが多い。いまは PC へ移してから Gallery に入れ直す手間がある
- Gallery の画像には「どの作品の、どの人物の画像か」が無いので、作品を書いているときに探せない

## 提案Surface（既存部品の流用を最優先）
- 端末側: 既存の Deck 画面（`static/panel.html` など）に「画像を送る」ボタンを1つ足す。中身は素の `<input type="file" accept="image/*">`（撮影も選択もブラウザ標準。D2: フレームワーク・PWA機構は入れない）
- 送る前に、作品（Title）と登場人物（Character）を選ぶ欄を出す。候補は、Note Story から書き出した Deck（`keydeck.deck_import.v1`）に入っている作品と人物を使う
- Hub 側: アイコン保存（`/api/icon/save`、`crates/proto-hub/src/icon.rs`）と同じ守り方で、画像1枚と付帯JSONを決まった置き場に保存する
- Note Story 側: Gallery に「KeyDeck から取り込む」ボタン。Hub の受信箱を読み、取り込んだものを Gallery の項目にする

## 必要な新規要素と実現可能性

### 1. 画像1枚につき JSON を1つ持たせる（利用者の質問への答え: **持たせてよい。むしろ持たせるべき**）
- 画像と同じ名前の付帯ファイル（`<id>.webp` と `<id>.json`）にする。まとめた1本の目録より、1枚ずつのほうが壊れにくく、足す・消すが簡単
- 作品・人物は **名前ではなく ID で持つ**。Note Story では人物の名前を後から変えられる（名前札・サイドバーの ✎）が、人物の ID（`char_…`）と作品の ID（notebook id）は変わらないため。名前は表示用の控えとして一緒に入れる
- 案（スキーマ名 `keydeck.gallery_upload.v1`）:

```json
{
  "schema": "keydeck.gallery_upload.v1",
  "id": "img_20260927_114502_a1b2",
  "file": "img_20260927_114502_a1b2.webp",
  "mimeType": "image/webp",
  "fileSize": 184233,
  "width": 1536,
  "height": 2048,
  "sha256": "…",
  "createdAt": "2026-09-27T11:45:02+09:00",
  "source": { "surface": "ipad", "label": "iPad（リビング）" },
  "label": "港町の朝・ロケハン",
  "description": "",
  "tags": ["ロケハン"],
  "title": { "id": "nb_xxx", "name": "軌跡＿3rd" },
  "characters": [{ "id": "char_mia", "name": "ミア" }],
  "scene": { "id": "p2", "name": "第2場面" }
}
```

- `title`・`characters`・`scene` は無くてもよい（あとで Gallery 側で付けられる）
- Note Story の Gallery 項目（`src/pages/gallery/types.ts` の `GalleryItem`）に `titleId?` `characterIds?` `sceneId?` を足し、フィルターに「作品」「登場人物」を足す（**TitleDB・CharacterTAB をキーに追加、フィルター表示に追加**＝利用者の依頼）

### 2. Hub の保存API（**CLAUDE.md 不変条件6 の追加が要る＝利用者の裁定が必要**）
アイコン保存と同じ条件をすべて満たすこと:
- 書き先は `gallery_inbox/` の下**だけ**。ファイル名は Hub が作る（クライアントから受け取らない）
- トークン必須（既存の保存APIと同じ）
- 中身の先頭バイトで形式を確かめる（JPEG・PNG・WebP のみ。SVG は受けない）。大きさの上限（例 10MB）
- 付帯JSONはそのまま書かず、スキーマで解釈し直して整形したものを書く。未知のキーは落とす
- 書いたあと読み直して検証。失敗したら消して、元の状態を保つ
- 送信の回数に上限（連打・誤送信対策）

### 3. Note Story が受信箱を読む道（**未決。裁定したい点**）
- 案A（おすすめ）: PC で開いた Note Story の Gallery から、Hub の `GET /api/gallery/inbox`（一覧）と画像を読んで取り込む。取り込み済みは Hub 側で `imported/` へ移す
  - 注意: 本番（https://kanban-note01.vercel.app）から `http://localhost:8770` を読むのは、ブラウザの「ローカルネットワークへのアクセス」の確認が出ることがある。PC の開発版（http://localhost）からなら問題ない
- 案B: Hub が Firebase Storage へ直接上げる → 外部サービスの鍵を Hub に持たせることになるので、おすすめしない
- 案C: 受信箱のフォルダーを、Gallery の既存の「ファイルから追加」で手で選ぶ → 仕組みは要らないが手間が残る

### 4. 端末が作品・人物の候補を知る道
- Note Story の「KeyDeckへ書き出し」（`keydeck.deck_import.v1`）に、作品の ID と、人物の ID を足す（いまは名前と番号だけ）
- 端末はその Deck を読んでいるので、送信の画面で候補として出せる

## 受け入れ基準案（G形式）
- G-a: 端末で「画像を送る」→ 画像を選ぶ → 作品と人物を選んで送ると、Hub の `gallery_inbox/` に画像と付帯JSONが1組できる
- G-b: JPEG・PNG・WebP 以外（SVG・偽の拡張子を含む）と上限を超える大きさは、コード付きの1行エラーで断られ、何も書かれない
- G-c: トークンの無い送信は断られる
- G-d: 付帯JSONの `title.id` と `characters[].id` は、Note Story の作品 ID・人物 ID と一致する
- G-e: Note Story の Gallery で「KeyDeck から取り込む」を押すと、受信箱の画像が項目になり、作品・人物のフィルターで絞れる
- G-f: 人物の名前を Note Story で変えたあとも、その人物で絞ると同じ画像が出る（ID で結んでいるため）
- G-g: 同じ画像（同じ sha256）を2回取り込んでも、項目は1つ

## 昇格条件5項目の自己評価（vision §2）
- ① MVP完成: KeyDeck 本体は動いている。Note Story の Gallery もある
- ② 利用場面1つに定まる: 「端末で撮った画像を、作品・人物つきで Gallery へ」の1つ
- ③ 受け入れ基準を先に書ける: 上の G-a〜G-g
- ④ 既存Protocol内か: **外れる**。画像を受ける保存APIが新しく要る（不変条件6に `gallery_inbox/` を足す裁定が必要）。キー入力の Protocol には触らない
- ⑤ 10日以内の粒度: Hub 保存API（アイコン保存の流用）＋端末ボタン＋ Note Story の取り込み・フィルター。分ければ収まる見込み

## 未決事項（裁定したい点）
1. `gallery_inbox/` への書き込みを不変条件6に足してよいか
2. 受信箱を読む道は 案A でよいか
3. 画像を Hub 側で縮小・WebP 化するか（端末の写真は大きい）、そのまま置くか
4. 付帯JSONに撮影位置（GPS）などの写真の情報を残すか（残さないほうが安全。おすすめは消す）

## 裁定
未（FABLEが記入）
