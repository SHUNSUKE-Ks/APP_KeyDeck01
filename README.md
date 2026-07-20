# APP_KeyDeck01 — Stream Deck面＋2台分割レイヤーキーボード（原型）

PCで動くRust基地局（proto-hub）に、Android ChromeがただのWebページとして接続するコントローラー。
PWA・フロントフレームワーク不使用。頭脳は全部Rust側。**CODEXのAPP_ControlDeckとは別アプリ。**

## 起動

**このフォルダ（workspace root）で**実行すること（`keymaps/`・`decks/`・`static/` を相対パスで読むため）:

```
cargo run -p proto-hub
```

D26: サーバー自体の起動はブラウザからは物理的に不可能（ページはサーバーが配るため）。
リポジトリ直下の **`start_hub.cmd`** をダブルクリックすればcdをスクリプト自身の場所に
合わせた上でHubを起動できる（`cargo run -p proto-hub`と同じ。ウィンドウは起動したまま）。

起動するとコンソールに接続URLが表示される（token付き）:

- **QRつきランディングページ**: `http://<LAN-IP>:8770/` ← まずはこれをPCブラウザで開く
- 分割キーボード左手: `http://<LAN-IP>:8770/kb?half=left&token=…`
- 分割キーボード右手: `http://<LAN-IP>:8770/kb?half=right&token=…`
- Stream Deck面:      `http://<LAN-IP>:8770/deck?token=…`
- **iPad一枚キーボード（Vol1.2）**: `http://<LAN-IP>:8770/ipad?token=…`（画面下部固定。
  ヘッダに取り消し(↺=Ctrl+Z)/進む(↻=Ctrl+Y)・QRボタン・接続状態ボタンを持つ。
  iPad Pro 12.9横持ち相当の横長画面から利用する想定）
- 設定（再読込ボタン）: `http://<LAN-IP>:8770/settings?token=…`（下記「段階B」参照）

PCでランディングページ（`/`）を開き、そこに出るQRコードをスマホ/タブレットのカメラで
読み取れば、URL/tokenを手入力せずそのまま接続できる（分割キーボード左手・右手・Deck・iPad）。
開発中はPCブラウザ複数窓で代替可（検証もまずこれで行う）。

**注意（D8）**: tokenはHub起動のたびに新しく生成される（プロセスの寿命だけ有効）。
Hubを再起動したら、コンソールに新しく出るURLを開き直すこと。
Android実機から繋がらない場合はWindows FirewallでTCP 8770のインバウンドを許可すること。

## フォーマットの変え方（G4: コード変更不要。設計書v0.5）

フォーマットの正は常に `keymaps/keymap_<id>.json`（盤面グリッド／マニフェスト）＋
`keymaps/layers/<id>_layer<N>.json`（キー割当・1レイヤー=1ファイル）のJSONそのもの。
GUI・API・エディタは全て「このファイルを読み書きする道具」に過ぎない。

- スキーマv2（T7/D13）: マニフェストは `kind`・`halves`（split）か`board`（single、D24の
  グリッド式=`cols`・`row`/`col`/`colSpan`/`rowSpan`）・`layerFiles` のみを持ち、
  キー定義そのものは `keymaps/layers/<id>_layer<N>.json` に分割されている
- レイヤー: `mo`（押下中のみ）/ `tg`（トグル）。書式は `schemas/keymap.schema.json`・
  `schemas/layer.schema.json` と設計書D3/D4/D13
- `text`アクション（D20）: `{"t":"text","string":"…"}`。SendInputのKEYEVENTF_UNICODEで
  IME状態に依存せず文字列を直接注入する（記号レイヤー・全角/半角記号入力に使用）

既存フォーマット:

- `keymaps/keymap_writing01.json` ＋ `keymaps/layers/writing01_layer*.json` … **改造用**
- `keymaps/keymap_default.json` ＋ `keymaps/layers/default_layer*.json` … **初期値・リセット用。
  編集しない**。Deckの「Default戻し」でいつでもここへ戻れる
- `keymaps/keymap_ipad01_vol12.json` ＋ `keymaps/layers/ipad01_vol12_layer*.json` … iPad一枚
  キーボード（`/ipad`）専用。分割/Deckの切替中キーマップ(`active_keymap_id`)とは独立に固定で使われる

### 段階A: JSON直編集＋Hub再起動（実装済み）

1. PCでエディタから `keymaps/layers/<id>_layer<N>.json`（ラベル・action・レイヤー）や
   `keymaps/keymap_<id>.json`（盤面の形＝`board`/`halves`）を直接編集する
2. Hubを再起動（`start_hub.cmd`、または `cargo run -p proto-hub`）→ 反映
3. 不正なJSONは起動時にcode＋causeで拒否されるので、壊れたまま動くことはない
   （`proto-hub: startup rejected due to N error(s):` がコンソールに出て終了する）

### 段階B: ディレクトリスキャン＋再起動不要の反映（設計書v0.5 B1/B2）

- **B1 ディレクトリスキャン**: Hub起動時に `keymaps/` 直下の `keymap_*.json` を**全て**
  スキャンしてロードする（固定3ファイルのハードコードは廃止済み）。新しいフォーマットを
  追加したいときは、`keymap_<新id>.json` と対応する `layers/<新id>_layer*.json` を
  ディレクトリに置くだけでよい（拡張規則は `brief/keydeck_format_editing_design_v0.5.md`
  「拡張規則」節を参照。コード変更は不要）
- **B2 再読込API**: PCでJSONを編集した後、Hubを再起動せずに反映したい場合は
  `POST /api/reload?token=…` を叩く（設定画面 `http://<LAN-IP>:8770/settings?token=…`
  の「再読込」ボタンからも実行できる）。手順:
  1. Hubのディスク上の `keymaps/`・`decks/deck_default.json` を段階Aと同じ経路で再読込・検証する
  2. **検証が1件でも失敗すれば現行構成は一切変更されない**（HTTPは`422`＋
     `{"code":"RELOAD_INVALID","cause":"…"}` を返し、Hubコンソールにも
     D9書式の1行エラーログが出る。既存接続はそのまま動き続ける）
  3. 検証が全て成功した場合のみ差し替え、接続中の分割/Deck面・iPad面**両方**へ
     `surface.config` を再配信する（画面側は再接続不要で新しい割当に切り替わる）
  - token必須（他のAPIと同じ`?token=…`クエリ）。書込系ではなく読込専用のAPIなので
    D22の「書き込み系APIは`keymaps/layers/`配下のみ」の制限とは無関係（reloadはファイルを
    書かず読むだけ）

## Deckセットリスト（D11）

- Import: JSONを `decks/` に置いて再起動（`schemas/deck.schema.json` で検証される）
- Export: ブラウザで `/api/deck/export` を開くと現在のセットリストがダウンロードされる

## トラブルシュート（D9）

エラーは必ず「原因コード＋cause」付きで **Hubのコンソール** と **ブラウザのF12コンソール** の両方に出る。
`[KD][ERR][コード]` で検索。コード一覧は `brief/keydeck_design_v0.2.md` のD9。

## 動作確認方法

```
cargo test --workspace          # 全crateの自動テスト（proto-keymap 28件・proto-adapter-win 8件・proto-hub 11件・hub-core 7件＝54件）
cargo run -p proto-adapter-win --example smoke_notepad   # 実SendInput(key/chord/text)の手動smoke（メモ帳フォーカスで実行）
```

## ドキュメント

- 設計書（正）: `brief/keydeck_design_v0.2.md` ／ 状況: `DEVBOARD.md` ／ 差し戻し: `brief/spec_return_log.md`
