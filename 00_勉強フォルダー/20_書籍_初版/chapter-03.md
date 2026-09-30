# 第II部　1回の押下を追う

# 第3章　JSON が盤面の正体

## この章の問い

「このボタンを押したら何が起きるか」は、どこに書かれているのか。

## 先に結論

**すべて JSON に書かれています。** コードは JSON を読み、検査し、画面と Windows へつなぐ道具です。
JSON は「どこに置くか」と「押したら何が起きるか」を **別のファイルに分けて** 持っています。
だから、配置を変えても意味は変わらず、意味を変えても配置は変わりません。

---

## 画面で起きること

第1章の主ボードの左上にある `Fn` キーを押している間、数字の段が `F1`〜`F10` に変わります（第6章で実際の画面を見ます）。
この「Fn の位置」「Fn を押すとレイヤー1になる」「レイヤー1では数字の段が F キーになる」は、それぞれ別の JSON の別の行に書かれています。

---

## 変えたいもの → 開く JSON

| 変えたいもの | 開く JSON | 例 |
|---|---|---|
| キーの **意味**（押したら何が起きるか） | `keymaps/layers/<名前>_layer<N>.json` | A キーを B キーにする |
| キーの **位置**（盤面のどこに置くか） | `keymaps/keymap_<ID>.json` の `board` | Bksp を右上へ移す |
| Stream Deck 風ボタンの中身 | `decks/deck_<ID>.json` | 「保存」ボタンを足す |
| ボードの **区画割り**（部品の並び） | `layouts/layout_<ID>.json` | 十字キーをトラックボールにする |
| トラックボール面の出口 | `surfaces/*.json` | 2本指タップを Ctrl+V にする |
| 起動してよいアプリ | `apps/apps.json` | Unity を登録する（第12章） |

この表の左の列を1つ決めれば、開くファイルは1つに決まります。
**「配置を変えたつもりが、意味まで変わった」という事故は、ファイルが分かれていることで起きにくくなります。**

---

## 意味 — レイヤーの JSON

```json
{
  "layer": 0,
  "keys": {
    "K101": {"label": "Fn", "action": {"t": "mo", "layer": 1}},
    "K102": {"label": "1", "action": {"t": "key", "vk": "1"}},
    "K103": {"label": "2", "action": {"t": "key", "vk": "2"}}
  }
}
```

出典: `keymaps/layers/ipad01_vol12_layer0.json`。3キーだけ残し、説明文を省いた。**説明用に簡略化**。

```json
{
  "layer": 1,
  "keys": {
    "K102": { "label": "F1", "action": { "t": "key", "vk": "F1" } },
    "K103": { "label": "F2", "action": { "t": "key", "vk": "F2" } }
  }
}
```

出典: `keymaps/layers/ipad01_vol12_layer1.json`。2キーだけ残し、説明文を省いた。**説明用に簡略化**。

| 項目 | 意味 |
|---|---|
| `"K101"` | 位置の ID。「K ＋ 行 ＋ 列2桁」の決まりで、`K101` は1行目の1列目 |
| `"label"` | 画面に出す文字 |
| `"action"` | 押したら何が起きるか。`"t"` が種類 |
| `{"t": "mo", "layer": 1}` | 押している間だけレイヤー1にする（第6章） |
| `{"t": "key", "vk": "F1"}` | F1 キーを押す |

レイヤー1に `K101` は書かれていません。**書かれていないキーは、下のレイヤー（0）の意味がそのまま使われます**（第6章）。

---

## 位置 — キーマップの JSON

```json
{
  "keymapId": "ipad01_vol12",
  "board": {
    "cols": 13,
    "keys": [
      {"id": "K101", "row": 1, "col": 1},
      {"id": "K102", "row": 1, "col": 2},
      {"id": "K103", "row": 1, "col": 3}
    ]
  },
  "layerFiles": [
    "layers/ipad01_vol12_layer0.json",
    "layers/ipad01_vol12_layer1.json",
    "layers/ipad01_vol12_layer2.json",
    "layers/ipad01_vol12_layer3.json"
  ]
}
```

出典: `keymaps/keymap_ipad01_vol12.json`。`kind` と説明文を省き、`board.keys` を3つだけ残した。**説明用に簡略化**。

キーマップは **目次** です。どの位置にどの ID を置くか（`board`）と、意味の書かれたレイヤーのファイル（`layerFiles`）を並べています。
意味そのものは持っていません。

---

## コード — 目次を読み、そこからレイヤーを読む

```rust
/// ファイルパスからロード。マニフェストを読み、layerFilesをマニフェストと同じ
/// ディレクトリ基準の相対パスとして解決してすべて読み込み、結合してから検証する。
pub fn load_keymap_from_path(path: impl AsRef<Path>) -> Result<Keymap, KeymapError> {
    let path = path.as_ref();
    let manifest_text = std::fs::read_to_string(path).map_err(|error| {
        KeymapError::new(
            LOAD_JSON_SYNTAX,
            format!("{}: failed to read file: {error}", path.display()),
        )
    })?;
    let base_dir: PathBuf = path
        .parent()
        .map(|parent| parent.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let source = path.display().to_string();

    load_keymap_with(&source, &manifest_text, |relative| {
        let layer_path = base_dir.join(relative);
        std::fs::read_to_string(&layer_path).map_err(|error| {
            // …（どの目次の、どのレイヤーが読めなかったかを付けてエラーにする。省略）…
        })
    })
}
```

出典: `crates/proto-keymap/src/lib.rs` の `load_keymap_from_path`。レイヤーが読めなかったときのエラー文の組み立てを省いた。**説明用に簡略化**。

| 行 | 読み方 |
|---|---|
| `read_to_string(path)` | 目次（キーマップ）の JSON を読む。読めなければ `LOAD_JSON_SYNTAX` |
| `path.parent()` | 目次が置かれているフォルダを、レイヤーを探す基準にする |
| `load_keymap_with(…, \|relative\| { … })` | 目次を解釈し、`layerFiles` の1つずつについて、この小さな関数でファイルを読んでもらう |
| `base_dir.join(relative)` | 「目次のフォルダ ＋ `layers/…json`」でレイヤーの場所を作る |

### 関数カード　`load_keymap_from_path`

| 項目 | 内容 |
|---|---|
| 役割 | キーマップの目次と、そこに並んだレイヤーをすべて読み、1つのキーマップとして検査して返す |
| 呼ばれる場面 | 起動時と再読込時。`load_startup_data`（第4章）が、見つけたキーマップの数だけ呼ぶ |
| 入力 | 目次の JSON ファイルの場所 |
| 処理順 | 目次を読む → 基準のフォルダを決める → レイヤーを1枚ずつ読む → 結合する → 検査する |
| 出力 | `Keymap`、または `KeymapError`（コードと原因） |
| 失敗時 | 目次もレイヤーも、読めなければ `LOAD_JSON_SYNTAX`。どのファイルかが原因に入る |
| 関連 | `load_keymap_with`（検査の本体。テストはメモリ上の文字列でここを通す）、`load_startup_data` |
| たとえ | 本の目次を開き、「第2章は p.30」と書いてあるページを順にめくって、1冊に綴じ直す |

### なぜ「目次のフォルダ」を基準にするのか

もし「Hub を起動したフォルダ」を基準にすると、Hub をどこから起動したかでレイヤーが見つかったり見つからなかったりします。
目次自身の場所を基準にすれば、KeyDeck 一式を別のフォルダへ移しても、目次とレイヤーの関係は変わりません。

`load_keymap_with` が「ファイルを読む関数」を引数で受け取るのも工夫です。
テストではファイルの代わりに文字列を渡せるので、ディスクに触らず同じ検査を通せます。

---

## JSON は自由帳ではない

JSON で盤面を変えられるのは便利ですが、**何を書いてもよいわけではありません。**

- 知らない項目があれば拒否（`deny_unknown_fields`。書き間違いを黙って無視しない）
- `vk` に書けるキーの名前は辞書にあるものだけ（92種類）
- `layerFiles` は端末から絶対に受け取らない（ファイルの場所の一覧なので、受け取ると任意の場所を読めてしまう。第11章）

---

## この章でできるようになったこと

- 変えたいものから、開く JSON を1つに決められる
- キーマップ（目次＝位置）とレイヤー（意味）の分担を説明できる
- レイヤーの場所を、目次のフォルダを基準に探す理由を言える

## 扱わなかったこと

- 各 JSON の schema の全項目（`schemas/` にある）
- 分割キーボード（左右2台）の JSON の形

## 確認問題

- **問 3-1**　Bksp キーを盤面の別の場所へ移したいとき、開くのはレイヤーの JSON とキーマップの JSON のどちらですか。
- **問 3-2**　レイヤー1に書かれていないキーを、レイヤー1が有効なときに押すと、どの意味が使われますか。
- **問 3-3**　レイヤーのファイルの場所を「Hub を起動したフォルダ」基準で探すと、何が困りますか。

（答えは付録 E）
