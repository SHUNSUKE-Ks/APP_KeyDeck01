# 第III部　画面を部品から作る

# 第9章　部品と区画

## この章の問い

HTML を書き換えずに、ボードの並びを組み替えるにはどうするか。

## 先に結論

ボードは **12×9 のマス目** に **区画（section）** を置き、区画ごとに **部品（Component）を1つ** 指定します。
部品は4種類しかありません。**キーボード・Deck・トラックボール・ダイヤル。**
画面を描く JavaScript は `static/components.js` の1か所だけで、実機の画面と編集画面が同じものを使います。

---

## 画面で起きること

![主ボード ipad_main](images/board_ipad_main.png)

![YouTube 用ボード ipad_youtube](images/board_ipad_youtube.png)

2枚のボードは、同じ HTML（`static/layout.html`）で描かれています。
違うのは読み込む JSON だけです。
2枚目には、主ボードに無い **トラックボール** と **ダイヤル** が置かれています。

---

## ボードの JSON

```json
{
  "layoutId": "ipad_main",
  "grid": { "cols": 12, "rows": 9 },
  "sections": [
    { "id": "SEC-LEFT",     "row": 1, "col": 1,  "colSpan": 3,  "rowSpan": 4,
      "component": { "kind": "deck",     "ref": "story_paths" } },
    { "id": "SEC-CENTER",   "row": 1, "col": 4,  "colSpan": 6,  "rowSpan": 4,
      "component": { "kind": "deck",     "ref": "default" } },
    { "id": "SEC-RIGHT",    "row": 1, "col": 10, "colSpan": 3,  "rowSpan": 4,
      "component": { "kind": "keyboard", "ref": "dpad_arrows" } },
    { "id": "SEC-KEYBOARD", "row": 5, "col": 1,  "colSpan": 12, "rowSpan": 5,
      "component": { "kind": "keyboard", "ref": "ipad01_vol13" } }
  ]
}
```

出典: `layouts/layout_ipad_main.json`。`description`（運用メモの長文）を省き、1区画を2行に詰めた。**説明用に簡略化**。

| 項目 | 意味 |
|---|---|
| `row` `col` | 区画の左上がマス目のどこか |
| `colSpan` `rowSpan` | 何マス分の幅と高さか |
| `component.kind` | 部品の種類（4種類のどれか） |
| `component.ref` | 部品の中身の ID。キーボードならキーマップの ID、Deck なら Deck の ID |

**十字キーをトラックボールに取り替える** なら、`SEC-RIGHT` の `component` を `{"kind":"trackball","ref":"tb01"}` に書き換えるだけです。HTML は1行も触りません。

---

## 4種類の部品

| kind | 中身はどこにあるか | 押すと送るもの |
|---|---|---|
| `keyboard` | `keymaps/keymap_<ref>.json` | `key.press`（位置） |
| `deck` | `decks/deck_<ref>.json` | `deck.press`（位置） |
| `trackball` | `surfaces/*.json` の `id` | `surface.state`・`surface.gesture`（第10章） |
| `jog`（ダイヤル） | ダイヤル設定を持つキーマップ | 目盛りを越えるたびに `CW` か `CCW` の `key.press`（第10章） |

**十字キーも、電卓も、ラジアルメニューも、中身は「小さいキーボード」です。**
見た目が新しくても、たいていはこの4種類の組み合わせで作れます。

---

## コード1 — 種類で描き分ける

```javascript
function renderComponent(body, component, effectiveRef) {
  switch (component.kind) {
    case "keyboard": return renderKeyboard(body, effectiveRef || component.ref);
    case "deck":     return renderDeck(body, component.ref);
    case "trackball":return renderTrackball(body, component.ref);
    case "jog":      return renderJog(body, component.ref);
    default: {
      const oops = document.createElement("div");
      oops.className = "oops";
      oops.textContent = "未知の部品 '" + component.kind + "'";
      body.appendChild(oops);
      log("ERR", "LAYOUT_UNKNOWN_KIND", "unknown component kind", component);
    }
  }
}
```

出典: `static/layout.html` の `renderComponent`。省略なし。

| 行 | 読み方 |
|---|---|
| `switch (component.kind)` | 部品の種類を見る |
| `case "keyboard": …` | 種類ごとの描画関数に任せる。キーボードだけは、切り替え中のキーマップ（`effectiveRef`）を優先する |
| `default: { … }` | 知らない種類なら、黙って空にせず **「未知の部品」と画面に出し**、コードつきでログに残す |

### 関数カード　`renderComponent`

| 項目 | 内容 |
|---|---|
| 役割 | 区画の部品を、種類に応じた描画関数へ振り分ける |
| 呼ばれる場面 | Hub からボードの設定（`surface.config`）を受け取り、区画を並べるとき |
| 入力 | 描く場所（区画の中身の要素）、部品の指定、切り替え中のキーマップ ID |
| 処理順 | 種類を見る → 描画関数を呼ぶ → 知らない種類なら表示とログ |
| 出力 | 区画の中に部品が描かれる |
| 失敗時 | 未知の種類は「未知の部品」と表示。Hub の起動時検査（第4章）を通っているので通常は起きない |
| 関連 | `KDComponents.renderKeyboard` ほか（`static/components.js`）、`load_startup_data` の参照確認 |
| たとえ | 棚の組み立て図。「この枠は引き出し、この枠は扉」と、枠ごとに部品を当てはめる |

---

## コード2 — 正方形のマス目は、高さから大きさを決める

Deck のボタンは正方形です。CSS で「正方形」とだけ指定すると、**幅** から大きさが決まります。
すると、横長の区画に行数の多い Deck を置いたとき、下の段が区画からはみ出します。

```javascript
/// 正方形スロットは幅で大きさが決まるため、行数が多いと区画からはみ出す。
/// 区画の高さから1マスの上限を逆算する。
function fitDeck(host, grid, cols, rows, minCell) {
  var gap = 6;
  var floor = minCell === undefined ? 40 : minCell;
  var style = getComputedStyle(host);
  var padY = parseFloat(style.paddingTop) + parseFloat(style.paddingBottom);
  var avail = host.clientHeight - padY;
  var cell = Math.max(floor, (avail - gap * (rows - 1)) / rows);
  grid.style.maxWidth = (cell * cols + gap * (cols - 1)) + "px";
}
```

出典: `static/components.js` の `fitDeck`。省略なし。

| 行 | 読み方 |
|---|---|
| `avail = host.clientHeight - padY` | 区画の中で、実際に使える高さ |
| `(avail - gap * (rows - 1)) / rows` | 行と行のすき間を引いて、行数で割る ＝ 高さから見た1マスの上限 |
| `Math.max(floor, …)` | ただし 40px（既定）より小さくはしない。指で押せなくなるため |
| `grid.style.maxWidth = …` | 1マスの大きさ × 列数 ＋ すき間 を **全体の最大幅** にする。幅が決まれば、正方形の高さも決まる |

### 関数カード　`fitDeck`

| 項目 | 内容 |
|---|---|
| 役割 | 正方形のマス目が区画の高さに収まるよう、マス目全体の最大幅を決める |
| 呼ばれる場面 | Deck を描いたとき、区画の大きさが変わったとき |
| 入力 | 区画の要素、マス目の要素、列数、行数、1マスの最小値 |
| 処理順 | 使える高さを測る → 1マスの上限を出す → 最小値と比べる → 最大幅を設定 |
| 出力 | マス目の `max-width` |
| 失敗時 | 例外は出ない。最小値を優先するので、区画が極端に低いとはみ出す（それは JSON の区画設計で直す） |
| 関連 | `renderDeck` |
| たとえ | 箱にタイルを並べるとき、横幅だけでなく箱の深さも測ってからタイルの大きさを決める |

---

## 部品の描画を1か所にした理由

Hub の `router`（第1章）で `/components.js` を配る行には、次のコメントがあります。

> 部品の描画は static/components.js が唯一の実装。実機の面とエディタがこれを共有するので、プレビューと実機の絵がズレない。

編集画面のプレビューと、iPad の実機画面を別々のコードで描くと、いつか必ずずれます。
「編集画面では収まっていたのに、実機でははみ出す」は、利用者から見れば嘘の画面です。
**同じ絵を描く場所は1つにする。**

ビルドツールもフレームワークも使わず（不変条件4）、ただの `<script src="/components.js">` で読み込んでいます。

---

## 共有する部品を直すか、複製するか

キーマップ `ipad01_vol13` は、主ボードの下段に使われています。
これを別のボードでも使っていれば、中身を書き換えると **両方のボードが変わります。**

| やりたいこと | 選ぶ方法 |
|---|---|
| 誤字を直す、どのボードでも効いてほしい修正 | そのまま直す |
| このボードだけキー数や配置を変えたい | **別の ID に複製してから** 変える |

迷ったら複製します。KeyDeck では、完成した版（Vol）を凍結し、変えるときは新しい Vol を作る決まりがあります（第14章）。

---

## この章でできるようになったこと

- ボード・区画・部品・部品の中身の関係を説明できる
- HTML を触らずに部品を取り替える手順を言える
- 正方形のマス目を高さから決める理由を説明できる

## 扱わなかったこと

- 5種類目の部品を足す方法（Hub の型・検査・描画・外部向け資料を同時に直す必要がある）
- 編集画面での区画のドラッグ操作の実装（第11章は保存の側だけを扱う）

## 確認問題

- **問 9-1**　十字キーをトラックボールに取り替えるとき、書き換えるのは何ですか。
- **問 9-2**　`fitDeck` が幅ではなく高さから1マスの大きさを決めるのはなぜですか。
- **問 9-3**　編集画面と実機画面が同じ `components.js` を使うことで、何が防げますか。

（答えは付録 E）
