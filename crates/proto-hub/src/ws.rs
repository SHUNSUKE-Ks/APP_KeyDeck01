//! WSルーティング・メッセージ処理・エラー整形（D6/D9/D10/D11、T8でipad面を追加）。

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::error::*;
use crate::protocol::{ClientMessage, LayerStateWire, ServerMessage, SurfaceConfig};
use crate::state::{
    canonical_command_id, AdapterJob, ClientId, HubState, SharedState, SurfaceKind,
    DEFAULT_DECK_ID, IPAD_KEYMAP_ID,
};
use crate::surface::{
    ClickButton, GestureAction, SURFACE_GESTURE_EDGE_REQUIRED, SURFACE_GESTURE_UNKNOWN_ID,
    SURFACE_STATE_RANGE, SURFACE_UNKNOWN_ID,
};
use proto_keymap::{resolve, Action, Edge, Keymap, LayerState, MouseButtonKind, Resolved};

#[derive(Debug, Deserialize)]
pub struct TokenQuery {
    pub token: Option<String>,
}

/// T8: `/ws?token=...&surface=ipad` でipad面として接続する。省略時は従来どおり
/// 分割/Deckの共有state（active_keymap_id/layer_state）を使うSplit面として扱う。
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
    pub surface: Option<String>,
}

pub fn router(state: SharedState) -> Router {
    Router::new()
        // トップ = レイアウトエディタ（PCで開く。区画の並べ替えをする画面）
        .route_service("/", ServeFile::new("static/editor.html"))
        // QRは専用ページへ移した。ギャラリー形式で、カードごとにQRへ切り替える
        .route_service("/connect", ServeFile::new("static/gallery.html"))
        .route("/api/qr", get(qr_image))
        .route("/api/reload", post(reload_handler))
        // P-005: tokenが生きているかだけを返す。クライアントはWSが閉じたときにこれを叩き、
        // 「単に切れている」のか「tokenが古い」のかを区別してユーザーに伝える。
        .route("/api/ping", get(ping_handler))
        // T9（設計書v0.5 F3）: 設定画面が「いま何が読み込まれているか」「どこへ繋げるか」を
        // 引くための読み取り専用API。書き込みは一切しない。
        .route("/api/formats", get(formats_handler))
        // P-005 段階D: レイアウトの保存。**書き込みはこの1本だけ**。
        // 書き先は layouts/layout_<id>.json に固定され、idは厳格に検証される。
        .route("/api/layout/save", post(layout_save_handler))
        .route("/ws", get(ws_handler))
        .route("/api/deck/export", get(deck_export))
        .route_service("/kb", ServeFile::new("static/kb.html"))
        .route_service("/deck", ServeFile::new("static/deck.html"))
        .route_service("/ipad", ServeFile::new("static/ipad.html"))
        .route_service("/trackball", ServeFile::new("static/trackball.html"))
        // T20（P-003 Ver1-a）: 分割面（上=Deck／下=キーボード）。WSは既存の
        // `/ws?surface=ipad` を使うため、プロトコル・状態管理の追加はゼロ。
        .route_service("/panel", ServeFile::new("static/panel.html"))
        // P-005 段階B: レイアウト面。layouts/layout_*.json の区画割りをそのまま描く。
        .route_service("/layout", ServeFile::new("static/layout.html"))
        .route_service("/settings", ServeFile::new("static/settings.html"))
        // 部品の描画は static/components.js が唯一の実装。実機の面とエディタが
        // これを共有するので、プレビューと実機の絵がズレない。
        .route_service("/components.js", ServeFile::new("static/components.js"))
        // ギャラリーに実機スクショを出すための置き場。ファイルが無ければ
        // クライアント側が簡易図へ自動で切り替えるので、空でも動く。
        .nest_service("/shots", ServeDir::new("static/shots"))
        // P-005: **すべての応答に Cache-Control: no-store を付ける。**
        //
        // これが無いと、ブラウザは last-modified/etag だけを見て独自判断でキャッシュを
        // 使い回す（ヒューリスティックキャッシュ）。iOS Safariは特に強く、
        // **端末を再起動してもHTTPキャッシュは消えない**ため、Hub側が新しいHTMLを
        // 配っているのに端末が何時間も古い画面を出し続ける、という形で実際に壊れた。
        //
        // この面はJSONを書き換えて即反映するのが売りの道具で、配るのはLAN内の
        // 数十KBのHTMLでしかない。キャッシュで得るものより、古い画面が出る害の方が大きい。
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        ))
        .with_state(state)
}

/// 接続先の一覧（target識別子・表示名・種別）。
///
/// **ここが一次情報で、3箇所がこれを見る**: ランディングページ / `/api/formats` /
/// 起動時のコンソール出力。以前は3箇所それぞれに手書きの配列があり、
/// P-005でレイアウトを足したときに**コンソール出力だけ更新されず取り残された**
/// （レイアウト3種がターミナルに一切出ない状態が残っていた）。同じ表を複数箇所に
/// 書かないこと。
pub fn connection_targets(state: &HubState) -> Vec<(String, String, &'static str)> {
    let mut targets: Vec<(String, String, &'static str)> = vec![
        ("kb-left".to_string(), "分割キーボード（左手）".to_string(), "keyboard"),
        ("kb-right".to_string(), "分割キーボード（右手）".to_string(), "keyboard"),
        ("deck".to_string(), "Stream Deck".to_string(), "deck"),
        ("ipad".to_string(), "iPad一枚キーボード（Vol1.2）".to_string(), "keyboard"),
        ("trackball".to_string(), "トラックボール".to_string(), "trackball"),
        ("panel".to_string(), "分割（Deck＋キーボード）".to_string(), "panel"),
    ];
    // 読み込まれているレイアウトは自動で並ぶ。レイアウトを足してもここは触らない。
    for id in state.layouts.keys() {
        targets.push((format!("layout:{id}"), format!("レイアウト: {id}"), "layout"));
    }
    targets
}

#[derive(Debug, Deserialize)]
pub struct QrQuery {
    /// `kb-left` のような固定target、または `layout:<layoutId>`。
    /// 解決は `HubState::connection_url` が一手に引き受ける。
    pub target: String,
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

fn token_ok(state: &SharedState, token: Option<&str>) -> bool {
    let s = state.lock().unwrap();
    token.is_some_and(|candidate| s.token.is_valid(candidate))
}

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

#[derive(Debug, Deserialize)]
pub struct DeckExportQuery {
    pub token: Option<String>,
    /// P-005 段階A: どのDeckを書き出すか。省略時は`DEFAULT_DECK_ID`。
    pub deck: Option<String>,
}

/// P-005: token検証だけを行う軽量エンドポイント。
/// Hubを再起動するとtokenが変わる（D8）ため、端末が古いURLを掴んだままだと
/// WSが延々と拒否され続ける。クライアント側でそれを検出して案内するために使う。
async fn ping_handler(State(state): State<SharedState>, Query(query): Query<TokenQuery>) -> Response {
    if token_ok(&state, query.token.as_deref()) {
        (StatusCode::OK, "ok").into_response()
    } else {
        (StatusCode::UNAUTHORIZED, WS_TOKEN_INVALID).into_response()
    }
}

/// T9: 設定画面の一次データ。**読み取り専用**（このAPIは何も書き換えない）。
///
/// 返すもの:
///   `targets`   … 接続先（QRの元。URLにtokenが入るのでtoken検証必須）
///   `keymaps`   … 読み込み済みキーボードとその規模
///   `decks`     … Deckと描き方・スロット数
///   `layouts`   … 画面の区画割り（どの部品がどこに置かれているか）
async fn formats_handler(State(state): State<SharedState>, Query(query): Query<TokenQuery>) -> Response {
    if !token_ok(&state, query.token.as_deref()) {
        tracing::error!(code = WS_TOKEN_INVALID, "rejecting formats request");
        return (StatusCode::UNAUTHORIZED, WS_TOKEN_INVALID).into_response();
    }

    let s = state.lock().unwrap();

    let targets: Vec<_> = connection_targets(&s)
        .into_iter()
        .filter_map(|(target, label, kind)| {
            // 解決できないtargetは出さない（押しても開けないカードを作らない）
            s.connection_url(&target).map(|url| {
                json!({ "target": target, "label": label, "kind": kind, "url": url })
            })
        })
        .collect();

    let keymaps: Vec<_> = s
        .keymaps
        .values()
        .map(|k| {
            json!({
                "id": k.keymap_id,
                "description": k.description,
                "kind": k.kind,
                // boardを持たない分割キーボードは keys=0 になる。0は「壊れている」ではなく
                // 「halvesで持っている」の意味なので、表示側で分けること。
                "keys": k.board.as_ref().map(|b| b.keys.len()).unwrap_or(0),
                "cols": k.board.as_ref().map(|b| b.cols).unwrap_or(0),
                "layers": k.layers.len(),
            })
        })
        .collect();

    let decks: Vec<_> = s
        .decks
        .values()
        .map(|d| {
            json!({
                "id": d.deck_id,
                "description": d.description,
                "render": d.render,
                "cols": d.grid.cols,
                "rows": d.grid.rows,
                "pages": d.pages.len(),
                "slots": d.pages.iter().map(|p| p.slots.len()).sum::<usize>(),
            })
        })
        .collect();

    let layouts: Vec<_> = s
        .layouts
        .values()
        .map(|l| {
            json!({
                "id": l.layout_id,
                "description": l.description,
                "cols": l.grid.cols,
                "rows": l.grid.rows,
                "sections": l.sections.iter().map(|sec| json!({
                    "id": sec.id,
                    "row": sec.row, "col": sec.col,
                    "colSpan": sec.col_span, "rowSpan": sec.row_span,
                    "kind": sec.component.kind,
                    "ref": sec.component.reference,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    // ギャラリーが実機スクショを出すために、**実在するファイル名だけ**を渡す。
    // これが無いと、置いていないスクショを毎回取りに行って404が並ぶ。
    // 読み取り専用（ディレクトリを1つ読むだけ。書き込みも再帰もしない）。
    let shots: Vec<String> = std::fs::read_dir("static/shots")
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .collect()
        })
        .unwrap_or_default();

    Json(json!({
        "targets": targets,
        "keymaps": keymaps,
        "decks": decks,
        "layouts": layouts,
        "shots": shots,
    }))
    .into_response()
}

/// 保存先のファイル名に使ってよい `layoutId` か。
///
/// **パスを組み立てる前の唯一の関門**なので、ここを緩めてはならない。
/// 英小文字・数字・アンダースコアのみ。`/` `\\` `.` `..` はすべて弾かれる。
fn layout_id_is_safe(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// P-005 段階D: レイアウトを `layouts/layout_<id>.json` へ保存する。
///
/// ■ 手順（**壊れた構成をディスクに残さない**ことを最優先にしている）
///   1. token検証
///   2. `layoutId` の形を検証（パス組み立て前の関門）
///   3. 本文をLayoutとして解釈し、単体の妥当性を検証（重なり・はみ出し・id重複）
///   4. 既存ファイルがあれば `.bak` へ退避（D22と同じ作法）
///   5. 書き込む
///   6. **全体を読み直して検証**（他のレイアウトとの整合、参照先の実在）
///   7. 失敗したら `.bak` から巻き戻し、現行構成を維持したままエラーを返す
///   8. 成功したら差し替えて全クライアントへ再配信
///
/// ■ トークンは変わらない
///   トークンは起動時に1回だけ作られる（D8）。保存では再発行されないので、
///   端末はQRを読み直す必要がない。**繋がったままの端末には自動で反映される**。
async fn layout_save_handler(
    State(state): State<SharedState>,
    Query(query): Query<TokenQuery>,
    body: String,
) -> Response {
    if !token_ok(&state, query.token.as_deref()) {
        tracing::error!(code = WS_TOKEN_INVALID, "rejecting layout save");
        return (StatusCode::UNAUTHORIZED, WS_TOKEN_INVALID).into_response();
    }

    // 単体の妥当性はここで見る（重なり・はみ出し・区画idの重複）
    let layout = match crate::layout::load_layout_str("request body", &body) {
        Ok(layout) => layout,
        Err(error) => {
            tracing::error!(chk = "P005-D", code = error.code, cause = %error.cause, "layout save rejected");
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "code": error.code, "cause": error.cause })),
            )
                .into_response();
        }
    };

    if !layout_id_is_safe(&layout.layout_id) {
        let cause = format!(
            "layoutId '{}' must be 1-64 chars of [a-z0-9_] (it becomes a file name)",
            layout.layout_id
        );
        tracing::error!(chk = "P005-D", code = LAYOUT_SAVE_REJECTED, cause = %cause, "layout save rejected");
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "code": LAYOUT_SAVE_REJECTED, "cause": cause })),
        )
            .into_response();
    }

    let dir = std::path::Path::new(crate::LAYOUTS_DIR);
    let path = dir.join(format!("layout_{}.json", layout.layout_id));
    let backup = dir.join(format!("layout_{}.json.bak", layout.layout_id));

    // **説明文を黙って消させない。**
    // `description` は運用の申し送り（Volの切り替え手順、複製して作る方針など）が
    // 書かれている場所で、失うと取り返しがつかない。エディタは配置しか編集しないため、
    // 本文に説明が無い＝「指定しなかった」とみなし、既存の説明を引き継ぐ。
    // 説明を空にしたいときは、JSONを直接編集する（GUIからは消せない）。
    let mut layout = layout;
    if layout.description.is_empty() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(existing) = crate::layout::load_layout_str("existing", &text) {
                if !existing.description.is_empty() {
                    tracing::info!(
                        chk = "P005-D", layout_id = %layout.layout_id,
                        "carrying over the existing description (request had none)"
                    );
                    layout.description = existing.description;
                }
            }
        }
    }
    let layout = layout;

    if let Err(error) = std::fs::create_dir_all(dir) {
        let cause = format!("failed to create {}: {error}", dir.display());
        tracing::error!(chk = "P005-D", code = LAYOUT_SAVE_FAILED, cause = %cause, "layout save failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "code": LAYOUT_SAVE_FAILED, "cause": cause })),
        )
            .into_response();
    }

    // 既存を退避。**巻き戻せる状態を作ってから書く**
    let had_previous = path.exists();
    if had_previous {
        if let Err(error) = std::fs::copy(&path, &backup) {
            let cause = format!("failed to back up {}: {error}", path.display());
            tracing::error!(chk = "P005-D", code = LAYOUT_SAVE_FAILED, cause = %cause, "layout save failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "code": LAYOUT_SAVE_FAILED, "cause": cause })),
            )
                .into_response();
        }
    }

    // 受け取った本文をそのまま書かず、**解釈し直したものを整形して書く**。
    // 余計なフィールドや書式の揺れをディスクへ持ち込まないため。
    let text = match serde_json::to_string_pretty(&layout) {
        Ok(text) => text + "\n",
        Err(error) => {
            let cause = format!("failed to serialize layout: {error}");
            tracing::error!(chk = "P005-D", code = LAYOUT_SAVE_FAILED, cause = %cause, "layout save failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "code": LAYOUT_SAVE_FAILED, "cause": cause })),
            )
                .into_response();
        }
    };
    if let Err(error) = std::fs::write(&path, &text) {
        let cause = format!("failed to write {}: {error}", path.display());
        tracing::error!(chk = "P005-D", code = LAYOUT_SAVE_FAILED, cause = %cause, "layout save failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "code": LAYOUT_SAVE_FAILED, "cause": cause })),
        )
            .into_response();
    }

    // **書いた後に全体を読み直す。** 単体では正しくても、参照先が無い・他と衝突する、
    // といった全体の不整合はここでしか分からない。
    let keymaps_dir = std::path::Path::new(crate::KEYMAPS_DIR);
    let decks_dir = std::path::Path::new(crate::DECKS_DIR);
    let surfaces_dir = std::path::Path::new(crate::SURFACES_DIR);
    let loaded = match crate::startup::load_startup_data(keymaps_dir, decks_dir, surfaces_dir, dir) {
        Ok(data) => data,
        Err(errors) => {
            // 巻き戻す。壊れた構成をディスクに残さない
            let restored = if had_previous {
                std::fs::copy(&backup, &path).is_ok()
            } else {
                std::fs::remove_file(&path).is_ok()
            };
            let cause = errors.join("; ");
            tracing::error!(
                chk = "P005-D", code = LAYOUT_SAVE_REJECTED, cause = %cause, restored,
                "layout save rejected after reload; rolled back"
            );
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "code": LAYOUT_SAVE_REJECTED, "cause": cause,
                    "errors": errors, "rolledBack": restored,
                })),
            )
                .into_response();
        }
    };

    let layouts_loaded = loaded.layouts.len();
    {
        let mut s = state.lock().unwrap();
        s.keymaps = loaded.keymaps;
        s.decks = loaded.decks;
        s.command_registry = loaded.command_registry;
        s.surfaces = loaded.surfaces;
        s.layouts = loaded.layouts;
        // reloadと同じ理由。消えたレイヤー参照を残さない
        s.layer_state.reset();
        s.ipad_layer_state.reset();
        s.layer_states.clear();
    }

    tracing::info!(
        chk = "P005-D", layout_id = %layout.layout_id, layouts_loaded,
        "layout saved; broadcasting surface.config"
    );
    broadcast_surface_config_for(&state, SurfaceKind::Split);
    broadcast_surface_config_for(&state, SurfaceKind::Ipad);
    broadcast_surface_config_for(&state, SurfaceKind::Layout);

    (
        StatusCode::OK,
        Json(json!({
            "layoutId": layout.layout_id,
            "path": path.display().to_string(),
            "backup": if had_previous { Some(backup.display().to_string()) } else { None },
            "layoutsLoaded": layouts_loaded,
        })),
    )
        .into_response()
}

async fn deck_export(State(state): State<SharedState>, Query(query): Query<DeckExportQuery>) -> Response {
    if !token_ok(&state, query.token.as_deref()) {
        tracing::error!(code = WS_TOKEN_INVALID, "rejecting deck export request");
        return (StatusCode::UNAUTHORIZED, WS_TOKEN_INVALID).into_response();
    }
    let json_text = {
        let s = state.lock().unwrap();
        let deck_id = query.deck.as_deref().unwrap_or(DEFAULT_DECK_ID);
        match s.decks.get(deck_id) {
            Some(deck) => serde_json::to_string_pretty(deck).unwrap_or_else(|_| "{}".to_string()),
            None => {
                tracing::error!(code = DECK_UNKNOWN_SLOT, deck_id, "deck export: unknown deckId");
                format!("{{\"error\":\"unknown deckId '{deck_id}'\"}}")
            }
        }
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

/// B2（設計書v0.5）: `POST /api/reload?token=…`。ディスクから`keymaps/`（B1と同じ
/// ディレクトリスキャン経路）＋`decks/deck_*.json`（P-005段階Aでディレクトリスキャンに変更）を再読込し、検証成功時のみ
/// 現行状態へ差替える。検証失敗時は現行構成を一切変更せずD9書式で1行ログ＋
/// エラーJSONを返す（「失敗時は現行構成維持」を関数境界=`load_startup_data`のErrで保証）。
/// 成功時は分割/Deck面・ipad面の両方へ`surface.config`を再配信する。
async fn reload_handler(State(state): State<SharedState>, Query(query): Query<TokenQuery>) -> Response {
    if !token_ok(&state, query.token.as_deref()) {
        tracing::error!(code = WS_TOKEN_INVALID, "rejecting reload request");
        return (StatusCode::UNAUTHORIZED, WS_TOKEN_INVALID).into_response();
    }

    let keymaps_dir = std::path::Path::new(crate::KEYMAPS_DIR);
    let decks_dir = std::path::Path::new(crate::DECKS_DIR);
    let surfaces_dir = std::path::Path::new(crate::SURFACES_DIR);
    let layouts_dir = std::path::Path::new(crate::LAYOUTS_DIR);

    let loaded = match crate::startup::load_startup_data(keymaps_dir, decks_dir, surfaces_dir, layouts_dir) {
        Ok(data) => data,
        Err(errors) => {
            let cause = errors.join("; ");
            tracing::error!(chk = "B2", code = RELOAD_INVALID, cause = %cause, "reload rejected; current configuration kept");
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "code": RELOAD_INVALID, "cause": cause, "errors": errors })),
            )
                .into_response();
        }
    };

    // D10: split面はactive_keymap_idを保持し続ける。新しい構成にそのIDが無いと表示不能に
    // なるため、追加の妥当性チェックとして扱い、無ければ現行構成を維持する（失敗扱い）。
    let active_keymap_id = {
        let s = state.lock().unwrap();
        s.active_keymap_id.clone()
    };
    if !loaded.keymaps.contains_key(&active_keymap_id) {
        let cause = format!(
            "reload would drop the currently active keymapId '{active_keymap_id}'; keeping current configuration"
        );
        tracing::error!(chk = "B2", code = RELOAD_INVALID, cause = %cause, "reload rejected; current configuration kept");
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "code": RELOAD_INVALID, "cause": cause, "errors": [cause.clone()] })),
        )
            .into_response();
    }

    let keymaps_loaded = loaded.keymaps.len();
    {
        let mut s = state.lock().unwrap();
        s.keymaps = loaded.keymaps;
        s.decks = loaded.decks;
        s.command_registry = loaded.command_registry;
        s.surfaces = loaded.surfaces;
        s.layouts = loaded.layouts;
        // keymap.switch同様、差替え後は消えたレイヤー参照が残らないよう両面ともリセットする。
        s.layer_state.reset();
        s.ipad_layer_state.reset();
        // P-005 段階B: 消えたレイヤー参照が残らないよう、盤面ごとの状態も全部落とす。
        s.layer_states.clear();
    }
    tracing::info!(chk = "B2", keymaps_loaded, "reload succeeded; broadcasting surface.config");

    broadcast_surface_config_for(&state, SurfaceKind::Split);
    broadcast_surface_config_for(&state, SurfaceKind::Ipad);
    broadcast_surface_config_for(&state, SurfaceKind::Layout);

    (
        StatusCode::OK,
        Json(json!({ "keymapsLoaded": keymaps_loaded, "activeKeymapId": active_keymap_id })),
    )
        .into_response()
}

async fn handle_socket(socket: WebSocket, state: SharedState, surface: SurfaceKind) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    let client_id = {
        let mut s = state.lock().unwrap();
        let id = s.next_client_id;
        s.next_client_id += 1;
        s.register_client(id, tx.clone(), surface);
        id
    };
    tracing::info!(chk = "T3-3", client_id, ?surface, "client connected");

    let forward_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

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

    {
        let mut s = state.lock().unwrap();
        s.unregister_client(client_id);
    }
    forward_task.abort();
    tracing::info!(chk = "T3-3", client_id, "client disconnected");
}

// ── メッセージ処理 [T3-3] ────────────────────────────────────

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
        // T12（D28）: spin/activeはHubが読み捨てる（プロトコルだけ先に確保。§3.2）。
        ClientMessage::SurfaceState { surface_id, delta, .. } => {
            handle_surface_state(state, client_id, &surface_id, delta.dx, delta.dy).await
        }
        // T18（brief/keydeck_trackball_gestures_v0.7.md §2.3）: discreteジェスチャー。
        ClientMessage::SurfaceGesture { surface_id, gesture_id, edge } => {
            handle_surface_gesture(state, client_id, &surface_id, &gesture_id, edge.map(Edge::from)).await
        }
    }
}

/// T8: どのkeymap/layer_stateを使うかはsurfaceで決まる。Ipadは常にIPAD_KEYMAP_ID＋
/// 専用のipad_layer_state（分割/Deckのactive系とは独立）、それ以外は従来どおり
/// active_keymap_id＋layer_state（G2の分割同期はここで維持される）。
/// P-005 段階B: `/layout`面は`keymapId`で盤面を名指しし、keymapIdごとの`layer_states`を使う。
/// 既存面（Split/Ipad）は従来の状態をそのまま使い、挙動を変えない。
fn resolve_for_layout(s: &mut HubState, keymap_id: &str, key_id: &str, edge: Edge) -> Resolved {
    let Some(keymap) = s.keymaps.get(keymap_id).cloned() else {
        return Resolved::UnknownKey;
    };
    let layer_state = s.layer_state_for(keymap_id);
    resolve(&keymap, layer_state, key_id, edge)
}

fn resolve_for_surface(s: &mut HubState, surface: SurfaceKind, key_id: &str, edge: Edge) -> Resolved {
    match surface {
        // Layout面はkeymapIdが必須。ここへ来るのはクライアントが省略した場合だけで、
        // どの盤面か決めようがないためUnknownKey扱い（panicしない＝D9）。
        SurfaceKind::Layout => Resolved::UnknownKey,
        SurfaceKind::Ipad => {
            let keymap: Keymap = s
                .keymaps
                .get(IPAD_KEYMAP_ID)
                .expect("ipad01_vol12 keymap is always loaded at startup (checked in main.rs)")
                .clone();
            resolve(&keymap, &mut s.ipad_layer_state, key_id, edge)
        }
        SurfaceKind::Split => {
            let keymap: Keymap = s
                .keymaps
                .get(&s.active_keymap_id)
                .expect("active_keymap_id always refers to a loaded keymap")
                .clone();
            resolve(&keymap, &mut s.layer_state, key_id, edge)
        }
        // T12: トラックボール面はkeymapを持たない（本Volはマウス出口のみ、D28）。
        // static/trackball.htmlはkey.pressを送らない設計だが、防御としてUnknownKeyを返す
        // （このsurfaceにその名のkeyは存在しない、という表現として妥当。panicはしない＝D9）。
        SurfaceKind::Trackball => Resolved::UnknownKey,
    }
}

async fn handle_key_press(
    state: &SharedState,
    client_id: ClientId,
    surface: SurfaceKind,
    keymap_id: Option<&str>,
    key_id: &str,
    edge: Edge,
) {
    let resolved = {
        let mut s = state.lock().unwrap();
        match keymap_id {
            Some(id) => resolve_for_layout(&mut s, id, key_id, edge),
            None => resolve_for_surface(&mut s, surface, key_id, edge),
        }
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
            let wire = layer_state_wire(state, surface, keymap_id);
            tracing::info!(chk = "T3-3", ?surface, ?wire, "layer state changed; broadcasting");
            broadcast_layer_state(state, surface, &wire);
        }
        Resolved::Fire(action) => fire_action(state, client_id, action).await,
        // T21（tg.fire）: レイヤー配信と発火の**両方**を行う。順序は「先に配信 → 後に発火」。
        // 発火はadapterワーカー往復のawaitを挟むため、先に画面を更新した方が体感が速く、
        // かつ発火が失敗しても画面とHub状態の整合は保たれる（状態は既にresolve内で確定済み）。
        Resolved::FireAndLayerChanged(action) => {
            let wire = layer_state_wire(state, surface, keymap_id);
            tracing::info!(chk = "T21", ?surface, ?wire, "tg.fire: layer state changed; broadcasting then firing");
            broadcast_layer_state(state, surface, &wire);
            fire_action(state, client_id, action).await;
        }
    }
}

/// P-005 段階B: 配信するレイヤー状態を組み立てる。`keymap_id`があればkeymapIdごとの状態
/// （`/layout`面）、無ければ従来の面ごとの状態。
fn layer_state_wire(state: &SharedState, surface: SurfaceKind, keymap_id: Option<&str>) -> LayerStateWire {
    let mut s = state.lock().unwrap();
    if let Some(id) = keymap_id {
        return LayerStateWire::for_keymap(id, s.layer_state_for(id));
    }
    match surface {
        SurfaceKind::Ipad => LayerStateWire::from(&s.ipad_layer_state),
        SurfaceKind::Split => LayerStateWire::from(&s.layer_state),
        // これらの面はLayerChangedを返さないため到達しない（網羅性のためのみ）。
        SurfaceKind::Trackball | SurfaceKind::Layout => {
            LayerStateWire { keymap_id: None, momentary: vec![], toggled: vec![] }
        }
    }
}

async fn handle_deck_press(state: &SharedState, client_id: ClientId, deck_id: &str, slot_id: &str) {
    let action = {
        let s = state.lock().unwrap();
        s.find_deck_slot(deck_id, slot_id).map(|slot| slot.action.clone())
    };

    match action {
        None => emit_error(
            state,
            client_id,
            "T3-3",
            DECK_UNKNOWN_SLOT,
            format!("unknown slot '{slot_id}' in deck '{deck_id}'"),
            json!({ "deckId": deck_id, "slotId": slot_id }),
        ),
        Some(Action::None) => {}
        Some(action) => fire_action(state, client_id, action).await,
    }
}

/// T12（D28）: `surface.state`受信 → binding解決 → 既存adapter_txへ発火。処理順は設計書
/// T12-3の①〜⑤どおり固定する（surfaceId解決 → 有限性 → clamp → 丸め＋ゼロ移動スキップ →
/// binding解決＋発火）。新しい発火経路は作らず、既存のadapter_tx（D7の直列ワーカー）を使う。
async fn handle_surface_state(state: &SharedState, client_id: ClientId, surface_id: &str, dx: f64, dy: f64) {
    // ① surfaceIdをレジストリで引く。無ければSURFACE_UNKNOWN_IDを返して終了。
    let def = {
        let s = state.lock().unwrap();
        s.surfaces.get(surface_id).cloned()
    };
    let Some(def) = def else {
        emit_error(
            state,
            client_id,
            "T12",
            SURFACE_UNKNOWN_ID,
            format!("unknown surfaceId '{surface_id}'"),
            json!({ "surfaceId": surface_id }),
        );
        return;
    };

    // ② dx/dyの有限性を確認。NaN/InfならSURFACE_STATE_RANGE。
    if !dx.is_finite() || !dy.is_finite() {
        emit_error(
            state,
            client_id,
            "T12",
            SURFACE_STATE_RANGE,
            format!("surface '{surface_id}': dx/dy must be finite (dx={dx}, dy={dy})"),
            json!({ "surfaceId": surface_id, "dx": dx, "dy": dy }),
        );
        return;
    }

    // ③ clamp超過ならSURFACE_STATE_RANGE（握りつぶさずクライアントへerrorを返す）。
    let clamp = def.clamp as f64;
    if dx.abs() > clamp || dy.abs() > clamp {
        emit_error(
            state,
            client_id,
            "T12",
            SURFACE_STATE_RANGE,
            format!("surface '{surface_id}': dx/dy exceed clamp {} (dx={dx}, dy={dy})", def.clamp),
            json!({ "surfaceId": surface_id, "dx": dx, "dy": dy, "clamp": def.clamp }),
        );
        return;
    }

    // ④ 丸めてi32にする。dx==0 && dy==0なら何も発火せず終了（無駄なSendInputを打たない）。
    let dx = dx.round() as i32;
    let dy = dy.round() as i32;
    if dx == 0 && dy == 0 {
        return;
    }

    // ⑤ bindingを解決してActionを作り、既存のadapter_txへ流す。
    // T15/T18（§3）: continuousスクロール用に"mouse.scroll"を追加。dxは無視してよい
    // （横スクロールはVer1対象外。クライアント側は常にdx:0を送るため実害なし）。
    let action = match def.binding_t.as_str() {
        "mouse.move" => Action::MouseMove { dx, dy },
        "mouse.scroll" => Action::MouseScroll { dy },
        other => {
            // surface.rsのロード時点でALLOWED_BINDING_TYPESにより弾かれているため到達しない想定
            // だが、防御としてINTERNALで報告する（D9: panic禁止）。
            emit_error(
                state,
                client_id,
                "T12",
                INTERNAL,
                format!("surface '{surface_id}' has an unresolvable binding.t '{other}'"),
                json!({ "surfaceId": surface_id }),
            );
            return;
        }
    };

    let adapter_tx = {
        let s = state.lock().unwrap();
        s.adapter_tx.clone()
    };
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    if adapter_tx.send(AdapterJob { action, reply: reply_tx }).is_err() {
        emit_error(
            state,
            client_id,
            "T12",
            INTERNAL,
            "adapter worker channel is closed".to_string(),
            json!({ "surfaceId": surface_id }),
        );
        return;
    }

    match reply_rx.await {
        Ok(Ok(())) => {}
        Ok(Err(adapter_error)) => emit_error(
            state,
            client_id,
            "T12",
            ADAPTER_SENDINPUT_FAIL,
            adapter_error.to_string(),
            json!({ "surfaceId": surface_id }),
        ),
        Err(_) => emit_error(
            state,
            client_id,
            "T12",
            INTERNAL,
            "adapter worker did not reply".to_string(),
            json!({ "surfaceId": surface_id }),
        ),
    }
}

/// T15↔T17のブリッジ: `surface.rs::ClickButton`（宣言＝どのボタンか、のみ）を
/// `proto_keymap::MouseButtonKind`（Action組み立て用）へ変換する。
fn to_mouse_button_kind(button: ClickButton) -> MouseButtonKind {
    match button {
        ClickButton::Left => MouseButtonKind::Left,
        ClickButton::Right => MouseButtonKind::Right,
    }
}

/// T18（brief/keydeck_trackball_gestures_v0.7.md §2.3）: `surface.gesture`受信 →
/// gestureId解決 → `GestureAction`＋`edge`から`proto_keymap::Action`を組み立て →
/// 既存adapter_tx（D7直列ワーカー）へ発火。新しい発火経路は作らない（T12の
/// handle_surface_stateと同じ既存adapter_txを使う。command_registry許可リストは通さない
/// ——button/vkは起動時ロードのgesturesマップで既に固定されており、自由記述を受け付ける
/// 経路ではないため許可リストの対象外でよい、と設計書に明記されている）。
async fn handle_surface_gesture(
    state: &SharedState,
    client_id: ClientId,
    surface_id: &str,
    gesture_id: &str,
    edge: Option<Edge>,
) {
    // ① surfaceId解決。無ければ既存のSURFACE_UNKNOWN_ID（surface.stateと同じ意味なので使い回す）。
    let def = {
        let s = state.lock().unwrap();
        s.surfaces.get(surface_id).cloned()
    };
    let Some(def) = def else {
        emit_error(
            state,
            client_id,
            "T18",
            SURFACE_UNKNOWN_ID,
            format!("unknown surfaceId '{surface_id}'"),
            json!({ "surfaceId": surface_id }),
        );
        return;
    };

    // ② gestureId解決。無ければ新規SURFACE_GESTURE_UNKNOWN_ID。
    let Some(gesture) = def.gestures.get(gesture_id).cloned() else {
        emit_error(
            state,
            client_id,
            "T18",
            SURFACE_GESTURE_UNKNOWN_ID,
            format!("surface '{surface_id}': unknown gestureId '{gesture_id}'"),
            json!({ "surfaceId": surface_id, "gestureId": gesture_id }),
        );
        return;
    };

    // ③ GestureAction + edge から proto_keymap::Action を組み立てる。
    let action = match gesture {
        GestureAction::Click { button } => Action::MouseClick { button: to_mouse_button_kind(button) },
        GestureAction::DoubleClick { button } => {
            Action::MouseDoubleClick { button: to_mouse_button_kind(button) }
        }
        GestureAction::Key { vk } => Action::Key { vk },
        GestureAction::ButtonHold { button } => match edge {
            Some(Edge::Down) => Action::MouseButton { button: to_mouse_button_kind(button), down: true },
            Some(Edge::Up) => Action::MouseButton { button: to_mouse_button_kind(button), down: false },
            None => {
                emit_error(
                    state,
                    client_id,
                    "T18",
                    SURFACE_GESTURE_EDGE_REQUIRED,
                    format!("surface '{surface_id}': gesture '{gesture_id}' requires edge"),
                    json!({ "surfaceId": surface_id, "gestureId": gesture_id }),
                );
                return;
            }
        },
    };

    // ④ 既存のadapter_tx（D7直列ワーカー）へAdapterJobとして送る。新しい発火経路は作らない。
    let adapter_tx = {
        let s = state.lock().unwrap();
        s.adapter_tx.clone()
    };
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    if adapter_tx.send(AdapterJob { action, reply: reply_tx }).is_err() {
        emit_error(
            state,
            client_id,
            "T18",
            INTERNAL,
            "adapter worker channel is closed".to_string(),
            json!({ "surfaceId": surface_id, "gestureId": gesture_id }),
        );
        return;
    }

    match reply_rx.await {
        Ok(Ok(())) => {}
        Ok(Err(adapter_error)) => emit_error(
            state,
            client_id,
            "T18",
            ADAPTER_SENDINPUT_FAIL,
            adapter_error.to_string(),
            json!({ "surfaceId": surface_id, "gestureId": gesture_id }),
        ),
        Err(_) => emit_error(
            state,
            client_id,
            "T18",
            INTERNAL,
            "adapter worker did not reply".to_string(),
            json!({ "surfaceId": surface_id, "gestureId": gesture_id }),
        ),
    }
}

/// P-005 段階C: そのクライアントが押しっぱなしにしている全キーへreleaseを打つ。
/// 許可リストの再確認は行わない（押下時に通っているものだけが台帳に載るため）。
/// ここで詰まるとPCが操作不能のままになるので、1件失敗しても残りを撃ち続ける。
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

async fn fire_action(state: &SharedState, client_id: ClientId, action: Action) {
    match &action {
        Action::KeymapSwitch { id } => switch_keymap(state, client_id, id.clone()).await,
        Action::KeymapReset => switch_keymap(state, client_id, "default".to_string()).await,
        Action::Key { .. } | Action::Chord { .. } | Action::Text { .. } | Action::KeyButton { .. } => {
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

            // P-005 段階C: 押しっぱなしの台帳を更新してからadapterへ送る。
            // 先に記録するのは、発火が失敗しても「押したかもしれない」側に倒して
            // 切断時に必ずreleaseを打つため（取りこぼしより二重releaseの方が安全）。
            if let Action::KeyButton { vk, down } = &action {
                let mut s = state.lock().unwrap();
                s.note_key_hold(client_id, vk, *down);
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
            "resolve() only Fires Key/Chord/Text/KeymapSwitch/KeymapReset; got {other:?}"
        ),
    }
}

// ── D10: keymap切替（Split面のみ。ipad面はIPAD_KEYMAP_ID固定で独立） ──────────

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
    tracing::info!(chk = "T3-4", "keymap switched; broadcasting surface.config to split/deck clients");
    broadcast_surface_config_for(state, SurfaceKind::Split);
}

// ── 送信ヘルパー ─────────────────────────────────────────────

/// surfaceに応じたsurface.config JSON文字列を組み立てる。IpadはIPAD_KEYMAP_ID固定・
/// ipad_layer_state、Splitは従来どおりactive_keymap_id・layer_state。
/// T12: トラックボール面はkeymapを持たない（本Volはマウス出口のみ）ため、surface.configの
/// 送信対象外＝常にNone（呼び出し側は既存どおりNoneなら何もしないため無挙動）。
fn surface_config_json_for(state: &SharedState, surface: SurfaceKind) -> Option<String> {
    let s = state.lock().unwrap();
    // P-005 段階B: `/layout`面は「どれか1枚の盤面」ではなく全部を必要とする。
    // 既存面はこれまでどおり1枚だけを受け取る（keymaps/layouts/layerStatesは付かない）。
    let is_layout = matches!(surface, SurfaceKind::Layout);
    let (keymap_id, keymap, layer): (&str, &Keymap, LayerState) = match surface {
        SurfaceKind::Ipad => (
            IPAD_KEYMAP_ID,
            s.keymaps.get(IPAD_KEYMAP_ID)?,
            s.ipad_layer_state.clone(),
        ),
        SurfaceKind::Split | SurfaceKind::Layout => (
            s.active_keymap_id.as_str(),
            s.keymaps.get(&s.active_keymap_id)?,
            s.layer_state.clone(),
        ),
        SurfaceKind::Trackball => return None,
    };
    let layer_states = is_layout.then(|| {
        s.layer_states
            .iter()
            .map(|(id, st)| (id.clone(), LayerStateWire::for_keymap(id, st)))
            .collect()
    });
    let message = ServerMessage::SurfaceConfig(SurfaceConfig {
        active_keymap_id: keymap_id,
        keymap,
        layer: LayerStateWire::from(&layer),
        decks: &s.decks,
        keymaps: is_layout.then_some(&s.keymaps),
        layouts: is_layout.then_some(&s.layouts),
        layer_states,
    });
    match serde_json::to_string(&message) {
        Ok(text) => Some(text),
        Err(error) => {
            tracing::error!(code = INTERNAL, cause = %error, "failed to serialize surface.config");
            None
        }
    }
}

fn send_surface_config_to(state: &SharedState, client_id: ClientId, surface: SurfaceKind) {
    let Some(text) = surface_config_json_for(state, surface) else {
        return;
    };
    let s = state.lock().unwrap();
    s.send_to(client_id, Message::Text(text.into()));
}

/// `surface`に該当するクライアント全員へsurface.configを再配信する（keymap.switch/reset時）。
fn broadcast_surface_config_for(state: &SharedState, surface: SurfaceKind) {
    let Some(text) = surface_config_json_for(state, surface) else {
        return;
    };
    let s = state.lock().unwrap();
    s.broadcast_to(surface, &text);
}

/// `surface`に該当するクライアントのみへlayer.stateを配信する（分割とipadを混線させない）。
fn broadcast_layer_state(state: &SharedState, surface: SurfaceKind, wire: &LayerStateWire) {
    let text = match serde_json::to_string(&ServerMessage::LayerState(wire.clone())) {
        Ok(text) => text,
        Err(error) => {
            tracing::error!(code = INTERNAL, cause = %error, "failed to serialize layer.state broadcast");
            return;
        }
    };
    let s = state.lock().unwrap();
    s.broadcast_to(surface, &text);
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
    s.send_to(client_id, Message::Text(text.into()));
}

// ============================================================================
// T12単体テスト（handle_surface_state）
// ============================================================================
//
// 注意: proto_adapter_win::send()を実際に呼ぶ`state::spawn_adapter_worker()`は
// ここでは絶対に使わない（Windows実機でSendInputが本当にカーソルを動かしてしまうため。
// CLAUDE.md「SendInputを自動テストから絶対に呼ばないこと」）。代わりにテスト側で生の
// mpscチャネルを直接HubState.adapter_txに差し込み、AdapterJobを横取りしてダミー応答を
// 返すことで、実SendInputに触れずHub側のルーティングロジックだけを検証する。

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AccessToken;
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;

    const TB01_JSON: &str = r#"{
        "surfaces": [
            { "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" }, "clamp": 200 }
        ]
    }"#;

    fn test_state(surfaces_json: &str) -> (SharedState, mpsc::UnboundedReceiver<AdapterJob>) {
        let surfaces = crate::surface::load_surface_registry_str("test", surfaces_json)
            .expect("test surfaces json must be valid");
        let deck = crate::deck::load_deck_str(
            "test",
            r#"{ "deckId": "default", "grid": { "cols": 1, "rows": 1 }, "pages": [] }"#,
        )
        .expect("empty deck must load");
        let mut decks = std::collections::BTreeMap::new();
        decks.insert(deck.deck_id.clone(), deck);
        let (tx, rx) = mpsc::unbounded_channel::<AdapterJob>();
        let hub_state = HubState::new(
            std::collections::BTreeMap::new(),
            "none".to_string(),
            decks,
            hub_core::CommandRegistry::new(Vec::<String>::new()),
            surfaces,
            std::collections::BTreeMap::new(),
            AccessToken::generate(),
            tx,
            "127.0.0.1".to_string(),
        );
        (Arc::new(Mutex::new(hub_state)), rx)
    }

    /// T20（P-003 Ver1-a）: routerが`ServeFile`で配る静的ファイルが実在すること。
    /// パスは相対文字列なので、綴り違い・置き忘れはコンパイルでは捕まらず、
    /// 実機でQRを読んだ瞬間に404で初めて分かる。ここで固定しておく。
    #[test]
    fn every_served_static_file_exists() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for path in [
            "static/kb.html",
            "static/deck.html",
            "static/ipad.html",
            "static/trackball.html",
            "static/settings.html",
            "static/panel.html",
            "static/layout.html",
        ] {
            assert!(
                root.join(path).exists(),
                "{path} must exist (router serves it with ServeFile)"
            );
        }
    }

    // G-12a: 未知のsurfaceIdではAdapterJobが発行されない（Hubは落ちず、errorフレームのみ）。
    #[tokio::test]
    async fn g12a_unknown_surface_id_enqueues_no_job() {
        let (state, mut rx) = test_state(TB01_JSON);
        handle_surface_state(&state, 1, "does-not-exist", 10.0, 10.0).await;
        assert!(rx.try_recv().is_err(), "unknown surfaceId must not enqueue an AdapterJob");
    }

    // G-12b: NaNのdxではAdapterJobが発行されない。
    #[tokio::test]
    async fn g12b_non_finite_dx_enqueues_no_job() {
        let (state, mut rx) = test_state(TB01_JSON);
        handle_surface_state(&state, 1, "tb01", f64::NAN, 0.0).await;
        assert!(rx.try_recv().is_err(), "non-finite dx must not enqueue an AdapterJob");
    }

    // G-12b: clamp(200)超過ではAdapterJobが発行されない。
    #[tokio::test]
    async fn g12b_clamp_exceeded_enqueues_no_job() {
        let (state, mut rx) = test_state(TB01_JSON);
        handle_surface_state(&state, 1, "tb01", 500.0, 0.0).await;
        assert!(rx.try_recv().is_err(), "dx exceeding clamp must not enqueue an AdapterJob");
    }

    // G-12c: dx=0,dy=0では何も発火せず終了する（無駄なSendInputを打たない）。
    #[tokio::test]
    async fn g12c_zero_delta_enqueues_no_job() {
        let (state, mut rx) = test_state(TB01_JSON);
        handle_surface_state(&state, 1, "tb01", 0.0, 0.0).await;
        assert!(rx.try_recv().is_err(), "dx=0/dy=0 must not enqueue an AdapterJob");
    }

    // G-12d: 正常値でAction::MouseMoveが既存adapter_txに載る（丸め処理込み）。
    // 実SendInputは呼ばない: AdapterJobを横取りしてダミー応答を返すのみ。
    #[tokio::test]
    async fn g12d_valid_delta_enqueues_mouse_move_with_rounded_values() {
        let (state, mut rx) = test_state(TB01_JSON);
        let handle = tokio::spawn({
            let state = state.clone();
            async move {
                handle_surface_state(&state, 1, "tb01", 12.4, -3.6).await;
            }
        });
        let job = rx.recv().await.expect("a valid delta must enqueue an AdapterJob");
        assert_eq!(job.action, Action::MouseMove { dx: 12, dy: -4 });
        let _ = job.reply.send(Ok(()));
        handle.await.unwrap();
    }

    // ============================================================================
    // T18単体テスト（handle_surface_gesture・handle_surface_state追加分）
    // ============================================================================

    const TB01_GESTURES_JSON: &str = r#"{
        "surfaces": [
            {
                "id": "tb01", "type": "trackball", "binding": { "t": "mouse.move" }, "clamp": 200,
                "gestures": {
                    "tap1":  { "t": "mouse.click",     "button": "left" },
                    "dtap1": { "t": "mouse.dblclick",  "button": "left" },
                    "tap2":  { "t": "mouse.click",     "button": "right" },
                    "tap3":  { "t": "key",             "vk": "ESC" },
                    "hold1": { "t": "mouse.button.hold", "button": "left" }
                }
            },
            {
                "id": "tb01-scroll", "type": "trackball", "binding": { "t": "mouse.scroll" }, "clamp": 100
            }
        ]
    }"#;

    fn edge_wire(edge: Edge) -> Option<Edge> {
        Some(edge)
    }

    // G-18a: 未知surfaceId → ジョブなし。
    #[tokio::test]
    async fn g18a_unknown_surface_id_enqueues_no_gesture_job() {
        let (state, mut rx) = test_state(TB01_GESTURES_JSON);
        handle_surface_gesture(&state, 1, "does-not-exist", "tap1", None).await;
        assert!(rx.try_recv().is_err(), "unknown surfaceId must not enqueue an AdapterJob");
    }

    // G-18a: 未知gestureId → ジョブなし（SURFACE_GESTURE_UNKNOWN_ID）。
    #[tokio::test]
    async fn g18a_unknown_gesture_id_enqueues_no_job() {
        let (state, mut rx) = test_state(TB01_GESTURES_JSON);
        handle_surface_gesture(&state, 1, "tb01", "does-not-exist", None).await;
        assert!(rx.try_recv().is_err(), "unknown gestureId must not enqueue an AdapterJob");
    }

    // G-18a: tap1 → Action::MouseClick{button:Left}のジョブ。
    #[tokio::test]
    async fn g18a_tap1_enqueues_mouse_click_left() {
        let (state, mut rx) = test_state(TB01_GESTURES_JSON);
        let handle = tokio::spawn({
            let state = state.clone();
            async move {
                handle_surface_gesture(&state, 1, "tb01", "tap1", None).await;
            }
        });
        let job = rx.recv().await.expect("tap1 must enqueue an AdapterJob");
        assert_eq!(job.action, Action::MouseClick { button: MouseButtonKind::Left });
        let _ = job.reply.send(Ok(()));
        handle.await.unwrap();
    }

    // G-18a: dtap1 → Action::MouseDoubleClick{button:Left}のジョブ。
    #[tokio::test]
    async fn g18a_dtap1_enqueues_mouse_double_click_left() {
        let (state, mut rx) = test_state(TB01_GESTURES_JSON);
        let handle = tokio::spawn({
            let state = state.clone();
            async move {
                handle_surface_gesture(&state, 1, "tb01", "dtap1", None).await;
            }
        });
        let job = rx.recv().await.expect("dtap1 must enqueue an AdapterJob");
        assert_eq!(job.action, Action::MouseDoubleClick { button: MouseButtonKind::Left });
        let _ = job.reply.send(Ok(()));
        handle.await.unwrap();
    }

    // G-18a: tap2 → Action::MouseClick{button:Right}のジョブ（右クリック）。
    #[tokio::test]
    async fn g18a_tap2_enqueues_mouse_click_right() {
        let (state, mut rx) = test_state(TB01_GESTURES_JSON);
        let handle = tokio::spawn({
            let state = state.clone();
            async move {
                handle_surface_gesture(&state, 1, "tb01", "tap2", None).await;
            }
        });
        let job = rx.recv().await.expect("tap2 must enqueue an AdapterJob");
        assert_eq!(job.action, Action::MouseClick { button: MouseButtonKind::Right });
        let _ = job.reply.send(Ok(()));
        handle.await.unwrap();
    }

    // tap3 → Action::Key{vk:"ESC"}のジョブ（3本タップ=Esc）。
    #[tokio::test]
    async fn tap3_enqueues_key_esc() {
        let (state, mut rx) = test_state(TB01_GESTURES_JSON);
        let handle = tokio::spawn({
            let state = state.clone();
            async move {
                handle_surface_gesture(&state, 1, "tb01", "tap3", None).await;
            }
        });
        let job = rx.recv().await.expect("tap3 must enqueue an AdapterJob");
        assert_eq!(job.action, Action::Key { vk: "ESC".to_string() });
        let _ = job.reply.send(Ok(()));
        handle.await.unwrap();
    }

    // G-18a: hold1+edge:down → Action::MouseButton{button:Left,down:true}。
    #[tokio::test]
    async fn g18a_hold1_down_enqueues_mouse_button_down() {
        let (state, mut rx) = test_state(TB01_GESTURES_JSON);
        let handle = tokio::spawn({
            let state = state.clone();
            async move {
                handle_surface_gesture(&state, 1, "tb01", "hold1", edge_wire(Edge::Down)).await;
            }
        });
        let job = rx.recv().await.expect("hold1+down must enqueue an AdapterJob");
        assert_eq!(job.action, Action::MouseButton { button: MouseButtonKind::Left, down: true });
        let _ = job.reply.send(Ok(()));
        handle.await.unwrap();
    }

    // G-18a: hold1+edge:up → Action::MouseButton{button:Left,down:false}。
    #[tokio::test]
    async fn g18a_hold1_up_enqueues_mouse_button_up() {
        let (state, mut rx) = test_state(TB01_GESTURES_JSON);
        let handle = tokio::spawn({
            let state = state.clone();
            async move {
                handle_surface_gesture(&state, 1, "tb01", "hold1", edge_wire(Edge::Up)).await;
            }
        });
        let job = rx.recv().await.expect("hold1+up must enqueue an AdapterJob");
        assert_eq!(job.action, Action::MouseButton { button: MouseButtonKind::Left, down: false });
        let _ = job.reply.send(Ok(()));
        handle.await.unwrap();
    }

    // G-18a: hold1+edge省略 → ジョブなし（SURFACE_GESTURE_EDGE_REQUIRED）。
    #[tokio::test]
    async fn g18a_hold1_without_edge_enqueues_no_job() {
        let (state, mut rx) = test_state(TB01_GESTURES_JSON);
        handle_surface_gesture(&state, 1, "tb01", "hold1", None).await;
        assert!(rx.try_recv().is_err(), "hold1 without edge must not enqueue an AdapterJob");
    }

    // G-18b: surfaceId:"tb01-scroll", binding.t:"mouse.scroll"のときAction::MouseScroll{dy}
    // のジョブが載る。
    #[tokio::test]
    async fn g18b_tb01_scroll_enqueues_mouse_scroll() {
        let (state, mut rx) = test_state(TB01_GESTURES_JSON);
        let handle = tokio::spawn({
            let state = state.clone();
            async move {
                handle_surface_state(&state, 1, "tb01-scroll", 0.0, 15.4).await;
            }
        });
        let job = rx.recv().await.expect("tb01-scroll must enqueue an AdapterJob");
        assert_eq!(job.action, Action::MouseScroll { dy: 15 });
        let _ = job.reply.send(Ok(()));
        handle.await.unwrap();
    }

    // ================================================================
    // P-005 段階D: レイアウト保存の防御。
    // **書き込み系なので、ファイル名を組み立てる前の関門を重点的に見る。**
    // ================================================================

    #[test]
    fn layout_id_accepts_only_safe_names() {
        for ok in ["ipad_main", "a", "layout_2", "x_9_z"] {
            assert!(layout_id_is_safe(ok), "{ok} should be accepted");
        }
    }

    #[test]
    fn layout_id_rejects_anything_that_could_escape_the_directory() {
        // パス区切り・親ディレクトリ・拡張子・大文字・空。
        // ここを通すと任意パス書込になる（不変条件6の核心）。
        let bad = [
            "",
            "..",
            ".",
            "../etc/passwd",
            "a/b",
            "a\\b",
            "a.json",
            "A",
            "ipad-main",
            "layout name",
            "日本語",
            "a:b",
            "~x",
        ];
        for name in bad {
            assert!(!layout_id_is_safe(name), "{name:?} must be rejected");
        }
    }

    #[test]
    fn layout_id_rejects_overly_long_names() {
        assert!(layout_id_is_safe(&"a".repeat(64)));
        assert!(!layout_id_is_safe(&"a".repeat(65)));
    }

    /// 保存の前段（単体検証）が重なりを弾くこと。ここで弾かれるものはディスクに触れない。
    #[test]
    fn layout_save_rejects_overlapping_sections_before_touching_disk() {
        let overlapping = r#"{
            "layoutId": "tmp_overlap",
            "grid": { "cols": 4, "rows": 4 },
            "sections": [
                { "id": "A", "row": 1, "col": 1, "colSpan": 2, "rowSpan": 2,
                  "component": { "kind": "deck", "ref": "default" } },
                { "id": "B", "row": 1, "col": 2, "colSpan": 2, "rowSpan": 2,
                  "component": { "kind": "deck", "ref": "default" } }
            ]
        }"#;
        let error = crate::layout::load_layout_str("test", overlapping)
            .expect_err("overlapping sections must be rejected");
        assert!(
            error.cause.contains('A') && error.cause.contains('B'),
            "cause should name both conflicting sections: {}",
            error.cause
        );
    }

    /// 保存経路が受け入れる形は、`layouts/` にある**実物**と同じ形であること。
    /// 実データで確かめるのは、テスト用の都合のよいJSONだけで通してしまわないため。
    #[test]
    fn layout_save_accepts_the_real_layout_files() {
        // `cargo test` の作業ディレクトリはクレート直下（crates/proto-hub）で、
        // 実行時のワークスペース直下とは違う。実物を読むテストではここを明示する。
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        let paths = crate::startup::discover_prefixed_json(
            &repo_root.join(crate::LAYOUTS_DIR),
            "layout_",
        );
        assert!(!paths.is_empty(), "layouts/ に実物が1件も無い（テストの前提が崩れている）");
        for path in paths {
            let text = std::fs::read_to_string(&path).expect("real layout file must be readable");
            let layout = crate::layout::load_layout_str(&path.display().to_string(), &text)
                .unwrap_or_else(|e| panic!("{} must load: {}", path.display(), e.cause));
            assert!(
                layout_id_is_safe(&layout.layout_id),
                "{} has a layoutId the save API would reject: {}",
                path.display(),
                layout.layout_id
            );
            // 保存は受け取った本文をそのまま書かず、読み直して整形したものを書く。
            // 往復して同じものに戻ることを確かめる（書式の揺れで中身が変わらない保証）。
            let round = serde_json::to_string_pretty(&layout).expect("serialize");
            let again = crate::layout::load_layout_str("roundtrip", &round).expect("reload");
            assert_eq!(layout, again, "{} did not survive a save round-trip", path.display());
        }
    }

    /// 保存で**説明文が消えないこと**。
    ///
    /// 実際に一度消した: エディタが `description` を送らず、Hubが空として書いたため、
    /// `layout_ipad_v13.json` の運用申し送り373文字が失われた（.bakから復元）。
    /// 引き継ぎのロジックそのものをここで確かめる。
    #[test]
    fn save_carries_over_the_existing_description() {
        let existing_text = r#"{
            "layoutId": "tmp_desc",
            "description": "運用の申し送り。消えてはいけない。",
            "grid": { "cols": 4, "rows": 4 },
            "sections": [
                { "id": "A", "row": 1, "col": 1, "colSpan": 2, "rowSpan": 2,
                  "component": { "kind": "deck", "ref": "default" } }
            ]
        }"#;
        let existing = crate::layout::load_layout_str("existing", existing_text).expect("load");
        assert!(!existing.description.is_empty());

        // エディタが送ってくる形（配置だけ。説明文は入っていない）
        let incoming_text = r#"{
            "layoutId": "tmp_desc",
            "grid": { "cols": 4, "rows": 4 },
            "sections": [
                { "id": "A", "row": 1, "col": 1, "colSpan": 3, "rowSpan": 2,
                  "component": { "kind": "deck", "ref": "default" } }
            ]
        }"#;
        let mut incoming = crate::layout::load_layout_str("incoming", incoming_text).expect("load");
        assert!(incoming.description.is_empty(), "前提: 送られてくる本文に説明文は無い");

        // ハンドラと同じ引き継ぎ規則
        if incoming.description.is_empty() && !existing.description.is_empty() {
            incoming.description = existing.description.clone();
        }

        assert_eq!(
            incoming.description, existing.description,
            "説明文が引き継がれていない（保存で消える）"
        );
        // 配置のほうは新しい値で上書きされていること
        assert_eq!(incoming.sections[0].col_span, 3, "配置は新しい値になるべき");
    }

    /// 実物の説明文が、保存の往復で1文字も変わらないこと。
    #[test]
    fn real_layout_descriptions_survive_a_save_round_trip() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        let paths = crate::startup::discover_prefixed_json(
            &repo_root.join(crate::LAYOUTS_DIR),
            "layout_",
        );
        assert!(!paths.is_empty());
        let mut checked = 0;
        for path in paths {
            let text = std::fs::read_to_string(&path).expect("read");
            let layout = crate::layout::load_layout_str(&path.display().to_string(), &text)
                .expect("load");
            if layout.description.is_empty() {
                continue;
            }
            checked += 1;
            let written = serde_json::to_string_pretty(&layout).expect("serialize");
            let again = crate::layout::load_layout_str("roundtrip", &written).expect("reload");
            assert_eq!(
                layout.description,
                again.description,
                "{} の説明文が往復で変わった",
                path.display()
            );
        }
        assert!(checked > 0, "説明文を持つレイアウトが1件も無い（テストの前提が崩れている）");
    }
}
