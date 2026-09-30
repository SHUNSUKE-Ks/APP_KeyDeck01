# 第4章　起動時に全部検査する

## この章の問い

JSON が1か所でも壊れていたら、Hub はどうするべきか。

## 先に結論

**1つでも壊れていたら、何も使わずに止まります。** 壊れた部分だけを捨てて残りで動く、ということをしません。
検査はすべてのファイルを読み終えてから行い、見つかったエラーは **まとめて全部** 報告します。
許可リスト（第2章）もこの検査を通ったあとで作られます。

---

## 画面で起きること — 正常なとき

Hub を起動すると、コンソールに次の2行が出ます（2026-09-15 の実際の出力から、時刻と色の制御文字を除いたもの）。

```text
INFO proto_hub: startup data loaded successfully chk="T3-1" keymaps=13 decks=2 surfaces=2 layouts=2 apps=1
INFO proto_hub: allow-list built chk="T3-1" allowed_commands=139
```

## 画面で起きること — 壊れているとき

キーマップのボタンが、登録されていないアプリを指していた場合（2026-09-12 に意図的に作って確かめた例）。

```text
[APP_LAUNCH_UNKNOWN] keymap 'zzztest' layer 0 key 'T1': app.launch references unknown app id 'nosuchapp' (add it to apps/apps.json)
```

Hub は起動せずに終了します。**どのファイルの、どのレイヤーの、どのキーが悪いか** まで名指しされています。

---

## コード — 読み込みと検査の流れ

```rust
pub fn load_startup_data(
    keymaps_dir: &Path,
    decks_dir: &Path,
    surfaces_dir: &Path,
    layouts_dir: &Path,
    apps_dir: &Path,
) -> Result<StartupData, Vec<String>> {
    let mut errors: Vec<String> = Vec::new();

    // ① キーマップを全部読む。失敗しても止まらず、エラーを貯める
    for path in &paths {
        match proto_keymap::load_keymap_from_path(path) {
            Ok(keymap) => { keymaps.insert(keymap.keymap_id.clone(), keymap); }
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }
    // ② Deck、③ 面、④ ボード、⑤ 起動許可アプリも同じ形で読む
    // …（省略）…

    // ⑥ 参照先の実在確認。全部読み終えた今しかできない
    if errors.is_empty() {
        for layout in layouts.values() {
            // 区画が指すキーマップ・Deck・面が実在するか
            // …（省略）…
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    // ⑦ ここまで来て、はじめて許可リストを作る
    let command_ids: Vec<String> = all_actions(&keymaps, &decks)
        .filter_map(canonical_command_id)
        .collect();
    let command_registry = hub_core::CommandRegistry::new(command_ids);
    // …（省略）…
}
```

出典: `crates/proto-hub/src/startup.rs` の `load_startup_data`。実物は約 190 行。Deck・面・ボード・アプリの読み込みと参照確認の本体を省き、`①〜⑦` の日本語コメントは説明のために足した。**説明用に簡略化**。

| 行 | 読み方 |
|---|---|
| `Result<StartupData, Vec<String>>` | 成功なら読み込んだもの一式、失敗なら **エラーの一覧**（1件ではない） |
| `errors.push(…)` | 失敗してもその場で止まらない。全部読んでから報告するため |
| `if errors.is_empty() { … 参照確認 … }` | 読み込みで既に失敗しているなら、参照確認はしない（壊れた前提で確認しても意味のないエラーが増えるだけ） |
| `return Err(errors)` | 1件でもあれば、読んだものは全部捨てる |
| `all_actions(…).filter_map(canonical_command_id)` | 全キー・全ボタンの操作から照合札を作る。`None`（Windows に届かない操作）は除かれる |
| `hub_core::CommandRegistry::new` | 許可リストの本体。**この型は hub-core（外部から取り込んだ凍結コピー）のもの** |

### 関数カード　`load_startup_data`

| 項目 | 内容 |
|---|---|
| 役割 | すべての JSON を読み、単体の検査と、ファイル同士の参照の検査を行い、使える一式を返す |
| 呼ばれる場面 | ① Hub の起動時 ② 設定画面からの再読込（`/api/reload`）③ 編集画面で保存した直後（第11章） |
| 入力 | 5つのフォルダ（キーマップ・Deck・面・ボード・起動許可アプリ） |
| 処理順 | キーマップ → Deck → Deck 内の `keymap.switch` の参照 → 面 → ボード → アプリ → ボードの区画の参照 → `app.launch` の参照 → 許可リスト |
| 出力 | `StartupData`（キーマップ、Deck、ボード、許可リスト、面、アプリ） |
| 失敗時 | エラー文字列の一覧を返す。各行は `[コード] どこで: 何が` の形 |
| 関連 | `proto_keymap::load_keymap_from_path`（第3章）、`canonical_command_id`（第2章） |
| たとえ | 出発前の改札。全員の切符と行き先を確かめ、1人でも合わなければ列車を出さない |

---

## なぜ部分適用しないのか

壊れた部分だけ捨てて動く設計にすると、次のことが起きます。

```text
ボード ipad_main が、キーマップ foo を指している
キーマップ foo の JSON に書き間違いがある
  ↓ foo だけ捨てて起動した
端末でボードを開くと、下半分が空っぽ
  ↓
利用者から見えるのは「キーボードが消えた」だけ
原因（foo の書き間違い）は、コンソールのどこかに1行流れて消えている
```

**症状が出る場所と、原因がある場所が遠い** と、調べるのに時間がかかります。
起動の時点で止まれば、症状と原因は同じ画面に並びます。

参照の検査を「押したとき」ではなく「起動時」にしているのも同じ理由です。
第12章のアプリ起動では、`app.launch` の参照先を起動時にまとめて確かめています。
押した瞬間に「そんなアプリは無い」と分かるのでは、原因が遠すぎるからです。

---

## 読み込み時の検査で止めているもの（抜粋）

| 検査 | エラーコード | どこで |
|---|---|---|
| JSON の書式が壊れている | `LOAD_JSON_SYNTAX` など | 各ファイルの読み込み |
| 同じ ID のボードや Deck が2つある | `LOAD_LAYOUT_INVALID` など | `load_startup_data` |
| ボードの区画が、無いキーマップ・Deck・面を指している | `LOAD_LAYOUT_REF_UNKNOWN` | `load_startup_data` |
| ダイヤル部品が、ダイヤル設定を持たないキーマップを指している | `LOAD_LAYOUT_REF_UNKNOWN` | 同上 |
| ボタンが、登録されていないアプリを指している | `APP_LAUNCH_UNKNOWN` | 同上 |
| 区画が重なっている・はみ出している | ボードの読み込みで拒否 | `layout.rs` |

---

## この章でできるようになったこと

- 「再読込」が、ファイルを読み直すだけでなく全体の検査であることを説明できる
- 起動時にまとめて止める設計が、原因を探す時間を短くする理由を言える

## 扱わなかったこと

- 壊れた設定を自動で直す機能（KeyDeck には無い。直すのは人）
- 各エラーコードの完全な一覧（`crates/proto-hub/src/error.rs` と各 crate の定数にある）

## 確認問題

- **問 4-1**　読み込みで失敗したとき、`load_startup_data` がその場で止まらずエラーを貯めるのはなぜですか。
- **問 4-2**　ボードが存在しないキーマップを指していたとき、それが見つかるのは「起動時」と「端末で開いたとき」のどちらですか。
- **問 4-3**　許可リストが、全部の検査が終わったあとに作られるのはなぜですか。

（答えは付録 E）
