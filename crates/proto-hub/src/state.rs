//! Hub状態の一元保持（D5/D6/D7/D8/D10）。

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use axum::extract::ws::Message;
use proto_keymap::{Action, Keymap, LayerState};
use subtle::ConstantTimeEq;
use tokio::sync::{mpsc, oneshot};

use crate::deck::DeckSetlist;

pub type ClientId = u64;
pub type SharedState = Arc<Mutex<HubState>>;

pub const PORT: u16 = 8770;

/// D8: 起動時生成・stdout1回表示・URLクエリ・定数時間比較（本線D4の簡略流用。有効期限は無し）。
#[derive(Debug, Clone)]
pub struct AccessToken {
    value: String,
}

impl AccessToken {
    pub fn generate() -> Self {
        use rand::RngExt;
        let mut rng = rand::rng();
        let bytes: [u8; 16] = rng.random();
        let mut hex = String::with_capacity(32);
        for byte in bytes {
            hex.push_str(&format!("{byte:02x}"));
        }
        Self { value: hex }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    /// 定数時間比較（D8）。文字列長が異なる場合はまず不一致だが、長さの違い自体は
    /// タイミング差として実用上の脅威にならない（LAN限定・トークンは固定長16進32文字のため）。
    pub fn is_valid(&self, candidate: &str) -> bool {
        if candidate.len() != self.value.len() {
            return false;
        }
        self.value.as_bytes().ct_eq(candidate.as_bytes()).into()
    }
}

/// proto_keymap::Actionを許可リストに載せる際の正規表現文字列（D5）。
/// Key/Chord（=OSへ到達しうるアクション）のみが対象。KeymapSwitch/KeymapReset・
/// レイヤー制御アクションはHub内部状態遷移でしかないため、この許可リストの対象外。
pub fn canonical_command_id(action: &Action) -> Option<String> {
    match action {
        Action::Key { vk } => Some(format!("key:{vk}")),
        Action::Chord { keys } => Some(format!("chord:{}", keys.join("+"))),
        _ => None,
    }
}

pub struct HubState {
    pub keymaps: BTreeMap<String, Keymap>,
    pub active_keymap_id: String,
    pub layer_state: LayerState,
    pub deck: DeckSetlist,
    /// D5: 起動時ロードしたJSON群に現れるKey/Chordアクションの集合のみ実行可。
    /// hub-core::CommandRegistryをそのまま再利用する（新規発明ゼロ）。requestId冪等や
    /// CommandService全体は今回の押下プロトコル（D6）にrequestIdが無いため使わず、
    /// 「許可リストに入っているか」だけを問うAPI(is_allowed)を借りる。
    pub command_registry: hub_core::CommandRegistry,
    pub clients: HashMap<ClientId, mpsc::UnboundedSender<Message>>,
    pub next_client_id: ClientId,
    pub token: AccessToken,
    pub adapter_tx: mpsc::UnboundedSender<AdapterJob>,
    /// D12: QRコード・ランディングページでURLを組み立てるために保持する。
    pub lan_ip: String,
}

impl HubState {
    /// D12: `target`（kb-left/kb-right/deck）から接続URLを組み立てる。tokenはHub内で
    /// 完結させ、クライアント側HTML/JSには一切埋め込まない。
    pub fn connection_url(&self, target: &str) -> Option<String> {
        let path = match target {
            "kb-left" => "/kb?half=left",
            "kb-right" => "/kb?half=right",
            "deck" => "/deck",
            _ => return None,
        };
        let separator = if path.contains('?') { '&' } else { '?' };
        Some(format!(
            "http://{}:{}{path}{separator}token={}",
            self.lan_ip,
            PORT,
            self.token.value()
        ))
    }
}

pub struct AdapterJob {
    pub action: Action,
    pub reply: oneshot::Sender<Result<(), proto_adapter_win::AdapterError>>,
}

/// D7: 「呼び出しは直列前提」。全クライアント・全アクションが単一のワーカーを通ることで、
/// 同時押下が来ても実OSへのSendInputは常にFIFOで直列実行される。
pub fn spawn_adapter_worker() -> mpsc::UnboundedSender<AdapterJob> {
    let (tx, mut rx) = mpsc::unbounded_channel::<AdapterJob>();
    tokio::spawn(async move {
        while let Some(job) = rx.recv().await {
            let action = job.action;
            let result = tokio::task::spawn_blocking(move || proto_adapter_win::send(&action))
                .await
                .unwrap_or_else(|join_error| {
                    Err(proto_adapter_win::AdapterError::Unsupported {
                        cause: format!("adapter worker task panicked: {join_error}"),
                    })
                });
            let _ = job.reply.send(result);
        }
    });
    tx
}
