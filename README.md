# APP_KeyDeck01 — Stream Deck面＋2台分割レイヤーキーボード（原型）

PCで動くRust基地局（proto-hub）に、Android ChromeがただのWebページとして接続するコントローラー。
PWA・フロントフレームワーク不使用。頭脳は全部Rust側。**CODEXのAPP_ControlDeckとは別アプリ。**

## 起動

**このフォルダ（workspace root）で**実行すること（`keymaps/`・`decks/`・`static/` を相対パスで読むため）:

```
cargo run -p proto-hub
```

起動するとコンソールに接続URLが表示される（token付き）:

- **QRつきランディングページ**: `http://<LAN-IP>:8770/` ← まずはこれをPCブラウザで開く
- 分割キーボード左手: `http://<LAN-IP>:8770/kb?half=left&token=…`
- 分割キーボード右手: `http://<LAN-IP>:8770/kb?half=right&token=…`
- Stream Deck面:      `http://<LAN-IP>:8770/deck?token=…`

PCでランディングページ（`/`）を開き、そこに出る3つのQRコードをAndroid 2台のカメラで
読み取れば、URL/tokenを手入力せずそのまま接続できる（分割キーボード左手・右手・Deck）。
開発中はPCブラウザ2窓で代替可（検証もまずこれで行う）。

**注意（D8）**: tokenはHub起動のたびに新しく生成される（プロセスの寿命だけ有効）。
Hubを再起動したら、コンソールに新しく出るURLを開き直すこと。
Android実機から繋がらない場合はWindows FirewallでTCP 8770のインバウンドを許可すること。

## キーマップの編集（G4: コード変更不要）

- `keymaps/keymap_writing01.json` … **改造用**。ここを編集して再起動すれば反映される
- `keymaps/keymap_default.json` … **初期値・リセット用。編集しない**。Deckの「Default戻し」でいつでもここへ戻れる
- レイヤー: `mo`（押下中のみ）/ `tg`（トグル）。書式は `schemas/keymap.schema.json` と設計書D3/D4

## Deckセットリスト（D11）

- Import: JSONを `decks/` に置いて再起動（`schemas/deck.schema.json` で検証される）
- Export: ブラウザで `/api/deck/export` を開くと現在のセットリストがダウンロードされる

## トラブルシュート（D9）

エラーは必ず「原因コード＋cause」付きで **Hubのコンソール** と **ブラウザのF12コンソール** の両方に出る。
`[KD][ERR][コード]` で検索。コード一覧は `brief/keydeck_design_v0.2.md` のD9。

## 動作確認方法

```
cargo test --workspace          # 全crateの自動テスト（proto-keymap 20件・proto-adapter-win 7件・他）
cargo run -p proto-adapter-win --example smoke_notepad   # 実SendInputの手動smoke（メモ帳フォーカスで実行）
```

## ドキュメント

- 設計書（正）: `brief/keydeck_design_v0.2.md` ／ 状況: `DEVBOARD.md` ／ 差し戻し: `brief/spec_return_log.md`
