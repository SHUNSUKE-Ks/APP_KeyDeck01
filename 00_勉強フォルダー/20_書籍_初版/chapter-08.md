# 第8章　押しっぱなしと、切断したときの解放

## この章の問い

「押している間ずっと効く」キーを作ったら、指を離す前に通信が切れたときどうなるのか。

## 先に結論

押しっぱなしのキー（`key.hold`）は、**押したとき・離したときの両方** で Windows へ入力を送る唯一の操作です。
離す通知が届かないと、PC のキーは押されたまま残ります。
そこで Hub は **接続ごとに「いま押しているキー」の台帳** を持ち、切断したらその台帳のキーを全部離します。

---

## 画面で起きること

ゲーム用の十字キーで `↑` を押し続けると、キャラクターは押している間ずっと前へ進みます。
ここで iPad の画面を閉じたり、Wi-Fi が切れたりすると、端末は「離した」を送れません。

もし何もしなければ、PC から見ると **↑ が押されっぱなし** です。
キャラクターは走り続け、PC のキーボードで何を打っても ↑ が混ざります。

---

## 押しっぱなしが最初は作れなかった

KeyDeck の最初の adapter には、キーを送る関数が `send_key` しかありませんでした。

```rust
fn send_key(vk: &str) -> Result<(), AdapterError> {
    let code = resolve_code(vk)?;
    press(code, vk)?;
    release(code, vk)?;
    Ok(())
}
```

出典: `crates/proto-adapter-win/src/lib.rs` の `send_key`。省略なし。

**押した直後に離す** が1つの関数になっているので、「押したまま」を表せません。
十字キーを作ろうとして初めて、それが分かりました。

そこで「押す」と「離す」を別々に送る操作を足したのですが、
**押しっぱなしを足した瞬間に、離し忘れという新しい危険が生まれました。** この章はその対策の話です。

---

## コード1 — `key.hold` は離したときも「実行する」

```rust
// P-005 段階C: 押しっぱなし。**upでもFireを返す唯一のアクション**
// （他の葉アクションはupがIgnored）。ここでdownフラグを確定させるので、
// Hub側は受け取ったKeyButtonをそのままadapterへ流すだけでよい。
Action::KeyHold { vk } => Resolved::Fire(Action::KeyButton {
    vk: vk.clone(),
    down: matches!(edge, Edge::Down),
}),
```

出典: `crates/proto-keymap/src/lib.rs` の `resolve` の中の `KeyHold` の枝。省略なし。

| 行 | 読み方 |
|---|---|
| `Action::KeyHold { vk }` | JSON には `{"t":"key.hold","vk":"UP"}` と書く |
| `Resolved::Fire(Action::KeyButton { … })` | 押したときも離したときも「実行する」を返す |
| `down: matches!(edge, Edge::Down)` | 押したなら `down: true`、離したなら `down: false` |

JSON に書くのは `KeyHold`、実際に adapter へ流れるのは `KeyButton` です。
第2章の `canonical_command_id` で **この2つが同じ照合札 `key.hold:UP` になるようにしてある** のは、
片方だけだと実行時に許可リストで弾かれるからです。

---

## コード2 — 送る前に台帳へ書く

```rust
// P-005 段階C: 押しっぱなしの台帳を更新してからadapterへ送る。
// 先に記録するのは、発火が失敗しても「押したかもしれない」側に倒して
// 切断時に必ずreleaseを打つため（取りこぼしより二重releaseの方が安全）。
if let Action::KeyButton { vk, down } = &action {
    let mut s = state.lock().unwrap();
    s.note_key_hold(client_id, vk, *down);
}
```

出典: `crates/proto-hub/src/ws.rs` の `fire_action` の中。省略なし。

**Windows へ送るより先に、台帳に書きます。**
送信が途中で失敗した場合、「押せたかどうか分からない」状態になります。
そのとき台帳に「押した」と残っていれば、切断時に離す処理が走ります。
押していないキーを離しても害はありませんが、押したキーを離し忘れると PC が操作できなくなります。
**間違えるなら、安全な側に間違える。** これがコメントの「二重releaseの方が安全」の意味です。

---

## コード3 — 切断したら台帳のキーを全部離す

```rust
async fn release_held_keys(state: &SharedState, client_id: ClientId) {
    let (held, adapter_tx) = {
        let mut s = state.lock().unwrap();
        (s.take_held_keys(client_id), s.adapter_tx.clone())
    };
    if held.is_empty() {
        return;
    }
    tracing::info!(
        chk = "P005-C",
        client_id,
        keys = ?held,
        "releasing keys still held by a disconnecting client"
    );
    for vk in held {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        let action = Action::KeyButton { vk: vk.clone(), down: false };
        if adapter_tx.send(AdapterJob { action, reply: reply_tx }).is_err() {
            tracing::error!(code = INTERNAL, vk = %vk, "adapter worker gone; cannot release held key");
            continue;
        }
        match reply_rx.await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                tracing::error!(code = ADAPTER_SENDINPUT_FAIL, vk = %vk, cause = %error, "failed to release held key")
            }
            Err(error) => {
                tracing::error!(code = INTERNAL, vk = %vk, cause = %error, "adapter reply dropped while releasing held key")
            }
        }
    }
}
```

出典: `crates/proto-hub/src/ws.rs` の `release_held_keys`。省略なし。

| 行 | 読み方 |
|---|---|
| `take_held_keys(client_id)` | この接続の台帳を **取り出して空にする**（二度離さないため） |
| `if held.is_empty() { return; }` | 何も押していなければ何もしない |
| `KeyButton { vk, down: false }` | 「離す」だけを作る |
| `adapter_tx.send(…)` | 普通の入力と同じ列に並べる。割り込まない |
| `continue` と `tracing::error!` | 1つ離せなくても、残りのキーは離し続ける |

### 関数カード　`release_held_keys`

| 項目 | 内容 |
|---|---|
| 役割 | 切断した端末が押したままにしていたキーを、すべて離す |
| 呼ばれる場面 | `handle_socket` の受信ループを抜けたとき（画面を閉じた、通信が切れた、端末がスリープした） |
| 入力 | 共有状態、接続番号 |
| 処理順 | 台帳を取り出す → 空なら終わり → ログを1行 → 1キーずつ「離す」を adapter の列へ → 結果を待つ |
| 出力 | 戻り値は無い。台帳は空になる |
| 失敗時 | そのキーをログに残し、次のキーへ進む。**途中で止めない** |
| 関連 | `handle_socket`（第5章）、`fire_action`（第7章）、`KeyHold`（`resolve`） |
| たとえ | 貸出中の鍵の回収。利用者が黙って帰っても、受付が台帳を見て全部回収する |

---

## `Mo`（レイヤーの押しっぱなし）は台帳の対象外

第6章の `Mo`（押している間だけレイヤーを変える）も「押しっぱなし」ですが、この台帳には載りません。
`Mo` は **Hub の中の状態** を変えるだけで、Windows には何も押していないからです。
台帳は「Windows に押したままのキーがあるか」を記録するためのもので、目的が違います。

### 観測したこと — `Mo` は切断しても残る

第6章の撮影では、Fn の「押す」を送った接続を、**「離す」を送る前に閉じています。**
それでも撮影中の画面は `Layer 1` のまま残り、あとで別の接続から「離す」を送ったときのログは次のとおりでした。

```text
key press chk="T3-3" surface=Ipad keymap_id="-" key_id="K101" edge=Up layers=0,1 outcome=layer-change
```

`layers=0,1` は「離す」を処理する直前の有効レイヤーです。つまり **押した接続が切れても、レイヤー1は有効なまま残っていました。**

これは Windows を操作不能にはしません（Windows には何も押していないため）。
ただ、Fn を押したまま iPad の画面を閉じると、次に開いたときもキーの文字が F キーのままになりえます。
その場合は Fn をもう一度押して離せば戻ります。
**これを直すべき不具合とするかは、この本の執筆時点では判断していません**（付録 D）。

---

## この章でできるようになったこと

- `key.hold` だけが「離したとき」にも入力を送る理由を説明できる
- 台帳に先に書くことで、失敗したときにも安全な側に倒れる理由を言える
- 押しっぱなしの機能と切断時の解放を、同時に作らなければならない理由を言える

## 扱わなかったこと

- Hub そのものが異常終了したときの解放（Hub が動いていないので、この仕組みでは離せない）
- OS 全体の非常停止キー

## 確認問題

- **問 8-1**　`KeyHold` が、他のキー操作と違って `Up` でも `Fire` を返すのはなぜですか。
- **問 8-2**　押しっぱなしの台帳を、Windows へ送った「後」ではなく「前」に更新するのはなぜですか。
- **問 8-3**　`Mo` を押したまま画面を閉じたとき、`release_held_keys` はそれを離しますか。

（答えは付録 E）
