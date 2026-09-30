# 第11章　保存できる編集画面

## この章の問い

ブラウザから JSON を保存させると、「任意の場所にファイルを書ける」穴にならないか。

## 先に結論

書き込める場所は **3種類に固定** され、ファイル名は端末から受け取りません。
保存は「届いた本文をそのまま書く」ではなく、**解釈し直す → ID を確かめる → 控えを取る → 書く → 全体を読み直す → だめなら控えに戻す** の順です。
全体の読み直しには、第4章の起動時と **同じ検査** を使います。

---

## 画面で起きること

PC で Hub のトップ（`/`）を開くと、ボードの編集画面が出ます。区画を動かして保存すると、
接続中の iPad の画面が、再読み込みしなくても新しい並びに変わります。

保存に失敗したとき（たとえば区画が重なっていたとき）は、画面上部に理由が数秒表示され、
**ファイルは保存前の状態のまま** です。

---

## 書き込める場所は3種類だけ

| API | 書き先 | 書き換えてよいもの |
|---|---|---|
| `POST /api/layout/save` | `layouts/layout_<layoutId>.json` | ボード全体 |
| `POST /api/layer/save` | `keymaps/layers/` の下 | レイヤー1枚（キーの意味） |
| `POST /api/keymap/board/save` | `keymaps/keymap_<keymapId>.json` | **`board`（キーの位置）だけ** |
| `POST /api/keymap/new` | 上の2か所 | 新しいキーボード1枚 |

API は4本ありますが、書き先のフォルダは `layouts/`、`keymaps/layers/`、`keymaps/keymap_<id>.json` の3種類だけです。
これが不変条件6です。`apps/apps.json`（第12章）も、テーマや既定のボードの設定も、**ここには含まれません。**

---

## コード1 — ファイル名になる ID を厳しく確かめる

```rust
fn layout_id_is_safe(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}
```

出典: `crates/proto-hub/src/ws.rs` の `layout_id_is_safe`。省略なし。

使ってよい文字は **英小文字・数字・`_` の3種類、1〜64文字** です。
`..` も `/` も `\` も `:` も通りません。

この ID はファイル名の一部になります（`layout_<id>.json`）。
`CLAUDE.md` はこの関数を「**パスを組み立てる前の唯一の関門**であり、ここを緩めると任意パス書込になる」と書いています。
たとえば `-` を許すだけなら無害ですが、`.` や `\` を許した瞬間に `..\..\` で上のフォルダへ出られます。

---

## コード2 — 保存の手順

```rust
async fn layout_save_handler(
    State(state): State<SharedState>,
    Query(query): Query<TokenQuery>,
    body: String,
) -> Response {
    // ① token
    if !token_ok(&state, query.token.as_deref()) {
        return (StatusCode::UNAUTHORIZED, WS_TOKEN_INVALID).into_response();
    }

    // ② 本文をボードとして解釈し直す（重なり・はみ出し・区画idの重複はここで拒否）
    let layout = match crate::layout::load_layout_str("request body", &body) { /* … */ };

    // ③ ファイル名になる ID を確かめる
    if !layout_id_is_safe(&layout.layout_id) { /* 422 を返して終わり */ }

    let path = dir.join(format!("layout_{}.json", layout.layout_id));
    let backup = dir.join(format!("layout_{}.json.bak", layout.layout_id));

    // ④ 説明文を黙って消させない（本文に無ければ既存の説明を引き継ぐ）
    // …（省略）…

    // ⑤ 既存を退避。**巻き戻せる状態を作ってから書く**
    let had_previous = path.exists();
    if had_previous {
        if let Err(error) = std::fs::copy(&path, &backup) { /* 500 を返して終わり */ }
    }

    // ⑥ 受け取った本文をそのまま書かず、**解釈し直したものを整形して書く**。
    let text = match serde_json::to_string_pretty(&layout) { /* … */ };
    if let Err(error) = std::fs::write(&path, &text) { /* 500 を返して終わり */ }

    // ⑦ **書いた後に全体を読み直す。**
    let loaded = match crate::startup::load_startup_data(keymaps_dir, decks_dir, surfaces_dir, dir, apps_dir) {
        Ok(data) => data,
        Err(errors) => {
            // 巻き戻す。壊れた構成をディスクに残さない
            // …（省略）…
        }
    };
    // ⑧ 新しい構成を Hub に入れ、接続中の画面へ配る
    // …（省略）…
}
```

出典: `crates/proto-hub/src/ws.rs` の `layout_save_handler`。実物は約 150 行。ログ出力と各エラー応答の本体を省き、`①〜⑧` のコメントは説明のために足した（`**` で囲んだコメントは実物のもの）。**説明用に簡略化**。

| 手順 | 何を防いでいるか |
|---|---|
| ① token | 利用者以外の保存 |
| ② 解釈し直す | 形の壊れたボード、重なった区画 |
| ③ ID の確認 | 任意の場所への書き込み |
| ④ 説明文の引き継ぎ | 編集画面が送らない情報（運用メモ）が、保存のたびに消えること |
| ⑤ 控えを取る | 書いたあとに問題が見つかったとき、戻せなくなること |
| ⑥ 整形したものを書く | 余計な項目や書式の揺れがディスクに入ること |
| ⑦ 全体の読み直し | 単体では正しいが、他のファイルと合わない（無いキーマップを指す等）ボード |

### 関数カード　`layout_save_handler`

| 項目 | 内容 |
|---|---|
| 役割 | ボードを1枚保存し、全体の整合を確かめ、接続中の画面へ反映する |
| 呼ばれる場面 | 編集画面で保存したとき |
| 入力 | URL の token、本文（ボードの JSON） |
| 処理順 | token → 解釈 → ID → 説明文の引き継ぎ → 控え → 整形して書く → 全体の読み直し → 反映 |
| 出力 | 成功なら新しい構成。失敗なら `{code, cause}` の JSON と HTTP の状態コード |
| 失敗時 | ①〜③ならファイルに触らない。⑦で失敗したら控えから戻し、今の構成のまま動き続ける |
| 関連 | `layout_id_is_safe`、`layout::load_layout_str`、`startup::load_startup_data`（第4章） |
| たとえ | 書類の差し替え窓口。形式を確かめ、元の書類の写しを取り、差し替えたあと綴り全体を読み直し、合わなければ写しに戻す |

---

## なぜ保存専用の検査を作らないのか

保存のときだけ使う「軽い検査」を作ると、次のことが起きます。

```text
保存の検査は通った
  ↓
Hub を再起動した
  ↓
起動時の検査で拒否され、Hub が起動しない
```

**保存できたのに、次に起動できない。** これを防ぐには、保存と起動で同じ検査を使うしかありません。
`CLAUDE.md` にも「保存専用の緩い検査を作らない」と明記されています。

---

## 保存されないもの（正直に）

次の2つは、画面から変更できますが **Hub を再起動すると元に戻ります。**

- 見た目のテーマ（青・赤・緑）
- 最初に開くボードとキーボード（「default にする」）

これらはメモリの中にしか持っていません。保存するには4つ目の書き込み場所を許す判断が要り、
その判断はまだされていません（不変条件6の外側）。

---

## この章でできるようになったこと

- 保存 API がファイル名を受け取らない理由と、ID の文字を絞る理由を説明できる
- 控えを取ってから書き、全体を読み直して戻す流れを追える
- 保存と起動で同じ検査を使う理由を言える

## 扱わなかったこと

- 任意の場所への保存、ファイル名の指定（作らない）
- 編集画面の操作の作り（区画のドラッグ、取り消し）

## 確認問題

- **問 11-1**　`layout_id_is_safe` に `.` と `\` を許すと、何ができてしまいますか。
- **問 11-2**　書いた後に全体を読み直すのは、単体の検査では見つからないどんな問題のためですか。
- **問 11-3**　テーマを赤にして Hub を再起動すると、テーマはどうなりますか。

（答えは付録 E）
