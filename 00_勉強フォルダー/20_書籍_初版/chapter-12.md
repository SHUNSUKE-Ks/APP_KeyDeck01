# 第IV部　境界を広げるとき

# 第12章　アプリ起動を足す — 任意コマンド実行にしない

## この章の問い

「ボタンを押したらアプリが起動する」機能を、決めた境界を壊さずに足すにはどうするか。

## 先に結論

**端末が送れるのは、登録したアプリの ID だけ** にします。
実行ファイルの場所も引数も PC 側の `apps/apps.json` が持ち、端末からは一切受け取りません。
シェルを通さず、`.exe` だけを、絶対パスで起動します。
この7つの条件は利用者の裁定を経て **不変条件7** になりました。**1つでも緩めた瞬間に、任意コマンド実行になります。**

---

## きっかけ

Unity でゲームを作る作業が始まり、「ボタン1つで Unity を起動して、同時に Unity 用のボードへ切り替えたい」という要望が出ました。

その時点で考えられた作り方は2つありました。

| 作り方 | 中身 | 判断 |
|---|---|---|
| A. 既存の機能だけで作る | タスクバーの N 番目のアプリを前に出す `Win+数字` のショートカットを置く | 境界に触らない。ただしアプリが起動していないと何も起きない |
| B. 許可リスト式の起動 | 登録したアプリだけを起動する新しい操作 `app.launch` | 本当に起動できる。**不変条件6「任意コマンド実行 API は作らない」に触れる** |

B は境界に触れるので、AI が勝手に選んではいけない判断です。利用者に2案を示し、利用者が B を選びました（2026-09-12）。
**この章は、その B をどう「任意」にしないで作ったかの記録です。**

---

## 危険はどこにあるのか

「アプリを起動する API」を素朴に作ると、こうなります。

```text
端末 → {"type":"launch", "path":"C:\\Program Files\\<アプリ>\\<アプリ>.exe", "args":"..."}
```

これは端末が **何でも** 起動できる API です。`path` に別のプログラムを書けば、それが動きます。
第2章の「token が漏れても、被害の上限はボードに置いたものの合計」が崩れます。

---

## 7つの条件

| # | 条件 | 緩めると何が起きるか |
|---|---|---|
| 1 | パスも引数も端末から受け取らない。送れるのは登録済みの `id` だけ | 端末が何でも起動できる |
| 2 | **シェルを通さない**（`cmd /C` や `start` を使わない） | 引数の `&` や `\|` が、別のコマンドとして実行される |
| 3 | `.exe` だけ。`.bat` `.cmd` `.ps1` は拒否 | スクリプトは中身に1行足すだけで、任意のコマンドを実行できる |
| 4 | 絶対パスだけ | 相対パスは Hub の作業フォルダ次第で、別のファイルを掴む |
| 5 | 端末へ出すのは `id` と `label` だけ | PC の中のフォルダ構成を教えることになる |
| 6 | 参照先（どの `id` を押すか）は起動時にまとめて確かめる | 押した瞬間に初めて「無い」と分かり、原因が遠い |
| 7 | 実行ファイルがあるかは、起動時と押したときの両方で確かめる | 後で消した・動かしたときに、押しても何も起きない嘘のボタンが残る |

---

## 登録の書き方

```json
{
  "apps": [
    {
      "id": "memo",
      "label": "メモ帳",
      "exe": "C:\\<Windows のフォルダ>\\System32\\notepad.exe",
      "args": []
    }
  ]
}
```

出典: `apps/apps.json` の書式。実物には説明文の項目があり、`args` は省略されている。`exe` の値は伏字にした。**説明用に簡略化**。
実物には、仕組みが生きているかを試すための見本として、Windows に付属するメモ帳が1件だけ登録されている。

ボードのボタン側は、ID だけを書きます。

```json
{ "label": "メモ帳", "action": { "t": "app.launch", "id": "memo" } }
```

---

## コード1 — 起動する

```rust
pub fn launch(registry: &AppRegistry, id: &str) -> Result<(), AppError> {
    let Some(def) = registry.get(id) else {
        return Err(AppError::new(
            APP_LAUNCH_UNKNOWN,
            format!("app '{id}' is not in apps/apps.json"),
        ));
    };
    // 起動時に在っても、その後に消された／動かされた可能性がある。押された時に見る。
    if !Path::new(&def.exe).is_file() {
        return Err(AppError::new(
            APP_LAUNCH_FAILED,
            format!("app '{id}': {} does not exist", def.exe),
        ));
    }
    std::process::Command::new(&def.exe)
        .args(&def.args)
        .spawn()
        .map(|_| ())
        .map_err(|error| {
            AppError::new(
                APP_LAUNCH_FAILED,
                format!("app '{id}': failed to start {}: {error}", def.exe),
            )
        })
}
```

出典: `crates/proto-hub/src/app_launch.rs` の `launch`。省略なし。

| 行 | 読み方 |
|---|---|
| `registry.get(id)` | 登録簿から `id` を引く。**端末から来るのはこの `id` だけ** |
| `else { … APP_LAUNCH_UNKNOWN … }` | 登録に無ければ断る |
| `Path::new(&def.exe).is_file()` | 押された今も、そのファイルがあるか（条件7） |
| `Command::new(&def.exe).args(&def.args)` | 実行ファイルを **直接** 起動する。シェルを通さないので、引数の中の記号はただの文字（条件2） |
| `.spawn()` | 起動したら終わるのを待たない。Hub は次の入力を受け付け続ける |

### 関数カード　`launch`

| 項目 | 内容 |
|---|---|
| 役割 | 登録済みのアプリを1つ、シェルを通さず起動する |
| 呼ばれる場面 | `app.launch` のボタンが押され、`fire_action` がこの操作を実行するとき |
| 入力 | 登録簿（起動時に `apps/apps.json` から作ったもの）、アプリの `id` |
| 処理順 | 登録簿を引く → ファイルの実在 → 直接起動 |
| 出力 | 成功、または `AppError`（コードと原因） |
| 失敗時 | 未登録は `APP_LAUNCH_UNKNOWN`、ファイルが無い・起動できないは `APP_LAUNCH_FAILED`。Hub は止まらず、押した端末にエラーが返る |
| 関連 | `load_app_registry`（登録簿の読み込みと検査）、`fire_action`、`load_startup_data` の参照確認 |
| たとえ | 鍵のかかった道具棚。利用者は棚の番号を言うだけで、棚の中身と置き場所は管理人しか知らない |

---

## コード2 — スクリプトを拒否することをテストで固定する

```rust
#[test]
fn refuses_script_interpreters() {
    for exe in [
        "C:\\tools\\run.bat",
        "C:\\tools\\run.cmd",
        "C:\\tools\\run.ps1",
        "C:\\tools\\RUN.BAT",
    ] {
        let text = format!(
            r#"{{ "apps": [ {{ "id": "x", "label": "x", "exe": "{}" }} ] }}"#,
            exe.replace('\\', "\\\\")
        );
        let err = load(&text).expect_err("script files must be refused");
        assert_eq!(err.code, LOAD_APP_EXE_INVALID, "exe={exe}");
        // …（省略）…
    }
}
```

出典: `crates/proto-hub/src/app_launch.rs` のテスト `refuses_script_interpreters`。末尾を省いた。**説明用に簡略化**。

大文字の `RUN.BAT` まで試しているのは、「拡張子の比較で大文字小文字を区別していた」という間違いを後から誰かが入れても、テストが落ちるようにするためです。
**条件を文章で書くだけでなく、テストで固定する。** 将来この部分を触る人（や AI）が、条件を知らずに緩めることを防げます。

同じファイルには、他に次のテストがあります。

```text
refuses_relative_exe                               相対パスを拒否する
refuses_bad_id                                     ID の文字を絞る
refuses_duplicate_id                               同じ ID を2つ登録できない
refuses_unknown_fields                             知らない項目があれば拒否
launching_an_unregistered_id_is_refused            未登録の ID は起動しない
launching_a_missing_exe_is_refused_without_spawning ファイルが無ければ起動を試みない
missing_file_is_not_an_error                       apps.json が無くても Hub は起動する
```

---

## 照合札にも載せる

第2章の `canonical_command_id` に、次の1行が足されています。

```rust
// アプリ起動も**キーと同じ強さの許可リストに載せる**。載せないと、
// 起動時に読んだキーマップに書かれていないものが撃てる余地が残る。
Action::AppLaunch { id, .. } => Some(format!("app:{id}")),
```

出典: `crates/proto-hub/src/state.rs` の `canonical_command_id` の中。省略なし。

---

## 確かめたこと（2026-09-12）

| 確かめたこと | 結果 |
|---|---|
| 一時的なキーマップに `app.launch` のボタンを置き、実際の WebSocket の `key.press` で押す | Hub のログに `outcome=app:memo` と `app launched`。**メモ帳が実際に起動した** |
| 同じボタンを離す | `outcome=ignored`（押した瞬間に1回だけ） |
| ボタンが未登録の ID を指すようにする | Hub が起動を拒否し、どのキーマップのどのキーかを名指しした（第4章の例） |
| `/api/schema` の応答 | アプリは `id` と `label` だけが出ており、`exe` は出ていない |

確認に使った一時的なキーマップは、確認後に削除しています。

---

## 起動のあとに、ボードを切り替える

`app.launch` には、起動のあとに続けて実行する操作（`fire`）を1つだけ付けられます。
入れられるのはキー・ショートカット・文字列だけです（ボード切り替えの操作も同じ制限）。

**起動を先に、続けての操作を後に** しています。逆にすると、アプリが前に出る前にキーが飛び、
いま前面にある別のアプリがそのキーを受け取ってしまうからです。

---

## この章でできるようになったこと

- 「アプリ起動 API」と「任意コマンド実行 API」の違いを、7つの条件で説明できる
- シェルを通さないこと、スクリプトを拒否することの理由を言える
- 境界に触れる機能を、AI が選ばず利用者に裁定を求めた理由を説明できる

## 扱わなかったこと

- 起動したアプリを前面に出す・終了させる機能（無い）
- 端末からアプリを登録する機能（作らない。登録は PC で `apps/apps.json` を書く）

## 確認問題

- **問 12-1**　`Command::new(exe).args(args)` の代わりに `cmd /C` を通すと、どんな危険がありますか。
- **問 12-2**　`.bat` を登録できないようにしているのはなぜですか。
- **問 12-3**　実行ファイルの実在を、起動時だけでなく押したときにも確かめるのはなぜですか。

（答えは付録 E）
