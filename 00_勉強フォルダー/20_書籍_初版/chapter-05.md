# 第5章　WebSocket で「位置」だけを送る

## この章の問い

端末で押したボタンは、どうやって Hub に届き、Hub の中のどこへ回されるのか。

## 先に結論

端末は Hub と **WebSocket を1本つなぎっぱなし** にします。
押すと `{"type":"key.press","keyId":"K101","edge":"down"}` のような短い JSON が1つ飛びます。
Hub は **① 接続を張る前に token を確かめ**、**② 届いた JSON を4種類のどれかとして読み**、読めなければその場で「読めなかった」と返します。

---

## 画面で起きること

端末が実際に送る1通です（第6章の撮影で送ったものと同じ）。

```json
{ "type": "key.press", "keyId": "K101", "edge": "down" }
```

指を離すと、`"edge": "up"` の1通がもう一度飛びます。**押す・離すは別々のメッセージ** です。

Hub のコンソールには、押したキーが必ず1行残ります（2026-09-15 の実際の出力）。

```text
key press chk="T3-3" surface=Ipad keymap_id="-" key_id="K101" edge=Down layers=0 outcome=layer-change
```

---

## HTTP と WebSocket の使い分け

| 通信 | 使いどころ | KeyDeck での例 |
|---|---|---|
| HTTP | 1回聞いて1回答えてもらう | 画面の HTML を取る、QR 画像を取る、保存する |
| WebSocket | つなぎっぱなしで、どちらからでも話しかける | 押した・離した、レイヤーが変わった、ボードを切り替えろ |

押す・離すは1秒に何回も起き、しかも **Hub の側から「レイヤーが変わった」と画面へ知らせる** 必要があります。
だから押下は WebSocket を通します。

---

## コード1 — 接続を張る前に token を確かめる

```rust
async fn ws_handler(
    State(state): State<SharedState>,
    Query(query): Query<WsQuery>,
    upgrade: WebSocketUpgrade,
) -> Response {
    // [T3-2] token検証。切断ではなく、まだ確立していないアップグレード自体を401で拒否する。
    if !token_ok(&state, query.token.as_deref()) {
        tracing::error!(
            chk = "T3-2",
            code = WS_TOKEN_INVALID,
            cause = "missing or invalid token on websocket upgrade",
            "rejecting websocket upgrade"
        );
        return (StatusCode::UNAUTHORIZED, WS_TOKEN_INVALID).into_response();
    }
    let surface = SurfaceKind::from_query(query.surface.as_deref());
    tracing::info!(chk = "T3-2", ?surface, "websocket upgrade authorized");
    upgrade.on_upgrade(move |socket| handle_socket(socket, state, surface))
}
```

出典: `crates/proto-hub/src/ws.rs` の `ws_handler`。省略なし。

| 行 | 読み方 |
|---|---|
| `Query(query)` | URL の `?token=…&surface=ipad` を読む |
| `if !token_ok(…)` | token が無い・違うなら、**つながる前に** 401 を返して終わる。一度つないでから切るのではない |
| `tracing::error!(… code …, cause …)` | エラーは必ずコードと原因つきで1行（不変条件3） |
| `SurfaceKind::from_query` | どの種類の画面からの接続か（iPad 一枚・分割キーボード・ボード面・トラックボール） |
| `upgrade.on_upgrade(…)` | ここで初めて WebSocket になり、以降は `handle_socket` が受け持つ |

---

## コード2 — つないでいる間の1接続

```rust
async fn handle_socket(socket: WebSocket, state: SharedState, surface: SurfaceKind) {
    // …（接続に番号を振り、送信用の通路を登録する。省略）…

    send_surface_config_to(&state, client_id, surface);

    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(text) => handle_client_text(&state, client_id, surface, text.as_str()).await,
            Message::Close(_) => break,
            _ => {}
        }
    }

    // P-005 段階C: 切断時に押しっぱなしのキーを必ず離す。
    // これを飛ばすと、十字キーを押したまま画面を閉じた瞬間にPCが操作不能になる。
    release_held_keys(&state, client_id).await;

    // …（登録を外す。省略）…
}
```

出典: `crates/proto-hub/src/ws.rs` の `handle_socket`。接続番号の採番、送信タスク、後片付けを省いた。**説明用に簡略化**。

| 行 | 読み方 |
|---|---|
| `send_surface_config_to` | つながった直後に「いま何を表示すべきか」を端末へ送る。端末は自分では何も決めない |
| `while let Some(Ok(message))` | 端末が話しかけてくる限り、1通ずつ待つ |
| `Message::Text(text) => handle_client_text(…)` | 文字のメッセージは、受付（次のコード）へ回す |
| `release_held_keys` | ループを抜けた＝切断した。押しっぱなしのキーをここで必ず離す（第8章） |

---

## コード3 — 受付: 4種類のどれかとして読む

```rust
async fn handle_client_text(state: &SharedState, client_id: ClientId, surface: SurfaceKind, text: &str) {
    let parsed: Result<ClientMessage, _> = serde_json::from_str(text);
    let message = match parsed {
        Ok(message) => message,
        Err(error) => {
            emit_error(
                state,
                client_id,
                "T3-3",
                WS_PARSE,
                format!("failed to parse client message: {error}"),
                json!({ "raw": text }),
            );
            return;
        }
    };

    match message {
        ClientMessage::KeyPress { keymap_id, key_id, edge } => {
            handle_key_press(state, client_id, surface, keymap_id.as_deref(), &key_id, edge.into()).await
        }
        ClientMessage::DeckPress { deck_id, slot_id } => {
            let deck_id = deck_id.as_deref().unwrap_or(DEFAULT_DECK_ID);
            handle_deck_press(state, client_id, deck_id, &slot_id).await
        }
        ClientMessage::SurfaceState { surface_id, delta, .. } => {
            handle_surface_state(state, client_id, &surface_id, delta.dx, delta.dy).await
        }
        ClientMessage::SurfaceGesture { surface_id, gesture_id, edge } => {
            handle_surface_gesture(state, client_id, &surface_id, &gesture_id, edge.map(Edge::from)).await
        }
    }
}
```

出典: `crates/proto-hub/src/ws.rs` の `handle_client_text`。途中のコメント2行を省いた。**説明用に簡略化**。

| 行 | 読み方 |
|---|---|
| `serde_json::from_str(text)` | 文字列を、第2章の `ClientMessage` の4種類のどれかとして読む |
| `Err(error) => emit_error(… WS_PARSE …)` | 4種類のどれでもない（知らない `type`、足りない項目）なら、送り主に「読めなかった」と返して終わる。**Hub は落ちない** |
| `match message { … }` | 種類ごとに窓口が分かれる。キーは `handle_key_press`（第6章）へ |

### 関数カード　`handle_client_text`

| 項目 | 内容 |
|---|---|
| 役割 | 端末から届いた文字列を型に読み、種類ごとの窓口へ回す |
| 呼ばれる場面 | つながっている端末からメッセージが1通届くたび |
| 入力 | 接続番号、画面の種類、受け取った文字列 |
| 処理順 | JSON として読む → 4種類のどれかに当てはめる → 窓口の関数を呼ぶ |
| 出力 | 戻り値は無い。結果は窓口の先で、画面への配信や Windows への入力として現れる |
| 失敗時 | `WS_PARSE` を送り主へ返す。接続は切らず、Hub も止めない |
| 関連 | `handle_key_press`（第6章）、`handle_surface_state`（第10章）、`emit_error` |
| たとえ | 郵便の仕分け。封筒の種類を見て窓口に回すだけで、中身の手続きはしない |

**この関数は Windows へ何も送りません。** 仕分けと実行を分けてあるので、
「知らない種類のメッセージが Windows 入力になる」経路がそもそもありません。

---

## Hub から端末へ届くもの

Hub から端末へ送るメッセージも、種類が決まっています。

| メッセージ | 意味 |
|---|---|
| `surface.config` | いま表示すべき盤面（つながった直後など） |
| `layer.state` | レイヤーが変わった（第6章） |
| `layout.switch` | 別のボードへ移れ（ボード切り替えボタン） |
| `error` | 何かが通らなかった。コードと原因つき |

`crates/proto-hub/src/protocol.rs` の冒頭のコメントには「Hub→client は3種のみ」と書かれていますが、
後から `layout.switch` が足されて今は4種類です。**コメントは実装を追いかけ損ねることがある** という実例として残しておきます。

---

## この章でできるようになったこと

- HTTP と WebSocket を、KeyDeck ではどう使い分けているか言える
- token の確認が「つないだ後に切る」ではなく「つなぐ前に断る」であることを説明できる
- 知らないメッセージが届いても Hub が止まらない理由を言える

## 扱わなかったこと

- 端末側の再接続の仕組み（`/api/ping` で「切れただけか、token が古いのか」を見分ける）
- 独自のメッセージ種別を足す方法（足すなら第2章の境界を先に検討する）

## 確認問題

- **問 5-1**　token が間違っているとき、Hub は WebSocket を「つないでから切る」のと「つなぐ前に断る」のどちらをしますか。
- **問 5-2**　`{"type":"run","command":"calc"}` が届いたら、Hub はどうなりますか。
- **問 5-3**　`handle_client_text` が Windows へ直接入力を送らないことで、何が守られていますか。

（答えは付録 E）
