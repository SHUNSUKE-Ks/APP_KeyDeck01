# 第7章　Windows へ渡す

## この章の問い

「この操作を実行する」と決まったあと、Windows に届くまでに何を確かめているのか。

## 先に結論

Windows へ渡す直前に、Hub は **もう一度、許可リストと照らします**（第2章の門3）。
通ったものだけが、**1本の順番待ちの列**（adapter のワーカー）に並び、`proto-adapter-win` が SendInput で Windows へ送ります。
文字列の入力はクリップボードを使わず、**1文字ずつ Unicode として直接** 送ります。

---

## 画面で起きること

キーボードの `A` を押すと、PC の前面アプリに `a` が入ります。
Hub のコンソールには、何に解決して何を送ったかが1行残ります。行の形は次のとおりです。

```text
key press chk="T3-3" surface=Ipad keymap_id="-" key_id="K302" edge=Down layers=0 outcome=key:A
```

（この行は **形を示すための例** で、撮影時の実際の出力ではありません。項目の並びは `handle_key_press` のログ出力のとおり。
本書の撮影では、前面のアプリへ意図しない文字が入るのを避けるため、文字キーは押していません）

最後の `outcome=key:A` が、第2章の照合札です。
**画面のほうは何も変わりません。** レイヤーは変わっていないからです（第6章）。

---

## コード1 — 発火の直前にもう一度照らす

```rust
async fn fire_action(state: &SharedState, client_id: ClientId, action: Action) {
    match &action {
        // …（ボード切り替え・アプリ起動・キーマップ切り替えは別の枝。省略）…
        Action::Key { .. }
        | Action::Chord { .. }
        | Action::Text { .. }
        | Action::KeyButton { .. }
        | Action::MouseClick { .. }
        | Action::MouseDoubleClick { .. } => {
            let Some(command_id) = canonical_command_id(&action) else {
                emit_error(state, client_id, "T3-3", INTERNAL,
                    "action has no canonical command id".to_string(), json!({}));
                return;
            };

            // D5: 許可リストに無いアクションはOSに届かせない。resolve()はロード済みキーマップ
            // からしかActionを取り出せないため通常は必ず許可されるが、防御として再検証する。
            let allowed = {
                let s = state.lock().unwrap();
                s.command_registry.is_allowed(&command_id)
            };
            if !allowed {
                emit_error(state, client_id, "T3-3", INTERNAL,
                    format!("action resolved but is absent from the startup allow-list: {command_id}"),
                    json!({ "commandId": command_id }));
                return;
            }

            // …（押しっぱなしの台帳を更新する。第8章）…

            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if adapter_tx.send(AdapterJob { action, reply: reply_tx }).is_err() {
                // …（省略）…
            }
            // …（結果を待ち、失敗ならエラーを返す。省略）…
        }
        // …（省略）…
    }
}
```

出典: `crates/proto-hub/src/ws.rs` の `fire_action`。別の枝と、台帳・結果待ちの処理を省き、`emit_error` の引数を詰めて書いた。**説明用に簡略化**。

| 行 | 読み方 |
|---|---|
| `canonical_command_id(&action)` | 実行しようとしている操作の照合札を作る |
| `command_registry.is_allowed(…)` | 起動時に作った許可リストに、その札があるか |
| `if !allowed { … return; }` | 無ければ Windows へは何も送らず、エラーを返して終わる |
| `adapter_tx.send(AdapterJob { … })` | 通ったものを adapter の列に並べる。**入力を送るのは1本の列が順番に行う** |
| `reply_tx` / `reply_rx` | 列の先で送り終わったら、成否が返ってくる |

コメントにあるとおり、`resolve` は読み込み済みのキーマップからしか操作を取り出せないので、ここで弾かれることは通常ありません。
**それでも照らすのは、「通常ありえない」を前提にしないため** です。
実際、入れ子の操作（`tg.fire`）を足したときには、ここで弾かれました（第13章）。

### 関数カード　`fire_action`

| 項目 | 内容 |
|---|---|
| 役割 | 解決済みの操作を、種類ごとに実行する。Windows に届く操作は許可リストで再確認してから adapter へ渡す |
| 呼ばれる場面 | キー（第6章）、Deck のボタン、ジェスチャーが「実行する」に解決されたとき |
| 入力 | 共有状態、接続番号、操作 |
| 処理順 | 種類で分岐 → 照合札 → 許可リスト → 押しっぱなし台帳 → adapter の列へ → 結果を待つ |
| 出力 | 戻り値は無い。失敗は `error` として送り主へ |
| 失敗時 | 札が無い・許可リストに無い・adapter が失敗、のどれもエラーを返すだけで Hub は止まらない |
| 関連 | `canonical_command_id`（第2章）、`proto_adapter_win::send`、`release_held_keys`（第8章） |
| たとえ | 倉庫の出荷口。伝票を名簿と照らし、合っていれば配送の列に並べる |

---

## なぜ1本の列にするのか

2人が同時に別のキーを押すと、Windows へのキー入力が混ざる恐れがあります。
たとえば Ctrl+C（Ctrl を押す → C を押す → C を離す → Ctrl を離す）の途中に、別の人の `V` が割り込むと、
Windows から見れば Ctrl+V になってしまいます。
**入力を送る係を1人にし、依頼を順番に処理する** ことで、1つの操作の途中に他の操作が割り込まないようにしています。

---

## コード2 — adapter: 種類ごとに送り方を変える

```rust
pub fn send(action: &Action) -> Result<(), AdapterError> {
    match action {
        Action::Key { vk } => send_key(vk),
        Action::Chord { keys } => send_chord(keys),
        Action::Text { string } => send_text(string),
        // P-005 段階C: 押しっぱなし。pressとreleaseを別々に打つ（send_key()はpress+releaseで一体）。
        Action::KeyButton { vk, down } => {
            let code = resolve_code(vk)?;
            if *down { press(code, vk) } else { release(code, vk) }
        }
        Action::MouseMove { dx, dy } => send_mouse_move(*dx, *dy),
        // …（マウスのクリック・ボタン・スクロール。省略）…
        other => Err(AdapterError::Unsupported {
            cause: format!("action cannot be sent to the OS: {other:?}"),
        }),
    }
}
```

出典: `crates/proto-adapter-win/src/lib.rs` の `send`。マウスの4行を省いた。**説明用に簡略化**。

最後の `other => Err(…)` が大事です。レイヤーの切り替えのような **Windows に届くはずのない操作が万一ここへ来ても、送らずにエラーにします。**

---

## コード3 — ショートカットは「修飾キーを必ず離す」

```rust
fn send_chord(keys: &[String]) -> Result<(), AdapterError> {
    // …（最後の1つを本体、それ以外を修飾キーに分ける。省略）…

    // 修飾↓ …本体↓ → 本体↑ … 修飾↑ の順（例 CTRL+S: CTRL↓ S↓ S↑ CTRL↑）。
    let mut pressed: Vec<(u16, &str)> = Vec::with_capacity(modifiers.len());
    for modifier in modifiers {
        // …（押す。途中で失敗したら、それまでに押した修飾キーを離してから返す。省略）…
        pressed.push((code, modifier.as_str()));
    }

    let main_result = resolve_code(main).and_then(|code| {
        press(code, main)?;
        release(code, main)
    });

    // 本体キーの成否に関わらず、押しっぱなしの修飾キーを残さないよう逆順で必ず離す。
    release_all_best_effort(&pressed);

    main_result
}
```

出典: `crates/proto-adapter-win/src/lib.rs` の `send_chord`。**説明用に簡略化**。

本体キーの送信に失敗しても、**修飾キーは必ず離します。** Ctrl が押されたまま残ると、
そのあと利用者が PC のキーボードで打つ文字がすべて Ctrl 付きになり、PC が壊れたように見えるからです。

---

## コード4 — 文字列はクリップボードを使わない

```rust
/// [T8-1] D20: KEYEVENTF_UNICODEで文字列を直接注入する。vk辞書は経由しない。
/// サロゲートペア対応のためUTF-16コード単位ごとにdown→upを1組ずつ送出する。
fn send_text(string: &str) -> Result<(), AdapterError> {
    if string.is_empty() {
        return Err(AdapterError::Unsupported {
            cause: "text action string must not be empty".into(),
        });
    }
    for code_unit in string.encode_utf16() {
        press_unicode(code_unit)?;
        release_unicode(code_unit)?;
    }
    Ok(())
}
```

出典: `crates/proto-adapter-win/src/lib.rs` の `send_text`。省略なし。

| 行 | 読み方 |
|---|---|
| `if string.is_empty()` | 空の文字列は送らない（何も起きないのに「送った」ことになるのを防ぐ） |
| `string.encode_utf16()` | 文字列を Windows が扱う単位（UTF-16）に分ける。絵文字などは2単位になる |
| `press_unicode` / `release_unicode` | 1単位ずつ「押して離す」。キーの位置ではなく **文字そのもの** として送る |

### 関数カード　`send_text`

| 項目 | 内容 |
|---|---|
| 役割 | 決まった文字列を、Windows の前面アプリへ文字として入力する |
| 呼ばれる場面 | JSON に書かれた `text` 操作が許可リストを通り、adapter の列で順番が来たとき |
| 入力 | 文字列（JSON に書かれていたもの。端末からは来ない） |
| 処理順 | 空なら拒否 → UTF-16 に分ける → 1単位ずつ `KEYEVENTF_UNICODE` で押して離す |
| 出力 | 成功、または `AdapterError` |
| 失敗時 | 途中の1単位で失敗したら、そこでエラーを返す（それまでの文字は入力済み） |
| 関連 | `send`、`press_unicode`、`release_unicode` |
| たとえ | 共有の机（クリップボード）に置いて持っていってもらうのではなく、1文字ずつ手渡しする |

### なぜクリップボードを使わないのか

コピーして貼り付ける方法なら速いですが、**利用者がコピーしていた内容を黙って上書き** してしまいます（不変条件5）。
また、キーの位置として送る方法は、日本語入力が有効かどうかで結果が変わります。
Unicode 直接入力なら、IME の状態に関係なく同じ文字が入ります。

**未確認:** Hub が送り終えたことと、**利用者が意図した入力欄に文字が入ったこと** は別です。
SendInput は「いま前面にあるウィンドウ」へ届くので、どの欄に入ったかは Hub からは確かめられません。

---

## この章でできるようになったこと

- 発火の直前に許可リストで照らし直す理由を説明できる
- 入力を1本の列で送る理由を、Ctrl+C の例で説明できる
- `key`、`chord`、`text` の送り方の違いを言える

## 扱わなかったこと

- macOS・Linux 向けの adapter（無い）
- 入力先のウィンドウを選ぶ機能（無い。常に前面のウィンドウ）

## 確認問題

- **問 7-1**　`resolve` はキーマップにある操作しか返さないのに、`fire_action` がもう一度許可リストで照らすのはなぜですか。
- **問 7-2**　`send_chord` が本体キーの失敗時にも修飾キーを離すのはなぜですか。
- **問 7-3**　`text` 操作が「送信に成功した」ことは、「入力欄に文字が入った」ことを意味しますか。

（答えは付録 E）
