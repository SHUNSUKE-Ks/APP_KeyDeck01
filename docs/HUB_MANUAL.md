# KeyDeck Hub 操作マニュアル

PC側（Hub）で盤面・部品・配置を変えるための手順書。**コードは一切書きません。JSONを書き換えるだけです。**

対象バージョン: 2026-09-05（P-005 段階B/C 実装後）

---

## 0. 最初に覚える3語

| 語 | 意味 | どこに書いてあるか |
|---|---|---|
| **section**（区画） | 画面の1区画。**中に部品をちょうど1つ**持つ | `layouts/layout_*.json` |
| **Component**（部品） | 部品の種類。`keyboard` / `deck` / `trackball` | 同上（section の中） |
| **slot**（スロット） | **Deck部品の中のボタン1つ** | `decks/deck_*.json` |

「部品を入れ替える」= section の中身を書き換えること。
「ボタンを増やす」= Deck の slot を足すこと。**別の話なので混同しないこと。**

---

## 1. Hubの起動と停止

```
起動: リポジトリ直下の start_hub.cmd をダブルクリック
停止: 開いたウィンドウを閉じる（Ctrl+C）
```

起動すると黒い画面に接続用URLが並びます。**tokenは起動のたびに変わります**（D8）。
「繋がらない」ときは、まずHubが生きているか・tokenが古くないかを疑ってください。

**端末（iPad/スマホ）から開くとき**は、PCで `http://<PCのIP>:8770/connect?token=…`
（QRギャラリー）を開き、目的のカード右上の「QR」を押してカメラで読むのが速いです。

> **2026-09-07 に画面構成が変わりました。**
> トップ `/` は**レイアウト編集**になり、QRは `/connect` の専用ページへ移りました。
> 起動時のコンソールに3本とも出ます（トップ／QRギャラリー／設定）。
**レイアウトは1つにつきQRが1枚**出るので、Vol1.2版・Vol1.3版・トラックボール版を
QRだけで切り替えられます（URLを手で打つ必要はありません）。

### ⚠️ 「PCでは変わったのに端末が変わらない」ときは、まずこれ

**Hubを再起動するとtokenが変わります（D8）。** 端末が古いURLを開いたままだと、
接続が拒否され続けて**新しい設定を受け取れません**。画面は残っているので、
一見「変わらないだけ」に見えます。

端末の画面右上に **「tokenが古い」** と出ていたらこれです。
**PCで `/connect` を開き直して、QRを読み直してください。**
（古いページを再読込しても直りません。URLに古いtokenが入っているためです）

| 面 | URL | 中身 |
|---|---|---|
| **レイアウト** | `/layout?id=ipad_main` | **部品を自由配置した面。これが主力**（キーボードはVol1.2） |
| レイアウト（Vol1.3） | `/layout?id=ipad_v13` | 同上。キーボードがGboard風のVol1.3 |
| レイアウト（ボール） | `/layout?id=ipad_trackball` | 同上。十字キーの代わりにトラックボール |
| 分割 | `/panel` | 上=Deck・下=キーボード（固定配置） |
| iPad一枚 | `/ipad` | キーボードだけ |
| Deck | `/deck?deck=<deckId>` | Deckだけ |
| トラックボール | `/trackball` | ボールだけ |
| 設定 | `/settings` | 再読込ボタン |

---

## 2. いちばん大事な操作: 変更を反映する

**JSONを書き換えたあと、Hubを再起動する必要はありません。**

```
方法A（推奨）: /settings をブラウザで開いて「再読込」ボタンを押す
方法B: PowerShellから
```

```bash
curl -X POST "http://localhost:8770/api/reload?token=<起動時に表示されたtoken>"
```

再読込は**全部が正しいときだけ**反映されます。1か所でも間違っていれば
**今の構成をそのまま維持して**エラーを返すので、書き間違えて画面が真っ白になることはありません。

失敗したときは、返ってきたJSONの `cause` を読んでください。「どのファイルの・どこが・なぜ駄目か」が書いてあります。

---

## 2-2. 端末の画面上で切り替える（JSONを触らずに）

レイアウト面（`/layout`）の**左上に2つのセレクタ**があります。**PCのHubを触らず、端末だけで切り替えられます。**

```
KeyDeck   BOARD [ ipad_main  ▾ ]   LAYOUT [ ipad01_vol13 ▾ ]
```

| セレクタ | 何が変わるか |
|---|---|
| **BOARD** | **画面を丸ごと差し替える。** `layouts/` に置いたレイアウトが全部出る。`layouts/` にファイルを足せば選択肢も自動で増える |
| **LAYOUT** | **キーボードの配列だけを差し替える。** 区画の配置（リスト・Deck・十字キーの位置）は変わらない |

- **LAYOUTは文字入力キーボードにしか効きません。** 十字キーのような小さな盤面は
  選択肢にも出ませんし、差し替えの対象にもなりません（区画ごと入れ替わってしまわないため）
- **LAYOUTセレクタは、いま出している画面に文字入力キーボードがあるときだけ表示されます**
- 選んだ内容は**その端末が覚えます**。次に開いたときも同じ組み合わせで出ます
- BOARDを変えるとURLにも反映されるので、そのURLを共有すればいきなりその画面を開けます
- **`layouts/*.json` は書き換わりません。** これは「この端末ではこう出す」という表示上の選択です。
  全端末の既定を変えたいときは §3 のとおりJSONを書き換えてください

例: 「配置は普段どおり（`ipad_main`）だが、キーボードだけVol1.3で試したい」は、
BOARDを `ipad_main`、LAYOUTを `ipad01_vol13` にするだけです。

---

## 3. 部品を入れ替える（パズル）

`layouts/layout_ipad_main.json` を開きます。

```json
{
  "layoutId": "ipad_main",
  "grid": { "cols": 12, "rows": 9 },
  "sections": [
    { "id": "SEC-LEFT",     "row": 1, "col": 1,  "colSpan": 3, "rowSpan": 4,
      "component": { "kind": "deck", "ref": "story_paths" } },
    { "id": "SEC-CENTER",   "row": 1, "col": 4,  "colSpan": 6, "rowSpan": 4,
      "component": { "kind": "deck", "ref": "default" } },
    { "id": "SEC-RIGHT",    "row": 1, "col": 10, "colSpan": 3, "rowSpan": 4,
      "component": { "kind": "keyboard", "ref": "dpad01" } },
    { "id": "SEC-KEYBOARD", "row": 5, "col": 1,  "colSpan": 12, "rowSpan": 5,
      "component": { "kind": "keyboard", "ref": "ipad01_vol12" } }
  ]
}
```

画面を **12列 × 9行** のマス目とみなし、各区画がどこからどこまでを占めるかを書いています。

- `row` / `col` … 左上の位置（**1から数える**）
- `rowSpan` / `colSpan` … 何マス分の広さか（省略すると1）

### 3-1. 左右を入れ替える

`SEC-LEFT` と `SEC-RIGHT` の **`col` だけ**を交換します（`1` ⇄ `10`）。再読込。

### 3-2. 十字キーをトラックボールに取り換える 🎯

`SEC-RIGHT` の `component` の行を、こう書き換えるだけです。

```json
"component": { "kind": "trackball", "ref": "tb01" }
```

再読込すると、その区画がボールに変わります。**戻すときは逆に書き換えるだけ。**

もっと手軽な方法として、**入れ替え済みのレイアウトを用意してあります**。

```
十字キー版:       /layout?id=ipad_main
トラックボール版: /layout?id=ipad_trackball
```

URLを変えるだけで切り替わります（Hubの再読込すら不要）。
普段どちらを使うか決まったら、その内容を `ipad_main` に写してしまうのが楽です。

### 3-3. 区画の広さを変える

`colSpan` / `rowSpan` を増減します。**合計がグリッドを超えないように**してください。
超えると再読込が「outside the 12x9 grid」と言って拒否します。

### 3-4. よくある失敗と、そのときのメッセージ

| やってしまうこと | 再読込が返すメッセージ | 直し方 |
|---|---|---|
| 区画が画面からはみ出す | `... spans to (10,13) which is outside the 12x9 grid` | `col`＋`colSpan`が13以上になっていないか確認 |
| 区画が重なる | `sections 'A' and 'B' both occupy cell (row 1, col 4)` | どちらかの `col`/`colSpan` をずらす |
| 存在しない部品を指す | `component references unknown id 'dpad02'` | `ref` の綴りを確認（一覧は §7） |
| `id` が重複 | `duplicate section id 'SEC-LEFT'` | section の `id` は全部違う名前にする |

**重なりは画面上「片方が消えた」ようにしか見えません。** だからHubが読み込みの時点で止めます。

---

## 4. Stream Deckのボタンを変える

`decks/deck_default.json` を開きます。

```json
{
  "deckId": "default",
  "grid": { "cols": 8, "rows": 2 },
  "render": "grid",
  "pages": [ { "id": 1, "slots": [
    { "slotId": "S01", "label": "消音", "action": { "t": "key", "vk": "MUTE" } }
  ] } ]
}
```

### 4-1. スロットの数を変える

`grid` の `cols`（列）と `rows`（段）を書き換えます。**これが今回いちばん要望のあった操作です。**

```
縦2段・横8列 → "grid": { "cols": 8, "rows": 2 }   （容量16）
縦3段・横5列 → "grid": { "cols": 5, "rows": 3 }   （容量15）
```

**スロットの数が `cols × rows` を超えると再読込が拒否します**（`capacity` を含むメッセージ）。
これは「置いたはずのボタンが黙って消える」事故を防ぐためです。減らすときは `slots` からも消してください。

### 4-2. ボタンに割り当てられる操作

| やりたいこと | `action` の書き方 |
|---|---|
| キーを1回押す | `{ "t": "key", "vk": "MUTE" }` |
| ショートカット | `{ "t": "chord", "keys": ["CTRL", "S"] }` |
| 決まった文字を打ち込む | `{ "t": "text", "string": "お世話になっております" }` |
| 盤面を切り替える | `{ "t": "keymap.switch", "id": "writing01" }` |
| 空き | `{ "t": "none" }` |

`vk` に書ける名前は決まっています（§7）。知らない名前を書くと再読込が
`unknown vk 'XXX'` と言って拒否します。

**Deckに `key.hold` は置けません**（押しっぱなしは「離す」合図が要りますが、Deckのボタンには
それがないため、押しっぱなしのまま戻れなくなります）。再読込が理由を添えて拒否します。

### 4-3. コピペリストを増やす

`decks/deck_story_paths.json` が、押すとフォルダのパスを打ち込むリストです。

```json
{
  "deckId": "story_paths",
  "grid": { "cols": 1, "rows": 2 },
  "render": "list",
  "pages": [ { "id": 1, "slots": [
    { "slotId": "S01", "label": "00_inbox",
      "action": { "t": "text", "string": "C:\\Users\\enjoy\\Dropbox\\..." } }
  ] } ]
}
```

**項目を足すときは `slots` に足して、`grid.rows` も一緒に増やしてください**（3件にするなら `rows: 3`）。

- `"render": "list"` … 横いっぱいの縦リストで描く（コピペリスト向き）
- `"render": "grid"` … 正方形のマス目で描く（Stream Deck向き）。省略時はこちら
- パスの `\` は JSON では `\\` と2つ重ねて書きます
- 打ち込みは**クリップボードを経由しません**。いまカーソルがある場所に直接入ります

### 4-4. Deckを新しく作る

`decks/` に `deck_なんとか.json` という名前で置いて再読込するだけで見つかります。
`deckId` は他と重複しないようにしてください。作ったら `layouts/` の `ref` から指名します。

---

## 5. 十字キーの割り当てを変える

`keymaps/layers/dpad01_layer0.json`:

```json
{
  "layer": 0,
  "keys": {
    "D102": { "label": "↑", "action": { "t": "key.hold", "vk": "W" } },
    "D201": { "label": "←", "action": { "t": "key.hold", "vk": "A" } },
    "D203": { "label": "→", "action": { "t": "key.hold", "vk": "D" } },
    "D302": { "label": "↓", "action": { "t": "key.hold", "vk": "S" } }
  }
}
```

### ⚠️ WASD版は「文字が出ます」

`vk` の `W` `A` `S` `D` は**英字キーそのもの**です。ゲーム中なら移動になりますが、
**テキスト欄にカーソルがあると `wasd` と打ち込まれます。** ラベルが `↑←→↓` でも中身は英字キーです。

文字を出したくないときは**矢印キー版**に差し替えてください。盤面の形は同じで `vk` だけが違います。

```
layouts/layout_ipad_main.json の SEC-RIGHT を、こう書き換えて再読込:
  "component": { "kind": "keyboard", "ref": "dpad_arrows" }
```

| 使うもの | `ref` | 送るキー | 文字が出るか |
|---|---|---|---|
| WASD版（ゲーム移動向き） | `dpad01` | W / A / S / D | **出る** |
| 矢印版（どこでも安全） | `dpad_arrows` | UP / LEFT / RIGHT / DOWN | 出ない |
| トラックボール | `tb01`（kind は `trackball`） | マウス移動 | 出ない |

`dpad01` の `vk` を直接書き換えても同じ結果になります。

- `key.hold` は**押している間ずっと押され続ける**（ゲームの移動用）
- `key` にすると**1回押して離すだけ**になります
- 押している間はボタンが緑色になります。**離し忘れに気づくため**の色です
- 万一押したまま画面を閉じたり電波が切れても、**Hubが自動で離します**のでPCが操作不能にはなりません

---

## 5-2. キーボードの世代を切り替える（Vol1.2 / Vol1.3）

キーボードは**世代ごとに別ファイル**にしてあります。**既存の盤面は編集せず、複製して新しい世代を作る**運用です。
以前「キーボードを直したらレイアウトが崩れた」ことがあったための備えで、片方を壊してももう片方は無傷で残ります。

| 世代 | `ref` | 中身 | URL |
|---|---|---|---|
| **Vol1.2** | `ipad01_vol12` | 数字段あり。Tab / Ctrl / Shift / Fn あり。汎用 | `/layout?id=ipad_main` |
| **Vol1.3** | `ipad01_vol13` | Gboard風の日本語配列。数字はQ〜Pの右肩ヒント＋記号盤。確定キー・矢印キーあり | `/layout?id=ipad_v13` |

**URLを変えるだけで切り替わります**（Hubの再読込も不要）。
常用する方を決めたら、`layouts/layout_ipad_main.json` の `SEC-KEYBOARD` の `ref` を書き換えてください。

### Vol1.3で気をつけること

- **数字はそのままでは打てません。** Q〜Pの右肩の数字は「記号盤の同じ位置にある」という**ヒント表示**です。
  数字を打つときは左下の **記号** を押してから、同じ位置のキーを押します
  （スマホのフリック入力にあたる機能はKeyDeckにはありません）
- **Tab / Ctrl / Shift がありません。** 日本語入力に振り切った盤面です。必要なときは Vol1.2 に戻してください
- **確定** は Enter を送ります（IMEの変換確定という意味づけ）
- 画面上部の予測変換（「こんばんわ / 言葉を …」）は**PC側のIMEが出すもの**で、KeyDeck側では出しません

---

## 6. キーボードの盤面を変える

`keymaps/layers/ipad01_vol12_layer0.json` の `label` と `action` を書き換えます。
キーの**位置**を変えたいときは `keymaps/keymap_ipad01_vol12.json` の `board` 側（`row`/`col`）です。

レイヤー構成:

| # | 役割 | 入り方 |
|---|---|---|
| 0 | 基盤 | 常時 |
| 1 | Fn（F1〜F12） | Fnキーを押している間 |
| 2 | 記号盤 | 「記号」キー |
| 3 | 日本語モード表示 | 「英数⇄日本語」キー（同時にPCのIMEも切り替わる） |

⚠️ **レイヤー番号は大きい方が勝ちます。** layer3 に layer2 と同じキーを足すと記号盤が壊れます。
詳しくは `keymaps/layers/README.md` と、各ファイル冒頭の `description` を読んでください。

---

## 7. 名前の一覧（`ref` と `vk`）

### 部品として指名できるもの

| `kind` | `ref` に書ける名前 | 実体 |
|---|---|---|
| `keyboard` | `ipad01_vol12`（Vol1.2） / `ipad01_vol13`（Vol1.3・Gboard風） / `dpad01`（WASD） / `dpad_arrows`（矢印） / `writing01` / `default` | `keymaps/keymap_*.json` の `keymapId` |
| `deck` | `default` / `story_paths` | `decks/deck_*.json` の `deckId` |
| `trackball` | `tb01` / `tb01-scroll` | `surfaces/trackball.json` の `id` |

**増やしたらこの表も増えます。** 正確な一覧はHub起動時のログ（`keymaps=4 decks=2 surfaces=2 layouts=2`）と、
各ディレクトリのファイルを見てください。

### `vk` に書ける名前

```
A〜Z / 0〜9 / F1〜F24
ENTER ESC TAB SPACE BKSP DEL
UP DOWN LEFT RIGHT
CTRL SHIFT ALT WIN
COMMA PERIOD SLASH SEMICOLON QUOTE MINUS EQUALS
LBRACKET RBRACKET BACKSLASH GRAVE
VOL_UP VOL_DOWN MUTE MEDIA_PLAY MEDIA_NEXT MEDIA_PREV
```

これ以外は再読込が拒否します。日本語や記号を「打ち込みたい」場合は `vk` ではなく
`{ "t": "text", "string": "…" }` を使ってください。

---

## 8. 困ったとき

| 症状 | まず疑うこと |
|---|---|
| 端末から繋がらない | Hubが起動しているか / **tokenが古くないか**（起動のたびに変わります）。端末に「tokenが古い」と出ていたらQRを読み直す |
| PCでは変わったのに端末が変わらない | ①端末の右上バッジが「tokenが古い」なら**QRを読み直す** ②そうでなければ**端末のキャッシュ**。Hubは `Cache-Control: no-store` を送るようになったので通常は起きないが、**2026-09-05より前に開いたページが端末に残っている場合は一度だけ**URL末尾に `&cb=1` を足して開き直すと抜けられる |
| 書き換えたのに変わらない | 再読込したか（JSONの変更は `/settings` の再読込ボタン） |
| 再読込が拒否される | 返ってきた `cause` を読む。どのファイルのどこが悪いか書いてあります |
| 区画が1つ消えた | 区画が重なっている可能性。再読込のメッセージを確認 |
| キーが押されっぱなし | どこかのキーを1回押せば直ります。端末を閉じてもHubが自動で離します |
| 十字キーで文字が入ってしまう | WASD版は英字キーを送ります（仕様）。`dpad_arrows` に差し替えてください（§5） |
| 全部おかしくなった | `git status` で変更を確認し、`git checkout -- <ファイル>` で戻す |

## 9. 触ってはいけないもの

- `keymaps/keymap_default.json` … リセット用の原本
- `crates/` 配下 … コード
- `brief/` 配下の設計書・モック

これらは触らずに、`layouts/` `decks/` `keymaps/layers/` の3つで大体のことができます。
