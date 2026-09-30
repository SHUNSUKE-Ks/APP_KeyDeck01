//! proto-hub — 基地局（T3）。axum/WS・状態一元保持・同期配信・エラー整形・切替・export。
//!
//! 設計書D5/D6/D8/D9/D10/D11。チェックポイントT3-1..T3-5はtracingログとサブモジュールの
//! 実装箇所コメントに残している（ws.rsを参照）。

mod app_launch;
mod deck;
mod device;
mod layout;
mod error;
mod icon;
mod protocol;
mod qr;
mod startup;
mod state;
mod surface;
mod ws;

use std::net::SocketAddr;
use std::path::Path;

use state::{AccessToken, HubState, PORT};

/// B1（設計書v0.5）: 固定3ファイルのハードコードを廃止し、このディレクトリ配下の
/// `keymap_*.json`を全てスキャン・ロードする（`startup::discover_keymap_paths`）。
/// 新フォーマットはここへファイルを置くだけで起動時に発見される。
const KEYMAPS_DIR: &str = "keymaps";
/// P-005 段階A: Deckは1枚固定をやめ、`keymaps/`と同じディレクトリスキャンにした。
const DECKS_DIR: &str = "decks";
/// T11（D28）: `surfaces/trackball.json`を置くディレクトリ。存在しなくても起動は成功する
/// （startup::load_startup_dataのT11-4）。
const SURFACES_DIR: &str = "surfaces";

/// P-005 段階B: 画面の区画割り（`layouts/layout_*.json`）。無くても起動する。
const LAYOUTS_DIR: &str = "layouts";

/// 起動してよいアプリの許可リスト（`apps/apps.json`）。無くても起動する
/// （その場合は起動できるアプリが0件になるだけ）。
const APPS_DIR: &str = "apps";

/// P-008: 端末スロット（`devices/devices.json`）。無ければ端末を区別しない（従来どおり）。
/// 起動時にだけ読む。書き換えたら Hub を再起動する。
const DEVICES_DIR: &str = "devices";

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
    let startup_data = match startup::load_startup_data(
        Path::new(KEYMAPS_DIR),
        Path::new(DECKS_DIR),
        Path::new(SURFACES_DIR),
        Path::new(LAYOUTS_DIR),
        Path::new(APPS_DIR),
    ) {
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
        decks,
        command_registry,
        surfaces,
        layouts,
        apps,
    } = startup_data;

    tracing::info!(
        chk = "T3-1",
        keymaps = keymaps.len(),
        decks = decks.len(),
        surfaces = surfaces.len(),
        layouts = layouts.len(),
        apps = apps.len(),
        "startup data loaded successfully"
    );
    tracing::info!(
        chk = "T3-1",
        allowed_commands = command_registry.allowed_command_ids().count(),
        "allow-list built"
    );

    // P-008: 端末スロット。defaultLayout の実在確認に盤面が要るので、盤面を読んだ後に読む
    let devices = match device::load_devices(Path::new(DEVICES_DIR), &layouts) {
        Ok(devices) => devices,
        Err(error) => {
            eprintln!("proto-hub: startup rejected due to 1 error(s):");
            eprintln!("  1. {error}");
            std::process::exit(1);
        }
    };
    tracing::info!(chk = "P008", devices = devices.len(), "device slots loaded");

    let token = AccessToken::generate();
    let adapter_tx = state::spawn_adapter_worker();

    let lan_ip = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|error| {
            tracing::warn!(cause = %error, "failed to determine LAN IP; falling back to 127.0.0.1");
            "127.0.0.1".to_string()
        });

    // このPCが持つIPv4を全部拾う。ループバックとリンクローカル(169.254)は
    // 端末から繋げないので外す。既定の経路のものを先頭に置く。
    let mut hosts: Vec<(String, String)> = local_ip_address::list_afinet_netifas()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(name, ip)| match ip {
            std::net::IpAddr::V4(v4)
                if !v4.is_loopback() && !v4.is_link_local() && !v4.is_unspecified() =>
            {
                Some((name, v4.to_string()))
            }
            _ => None,
        })
        .collect();
    hosts.sort_by_key(|(_, ip)| (*ip != lan_ip, ip.clone()));

    let active_keymap_id = "writing01".to_string();
    let mut hub_state = HubState::new(
        keymaps,
        active_keymap_id,
        decks,
        command_registry,
        surfaces,
        layouts,
        apps,
        token.clone(),
        adapter_tx,
        lan_ip.clone(),
    );
    hub_state.set_hosts(hosts.clone());
    hub_state.set_devices(devices);
    let shared = std::sync::Arc::new(std::sync::Mutex::new(hub_state));

    // 起動バナー用に1本持っておく（routerへはこの後moveされるため）
    let banner_state = std::sync::Arc::clone(&shared);
    let router = ws::router(shared);
    let listener = match tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], PORT))).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("proto-hub: failed to bind 0.0.0.0:{PORT}: {error}");
            std::process::exit(1);
        }
    };

    println!("proto-hub: listening on 0.0.0.0:{PORT}");
    println!("  ▼ PCで開く");
    println!("    トップ（統合編集）: http://{lan_ip}:{PORT}/?token={}", token.value());
    println!("    QRギャラリー（端末を繋ぐ）: http://{lan_ip}:{PORT}/connect?token={}", token.value());
    println!("    設定（構成の確認・再読込）: http://{lan_ip}:{PORT}/settings?token={}", token.value());
    if hosts.len() > 1 {
        println!("  ▼ このPCの他のアドレス（外出先でアクセスポイントにしたときはこちら）");
        for (name, ip) in hosts.iter().filter(|(_, ip)| *ip != lan_ip) {
            println!("    {name}: http://{ip}:{PORT}/connect?token={}", token.value());
        }
        println!("    （QRギャラリー右上の「つなぎ先」で切り替えると、QRもこちらになります）");
    }
    // P-008: 端末スロットがあれば、端末ごとの入口を先に出す（その端末専用の URL）
    {
        let s = banner_state.lock().unwrap();
        if !s.devices.is_empty() {
            println!("  ▼ 端末ごとの入口（devices/devices.json。盤面の切り替えはその端末だけに届く）");
            for slot in s.devices.iter() {
                if let Some(url) = s.connection_url(&format!("device:{}", slot.id)) {
                    println!("    {}: {url}", slot.label);
                }
            }
        }
    }
    println!("  ▼ 端末で開く（QRギャラリーから読み取るのが早い）");
    // T9: 一覧は手書きしない。`connection_targets()`（ws.rs）が唯一の表で、
    // ランディングページ・/api/formats・ここが同じものを見る。
    // 以前はここだけ手書きの7行で、P-005で足したレイアウト3種が出ていなかった。
    {
        let s = banner_state.lock().unwrap();
        for (target, label, _kind) in ws::connection_targets(&s) {
            if let Some(url) = s.connection_url(&target) {
                println!("    {label}: {url}");
            }
        }
    }


    // ================================================================
    // 起動したらブラウザを開く。
    //
    // トークンは起動のたびに変わる（D8）ため、URLを手で打つ運用が成立しない。
    // 以前はトップがQR一覧でトークン不要だったが、トップを編集画面にした結果、
    // **トークン無しで入れる入口が無くなった**（コンソールからURLを写すしかない）。
    // 開く側をこちらが用意すれば、写す作業ごと無くなる。
    //
    // トークンをページに埋め込むのではなく、URLとして渡すだけなので
    // 認証の強さは変わらない。開いてほしくないときは KEYDECK_NO_OPEN=1。
    if std::env::var_os("KEYDECK_NO_OPEN").is_none() {
        let url = format!("http://localhost:{PORT}/?token={}", token.value());
        // `start` は cmd の内部コマンドなので cmd 経由で呼ぶ。
        // 第1引数の "" はウィンドウタイトル（URLがタイトルと誤解されるのを防ぐ）。
        match std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
        {
            Ok(_) => println!("  （ブラウザを開きました。開かない場合は上のURLをコピーしてください）"),
            // 開けなくても起動は続ける。URLは上に出ているので手で開けば済む
            Err(error) => tracing::warn!(cause = %error, "failed to open the browser"),
        }
    }

    if let Err(error) = axum::serve(listener, router).await {
        eprintln!("proto-hub: server error: {error}");
        std::process::exit(1);
    }
}
