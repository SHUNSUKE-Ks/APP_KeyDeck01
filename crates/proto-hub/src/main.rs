//! proto-hub — 基地局（T3）。axum/WS・状態一元保持・同期配信・エラー整形・切替・export。
//!
//! 設計書D5/D6/D8/D9/D10/D11。チェックポイントT3-1..T3-5はtracingログとサブモジュールの
//! 実装箇所コメントに残している（ws.rsを参照）。

mod deck;
mod error;
mod protocol;
mod qr;
mod startup;
mod state;
mod ws;

use std::net::SocketAddr;
use std::path::Path;

use state::{AccessToken, HubState, PORT};

/// B1（設計書v0.5）: 固定3ファイルのハードコードを廃止し、このディレクトリ配下の
/// `keymap_*.json`を全てスキャン・ロードする（`startup::discover_keymap_paths`）。
/// 新フォーマットはここへファイルを置くだけで起動時に発見される。
const KEYMAPS_DIR: &str = "keymaps";
const DECK_PATH: &str = "decks/deck_default.json";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // [T3-1] 起動時ロード。失敗はD9のエラーコード＋causeを全件printして終了（起動拒否）。
    // B2の`/api/reload`（ws.rs）もこの同じ`startup::load_startup_data`を通るため、
    // 起動時とreload時で検証経路が1本に保たれる。
    let startup_data = match startup::load_startup_data(Path::new(KEYMAPS_DIR), Path::new(DECK_PATH)) {
        Ok(data) => data,
        Err(startup_errors) => {
            eprintln!("proto-hub: startup rejected due to {} error(s):", startup_errors.len());
            for (index, message) in startup_errors.iter().enumerate() {
                eprintln!("  {}. {message}", index + 1);
            }
            std::process::exit(1);
        }
    };
    let startup::StartupData {
        keymaps,
        deck,
        command_registry,
    } = startup_data;

    tracing::info!(
        chk = "T3-1",
        keymaps = keymaps.len(),
        decks = 1,
        "startup data loaded successfully"
    );
    tracing::info!(
        chk = "T3-1",
        allowed_commands = command_registry.allowed_command_ids().count(),
        "allow-list built"
    );

    let token = AccessToken::generate();
    let adapter_tx = state::spawn_adapter_worker();

    let lan_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|error| {
            tracing::warn!(cause = %error, "failed to determine LAN IP; falling back to 127.0.0.1");
            "127.0.0.1".to_string()
        });

    let active_keymap_id = "writing01".to_string();
    let hub_state = HubState::new(
        keymaps,
        active_keymap_id,
        deck,
        command_registry,
        token.clone(),
        adapter_tx,
        lan_ip.clone(),
    );
    let shared = std::sync::Arc::new(std::sync::Mutex::new(hub_state));

    let router = ws::router(shared);
    let listener = match tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], PORT))).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("proto-hub: failed to bind 0.0.0.0:{PORT}: {error}");
            std::process::exit(1);
        }
    };

    println!("proto-hub: listening on 0.0.0.0:{PORT}");
    println!("  QRつきランディングページ: http://{lan_ip}:{PORT}/  ← まずはこれを開く");
    println!("  keyboard (left) : http://{lan_ip}:{PORT}/kb?half=left&token={}", token.value());
    println!("  keyboard (right): http://{lan_ip}:{PORT}/kb?half=right&token={}", token.value());
    println!("  deck            : http://{lan_ip}:{PORT}/deck?token={}", token.value());
    println!("  ipad (Vol1.2)   : http://{lan_ip}:{PORT}/ipad?token={}", token.value());
    println!("  settings（再読込）: http://{lan_ip}:{PORT}/settings?token={}", token.value());

    if let Err(error) = axum::serve(listener, router).await {
        eprintln!("proto-hub: server error: {error}");
        std::process::exit(1);
    }
}
