//! WSルーティング・メッセージ処理・エラー整形（D6/D9/D10/D11）。

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tower_http::services::ServeFile;

use crate::error::*;
use crate::protocol::{ClientMessage, LayerStateWire, ServerMessage, SurfaceConfig};
use crate::state::{canonical_command_id, AdapterJob, ClientId, SharedState};
use proto_keymap::{resolve, Action, Edge, Resolved};

#[derive(Debug, Deserialize)]
pub struct TokenQuery {
    pub token: Option<String>,
}

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(index_page))
        .route("/api/qr", get(qr_image))
        .route("/ws", get(ws_handler))
        .route("/api/deck/export", get(deck_export))
        .route_service("/kb", ServeFile::new("static/kb.html"))
        .route_service("/deck", ServeFile::new("static/deck.html"))
        .with_state(state)
}

/// D12: 手でURL/tokenを入力しなくて済むよう、QRコード付きのランディングページを出す。
/// token自体はこのページのHTML/JSには一切埋め込まない（QR画像は`/api/qr`が都度生成する）。
async fn index_page(State(state): State<SharedState>) -> Response {
    let targets = [
        ("kb-left", "分割キーボード（左手）"),
        ("kb-right", "分割キーボード（右手）"),
        ("deck", "Stream Deck"),
    ];
    let mut cards = String::new();
    for (target, label) in targets {
        let url = {
            let s = state.lock().unwrap();
            s.connection_url(target).unwrap_or_default()
        };
        cards.push_str(&format!(
            r#"<section class="card">
  <h2>{label}</h2>
  <img src="/api/qr?target={target}" alt="QR: {label}" width="220" height="220">
  <p class="url">{url}</p>
</section>
"#
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>KeyDeck — 接続</title>
<style>
  :root {{ color-scheme: dark; }}
  body {{ margin: 0; font-family: system-ui, sans-serif; background: #101526; color: #f4f7ff; padding: 20px; }}
  h1 {{ font-size: 18px; }}
  p.hint {{ color: #b7bfd6; font-size: 13px; margin-top: -4px; }}
  /* QRコード同士が近いと誤読の原因になるため、縦積み＋大きな余白で1枚ずつ隔離する */
  .cards {{ display: flex; flex-direction: column; gap: 56px; max-width: 320px; }}
  .card {{ background: #1c2338; border: 1px solid #2d3550; border-radius: 14px; padding: 20px; text-align: center; }}
  .card h2 {{ font-size: 14px; margin: 0 0 12px; }}
  .card img {{ background: #fff; border-radius: 8px; padding: 20px; display: block; margin: 0 auto; }}
  .url {{ font-size: 11px; color: #b7bfd6; word-break: break-all; max-width: 260px; margin: 12px auto 0; }}
</style>
</head>
<body>
<h1>KeyDeck — スマホのカメラでQRを読み取って開いてください</h1>
<p class="hint">QRは縦に並んでいます。1枚だけが画面に収まるようにカメラを近づけてください。</p>
<div class="cards">
{cards}
</div>
</body>
</html>
"#
    );
    Html(html).into_response()
}

#[derive(Debug, Deserialize)]
struct QrQuery {
    target: String,
}

async fn qr_image(State(state): State<SharedState>, Query(query): Query<QrQuery>) -> Response {
    let url = {
        let s = state.lock().unwrap();
        s.connection_url(&query.target)
    };
    let Some(url) = url else {
        return (StatusCode::BAD_REQUEST, "unknown target").into_response();
    };
    match crate::qr::svg_for_url(&url) {
        Ok(svg) => ([(header::CONTENT_TYPE, "image/svg+xml")], svg).into_response(),
        Err(error) => {
            tracing::error!(code = INTERNAL, cause = %error, "failed to generate QR code");
            (StatusCode::INTERNAL_SERVER_ERROR, "qr generation failed").into_response()
        }
    }
}

fn token_ok(state: &SharedState, query: &TokenQuery) -> bool {
    let s = state.lock().unwrap();
    query.token.as_deref().is_some_and(|candidate| s.token.is_valid(candidate))
}

async fn ws_handler(
    State(state): State<SharedState>,
    Query(query): Query<TokenQuery>,
    upgrade: WebSocketUpgrade,
) -> Response {
    // [T3-2] token検証。切断ではなく、まだ確立していないアップグレード自体を401で拒否する。
    if !token_ok(&state, &query) {
        tracing::error!(
            chk = "T3-2",
            code = WS_TOKEN_INVALID,
            cause = "missing or invalid token on websocket upgrade",
            "rejecting websocket upgrade"
        );
        return (StatusCode::UNAUTHORIZED, WS_TOKEN_INVALID).into_response();
    }
    tracing::info!(chk = "T3-2", "websocket upgrade authorized");
    upgrade.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn deck_export(State(state): State<SharedState>, Query(query): Query<TokenQuery>) -> Response {
    if !token_ok(&state, &query) {
        tracing::error!(code = WS_TOKEN_INVALID, "rejecting deck export request");
        return (StatusCode::UNAUTHORIZED, WS_TOKEN_INVALID).into_response();
    }
    let json_text = {
        let s = state.lock().unwrap();
        serde_json::to_string_pretty(&s.deck).unwrap_or_else(|_| "{}".to_string())
    };
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"deck_export.json\"",
            ),
        ],
        json_text,
    )
        .into_response()
}

async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    let client_id = {
        let mut s = state.lock().unwrap();
        let id = s.next_client_id;
        s.next_client_id += 1;
        s.clients.insert(id, tx.clone());
        id
    };
    tracing::info!(chk = "T3-3", client_id, "client connected");

    let forward_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    send_surface_config_to(&state, client_id);

    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(text) => handle_client_text(&state, client_id, text.as_str()).await,
            Message::Close(_) => break,
            _ => {}
        }
    }

    {
        let mut s = state.lock().unwrap();
        s.clients.remove(&client_id);
    }
    forward_task.abort();
    tracing::info!(chk = "T3-3", client_id, "client disconnected");
}

// ── メッセージ処理 [T3-3] ────────────────────────────────────

async fn handle_client_text(state: &SharedState, client_id: ClientId, text: &str) {
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
        ClientMessage::KeyPress { key_id, edge } => {
            handle_key_press(state, client_id, &key_id, edge.into()).await
        }
        ClientMessage::DeckPress { slot_id } => handle_deck_press(state, client_id, &slot_id).await,
    }
}

async fn handle_key_press(state: &SharedState, client_id: ClientId, key_id: &str, edge: Edge) {
    let resolved = {
        let mut s = state.lock().unwrap();
        let keymap = s
            .keymaps
            .get(&s.active_keymap_id)
            .expect("active_keymap_id always refers to a loaded keymap")
            .clone();
        resolve(&keymap, &mut s.layer_state, key_id, edge)
    };

    match resolved {
        Resolved::UnknownKey => emit_error(
            state,
            client_id,
            "T3-3",
            KEY_UNKNOWN_ID,
            format!("unknown keyId '{key_id}'"),
            json!({ "keyId": key_id }),
        ),
        Resolved::NoResolution => emit_error(
            state,
            client_id,
            "T3-3",
            KEY_RESOLVE_NONE,
            format!("no action resolves for keyId '{key_id}' in the current layer stack"),
            json!({ "keyId": key_id }),
        ),
        Resolved::Ignored => {}
        Resolved::LayerChanged => {
            let wire = {
                let s = state.lock().unwrap();
                LayerStateWire::from(&s.layer_state)
            };
            tracing::info!(chk = "T3-3", ?wire, "layer state changed; broadcasting");
            broadcast(state, &ServerMessage::LayerState(wire));
        }
        Resolved::Fire(action) => fire_action(state, client_id, action).await,
    }
}

async fn handle_deck_press(state: &SharedState, client_id: ClientId, slot_id: &str) {
    let action = {
        let s = state.lock().unwrap();
        s.deck.find_slot(slot_id).map(|slot| slot.action.clone())
    };

    match action {
        None => emit_error(
            state,
            client_id,
            "T3-3",
            DECK_UNKNOWN_SLOT,
            format!("unknown slotId '{slot_id}'"),
            json!({ "slotId": slot_id }),
        ),
        Some(Action::None) => {}
        Some(action) => fire_action(state, client_id, action).await,
    }
}

async fn fire_action(state: &SharedState, client_id: ClientId, action: Action) {
    match &action {
        Action::KeymapSwitch { id } => switch_keymap(state, client_id, id.clone()).await,
        Action::KeymapReset => switch_keymap(state, client_id, "default".to_string()).await,
        Action::Key { .. } | Action::Chord { .. } => {
            let Some(command_id) = canonical_command_id(&action) else {
                emit_error(
                    state,
                    client_id,
                    "T3-3",
                    INTERNAL,
                    "action has no canonical command id".to_string(),
                    json!({}),
                );
                return;
            };

            // D5: 許可リストに無いアクションはOSに届かせない。resolve()はロード済みキーマップ
            // からしかActionを取り出せないため通常は必ず許可されるが、防御として再検証する。
            let allowed = {
                let s = state.lock().unwrap();
                s.command_registry.is_allowed(&command_id)
            };
            if !allowed {
                emit_error(
                    state,
                    client_id,
                    "T3-3",
                    INTERNAL,
                    format!("action resolved but is absent from the startup allow-list: {command_id}"),
                    json!({ "commandId": command_id }),
                );
                return;
            }

            let adapter_tx = {
                let s = state.lock().unwrap();
                s.adapter_tx.clone()
            };
            let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
            if adapter_tx.send(AdapterJob { action, reply: reply_tx }).is_err() {
                emit_error(
                    state,
                    client_id,
                    "T3-3",
                    INTERNAL,
                    "adapter worker channel is closed".to_string(),
                    json!({ "commandId": command_id }),
                );
                return;
            }

            match reply_rx.await {
                Ok(Ok(())) => {}
                Ok(Err(adapter_error)) => emit_error(
                    state,
                    client_id,
                    "T3-3",
                    ADAPTER_SENDINPUT_FAIL,
                    adapter_error.to_string(),
                    json!({ "commandId": command_id }),
                ),
                Err(_) => emit_error(
                    state,
                    client_id,
                    "T3-3",
                    INTERNAL,
                    "adapter worker did not reply".to_string(),
                    json!({ "commandId": command_id }),
                ),
            }
        }
        other => unreachable!(
            "resolve() only Fires Key/Chord/KeymapSwitch/KeymapReset; got {other:?}"
        ),
    }
}

// ── D10: keymap切替 ─────────────────────────────────────────

async fn switch_keymap(state: &SharedState, client_id: ClientId, target_id: String) {
    let exists = {
        let s = state.lock().unwrap();
        s.keymaps.contains_key(&target_id)
    };
    if !exists {
        emit_error(
            state,
            client_id,
            "T3-4",
            KEYMAP_SWITCH_UNKNOWN,
            format!("unknown keymapId '{target_id}'"),
            json!({ "keymapId": target_id }),
        );
        return;
    }

    {
        let mut s = state.lock().unwrap();
        s.active_keymap_id = target_id;
        s.layer_state.reset();
    }
    tracing::info!(chk = "T3-4", "keymap switched; broadcasting surface.config");
    broadcast_surface_config(state);
}

// ── 送信ヘルパー ─────────────────────────────────────────────

fn surface_config_json(state: &SharedState) -> Option<String> {
    let s = state.lock().unwrap();
    let keymap = s.keymaps.get(&s.active_keymap_id)?;
    let message = ServerMessage::SurfaceConfig(SurfaceConfig {
        active_keymap_id: &s.active_keymap_id,
        keymap,
        layer: LayerStateWire::from(&s.layer_state),
        deck: &s.deck,
    });
    match serde_json::to_string(&message) {
        Ok(text) => Some(text),
        Err(error) => {
            tracing::error!(code = INTERNAL, cause = %error, "failed to serialize surface.config");
            None
        }
    }
}

fn send_surface_config_to(state: &SharedState, client_id: ClientId) {
    let Some(text) = surface_config_json(state) else {
        return;
    };
    let s = state.lock().unwrap();
    if let Some(tx) = s.clients.get(&client_id) {
        let _ = tx.send(Message::Text(text.into()));
    }
}

fn broadcast_surface_config(state: &SharedState) {
    let Some(text) = surface_config_json(state) else {
        return;
    };
    let s = state.lock().unwrap();
    for tx in s.clients.values() {
        let _ = tx.send(Message::Text(text.clone().into()));
    }
}

fn broadcast(state: &SharedState, message: &ServerMessage<'_>) {
    let text = match serde_json::to_string(message) {
        Ok(text) => text,
        Err(error) => {
            tracing::error!(code = INTERNAL, cause = %error, "failed to serialize broadcast message");
            return;
        }
    };
    let s = state.lock().unwrap();
    for tx in s.clients.values() {
        let _ = tx.send(Message::Text(text.clone().into()));
    }
}

/// [T3-5] D9のエラー整形の要。Hub側は1行ログ、クライアント側にはerrorフレームを送る。
fn emit_error(
    state: &SharedState,
    client_id: ClientId,
    chk: &'static str,
    code: &'static str,
    cause: impl Into<String>,
    context: serde_json::Value,
) {
    let cause = cause.into();
    tracing::error!(chk, code, cause = %cause, context = %context, "protocol error");
    let text = match serde_json::to_string(&ServerMessage::Error {
        code,
        cause,
        context,
    }) {
        Ok(text) => text,
        Err(error) => {
            tracing::error!(code = INTERNAL, cause = %error, "failed to serialize error frame itself");
            return;
        }
    };
    let s = state.lock().unwrap();
    if let Some(tx) = s.clients.get(&client_id) {
        let _ = tx.send(Message::Text(text.into()));
    }
}
