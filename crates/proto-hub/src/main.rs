//! proto-hub — 基地局（T3）。axum/WS・状態一元保持・同期配信・エラー整形・切替・export。
//!
//! 設計書D5/D6/D8/D9/D10/D11。チェックポイントT3-1..T3-5はtracingログとサブモジュールの
//! 実装箇所コメントに残している（ws.rsを参照）。

mod deck;
mod error;
mod protocol;
mod qr;
mod state;
mod ws;

use std::collections::BTreeMap;
use std::net::SocketAddr;

use proto_keymap::{Action, Keymap};
use state::{canonical_command_id, AccessToken, HubState, IPAD_KEYMAP_ID, PORT};

const DEFAULT_KEYMAP_PATH: &str = "keymaps/keymap_default.json";
const WRITING01_KEYMAP_PATH: &str = "keymaps/keymap_writing01.json";
const IPAD01_VOL12_KEYMAP_PATH: &str = "keymaps/keymap_ipad01_vol12.json";
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
    let mut startup_errors: Vec<String> = Vec::new();
    let mut keymaps: BTreeMap<String, Keymap> = BTreeMap::new();

    for path in [DEFAULT_KEYMAP_PATH, WRITING01_KEYMAP_PATH, IPAD01_VOL12_KEYMAP_PATH] {
        match proto_keymap::load_keymap_from_path(path) {
            Ok(keymap) => {
                keymaps.insert(keymap.keymap_id.clone(), keymap);
            }
            Err(error) => startup_errors.push(format!("{path}: {error}")),
        }
    }

    // T8: ipad面は分割/Deckのactive系とは独立にIPAD_KEYMAP_IDへ固定される。
    // 起動時に必ずロードされている前提をここで検査する（既存のstartup_errors経路で拒否）。
    if startup_errors.is_empty() && !keymaps.contains_key(IPAD_KEYMAP_ID) {
        startup_errors.push(format!(
            "[{}] ipad surface requires keymapId '{IPAD_KEYMAP_ID}' to be loaded",
            error::LOAD_SCHEMA_INVALID
        ));
    }

    let deck = match deck::load_deck_from_path(DECK_PATH) {
        Ok(deck) => Some(deck),
        Err(error) => {
            startup_errors.push(format!("{DECK_PATH}: {error}"));
            None
        }
    };

    // KeymapSwitchの参照先keymapId存在チェック（両方が読み込めている場合のみ意味を持つ）。
    if let Some(deck) = &deck {
        if startup_errors.is_empty() {
            for action in all_actions(&keymaps, deck) {
                if let Action::KeymapSwitch { id } = action {
                    if !keymaps.contains_key(id) {
                        startup_errors.push(format!(
                            "[{}] keymap.switch references unknown keymapId '{id}'",
                            error::LOAD_SCHEMA_INVALID
                        ));
                    }
                }
            }
        }
    }

    if !startup_errors.is_empty() {
        eprintln!("proto-hub: startup rejected due to {} error(s):", startup_errors.len());
        for (index, message) in startup_errors.iter().enumerate() {
            eprintln!("  {}. {message}", index + 1);
        }
        std::process::exit(1);
    }

    let deck = deck.expect("deck load succeeded because startup_errors is empty");
    tracing::info!(
        chk = "T3-1",
        keymaps = keymaps.len(),
        decks = 1,
        "startup data loaded successfully"
    );

    // D5: 許可リスト構築（Key/Chordのみ。ロード済みJSON群全体から集める）。
    let command_ids: Vec<String> = all_actions(&keymaps, &deck)
        .filter_map(canonical_command_id)
        .collect();
    tracing::info!(chk = "T3-1", allowed_commands = command_ids.len(), "allow-list built");
    let command_registry = hub_core::CommandRegistry::new(command_ids);

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

    if let Err(error) = axum::serve(listener, router).await {
        eprintln!("proto-hub: server error: {error}");
        std::process::exit(1);
    }
}

fn all_actions<'a>(
    keymaps: &'a BTreeMap<String, Keymap>,
    deck: &'a deck::DeckSetlist,
) -> impl Iterator<Item = &'a Action> {
    keymaps
        .values()
        .flat_map(|keymap| keymap.layers.iter())
        .flat_map(|layer| layer.keys.values())
        .map(|key_def| &key_def.action)
        .chain(deck.actions())
}
