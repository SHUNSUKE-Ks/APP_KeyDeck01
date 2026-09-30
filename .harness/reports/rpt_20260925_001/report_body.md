# Deckの取り込み＋アイコン保存（外のアプリ→KeyDeck）

## 完成状態

- Deck編集 `/deckedit` の「取り込み」ボタンで、外のアプリが書き出した1ファイル（`keydeck.deck_import.v1`）から、Deck とキャラクターの四角アイコンをまとめて取り込める。
- 画像は新しい `POST /api/icon/save`、Deck は既存の `POST /api/deck/save` を通る。検査はどちらも Hub 側で行う。

## 変更内容と判断

- **利用者の裁定（2026-09-25）で、不変条件6に `static/icons/<name>.<ext>` を追加した。** CLAUDE.md に条件を書いた。
  - ファイル名と拡張子はクライアントから受け取らない。拡張子は先頭バイトで決める（PNG/JPEG/WebP）。
  - **SVG は拒否する。** `/icons` は Hub と同じオリジンで配られ、SVG はスクリプトを埋め込めるため。
  - 1MiBまで。`.bak` へ退避してから書き、読み直して一致を確かめ、合わなければ巻き戻す。
- PWA から Hub へ直接送る案は採らなかった。Hub は CORS を許可しておらず、HTTPS の PWA から LAN の HTTP への通信もブラウザが遮断するため、ファイルの受け渡しにした。
- 取り込み時は、参照先の無い `iconRef` や形式違いを、画像を1枚も書く前に止める（中途半端に画像だけ残らない）。
- 保存後に Hub が配り直す surface.config が、fetch の応答より先に届くことがある。そのため、保存前に「開くDeck」の印を付けておく。

## 検証結果

- `cargo test --workspace` = **158 passed, 0 failed**（新規6件）。`cargo build` 警告0。
- deckedit.html を jsdom 上で動かした（fetch と WebSocket は差し替え）。正常系では画像2枚→icon保存が2回、Deck の icon が `/icons/*.png` に置き換わり、新しい Deck が開いた。異常系では、iconRef が欠けていると通信0回で停止し、形式違いでも停止した。
- **未実施**: 実Hubでの通し。Hub は起動していない（AGENTS.md の規則、および利用者が使用中の8770を止めないため）。

## 成果物

- `crates/proto-hub/src/icon.rs`（新規）、`ws.rs`・`error.rs`・`main.rs`
- `static/deckedit.html`
- `CLAUDE.md`（不変条件6）、`DEVBOARD.md`（2026-09-25 節）
- `C:\00_CreatorCompass\KeyDeck\できること.md`（版2.2）

## 次の行動

- Hub を再起動 → `/deckedit` → 取り込み → kanban-note01 の「KeyDeckへ書き出し」で作ったファイルを選ぶ → Deck とアイコンが出ることを確認する。
- その Deck をレイアウト編集で board に置き、ALT+番号で PC の会話ログVIEW にセリフが増えることを実機で確認する。
