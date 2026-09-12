//! Hub状態の一元保持（D5/D6/D7/D8/D10）。

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use axum::extract::ws::Message;
use proto_keymap::{Action, Keymap, LayerState};
use subtle::ConstantTimeEq;
use tokio::sync::{mpsc, oneshot};

use crate::deck::DeckSetlist;

pub type ClientId = u64;
pub type SharedState = Arc<Mutex<HubState>>;

pub const PORT: u16 = 8770;

/// T8: ipad面は分割(kb-left/kb-right)＋Deckの共有状態(active_keymap_id/layer_state)とは
/// 独立に、常にこのkeymapIdへ固定する（"ipad面はkeymap ipad01_vol12固定でよい（splitの
/// active系とは独立）"）。keymap.switch/keymap.resetの影響も受けない。
pub const IPAD_KEYMAP_ID: &str = "ipad01_vol12";

/// P-005 段階A: `deck.press`が`deckId`を省略したとき、および`/deck`・`/panel`が
/// 指定なしで開かれたときに使うDeck。起動時に存在を検証する（startup.rs）。
pub const DEFAULT_DECK_ID: &str = "default";

/// WS接続がどの面かを表す。Split=分割キーボード/Deck（従来どおり共有state.active_keymap_id・
/// layer_stateを使う）、Ipad=iPad一枚キーボード（IPAD_KEYMAP_ID固定・独立したlayer_state）、
/// Trackball=T12トラックボール面（D28。keymap/layerを持たず、surface.stateのみを扱う）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Split,
    Ipad,
    Trackball,
    /// P-005 段階B: `/layout` 面。1画面に複数の部品（keyboard/deck/trackball）を並べる。
    /// レイヤー状態はkeymapIdごとに持つ（`HubState::layer_states`）ため、この面は
    /// SplitともIpadとも状態を共有しない。
    Layout,
}

impl SurfaceKind {
    pub fn from_query(value: Option<&str>) -> Self {
        match value {
            Some("ipad") => SurfaceKind::Ipad,
            Some("trackball") => SurfaceKind::Trackball,
            Some("layout") => SurfaceKind::Layout,
            _ => SurfaceKind::Split,
        }
    }
}

struct ClientEntry {
    tx: mpsc::UnboundedSender<Message>,
    surface: SurfaceKind,
}

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
/// Key/Chord/Text（=OSへ到達しうるアクション）のみが対象。KeymapSwitch/KeymapReset・
/// レイヤー制御アクションはHub内部状態遷移でしかないため、この許可リストの対象外。
/// D20: textは`text:<string>`形式でcanonical化する（ロード済みJSON由来の文字列のみが
/// 許可リストに載るため、任意文字列を受け付けるAPIにはならない）。
fn button_id(button: &proto_keymap::MouseButtonKind) -> &'static str {
    match button {
        proto_keymap::MouseButtonKind::Left => "left",
        proto_keymap::MouseButtonKind::Right => "right",
    }
}

pub fn canonical_command_id(action: &Action) -> Option<String> {
    match action {
        Action::Key { vk } => Some(format!("key:{vk}")),
        // キーマップからマウスのクリックを撃てるようにしたぶん、**許可リストの
        // 対象にもする**（D5）。起動時に読み込んだキーマップに書かれているものだけが
        // 実行できる、という防御をキーと同じ強さで掛けるため。
        // トラックボール面のジェスチャーは別経路（handle_surface_gesture）で
        // surfaces/*.json の検証を通っており、ここは通らない。
        Action::MouseClick { button } => Some(format!("mouse.click:{}", button_id(button))),
        Action::MouseDoubleClick { button } => {
            Some(format!("mouse.dblclick:{}", button_id(button)))
        }
        Action::Chord { keys } => Some(format!("chord:{}", keys.join("+"))),
        Action::Text { string } => Some(format!("text:{string}")),
        // T21: tg.fire自体はOSへ届かないが、**中の`fire`は届く**。ここで潜らないと
        // 起動時の許可リストに内側のアクションが載らず、発火が実行時に
        // 「absent from the startup allow-list」で弾かれる（実際に一度そうなった。
        // 症状は「画面表示だけ切り替わり、PCのIMEが切り替わらない」）。
        // 入れ子は1段だけ（ロード時にfire=key/chord/textへ制限済み）。
        Action::TgFire { fire, .. } => canonical_command_id(fire),
        // P-005 段階C: JSONに書くのはKeyHold、実際に発火するのはKeyButton。
        // **両方が同じidになるようにする**（片方だけだと実行時に許可リストで弾かれる。
        // T21のtg.fireで実際に踏んだ罠と同じ形）。
        Action::KeyHold { vk } | Action::KeyButton { vk, .. } => Some(format!("key.hold:{vk}")),
        _ => None,
    }
}

pub struct HubState {
    pub keymaps: BTreeMap<String, Keymap>,
    pub active_keymap_id: String,
    pub layer_state: LayerState,
    /// T8: ipad面専用のレイヤー状態。IPAD_KEYMAP_IDに対してのみ使う。分割/Deck側の
    /// layer_stateとは独立（keymap.switch/keymap.resetの影響を受けない）。
    pub ipad_layer_state: LayerState,
    /// P-005 段階A: Deckは複数持つ（キーは`deckId`）。`DEFAULT_DECK_ID`は必ず存在する。
    pub decks: BTreeMap<String, DeckSetlist>,
    /// P-005 段階B: 画面の区画割り（キーは`layoutId`）。0件でも起動する。
    pub layouts: BTreeMap<String, crate::layout::Layout>,
    /// P-005 段階B: **keymapIdごと**のレイヤー状態。1画面に複数のキーボード部品を置ける
    /// ようになったため、面ごと（layer_state/ipad_layer_state）では足りない。
    /// 同じkeymapを2区画に置いたら状態は共有される＝同じキーボードなら同じレイヤー、が正しい。
    pub layer_states: BTreeMap<String, LayerState>,
    /// D5: 起動時ロードしたJSON群に現れるKey/Chord/Textアクションの集合のみ実行可。
    /// hub-core::CommandRegistryをそのまま再利用する（新規発明ゼロ）。requestId冪等や
    /// CommandService全体は今回の押下プロトコル（D6）にrequestIdが無いため使わず、
    /// 「許可リストに入っているか」だけを問うAPI(is_allowed)を借りる。
    pub command_registry: hub_core::CommandRegistry,
    /// T11（D28）: `surfaces/trackball.json`から構築したレジストリ。T12でsurface.state
    /// 受信時にsurfaceIdを引くために使う。
    pub surfaces: crate::surface::SurfaceRegistry,
    /// P-005 段階C: いま押しっぱなしになっているキー（クライアント別）。
    /// これが無いと、十字キーを押したまま切断・画面を閉じる・電波が切れる、で
    /// **キーが押されっぱなしになりPCが操作不能になる**。切断時にここを見て全部離す。
    held_keys: HashMap<ClientId, BTreeSet<String>>,
    clients: HashMap<ClientId, ClientEntry>,
    pub next_client_id: ClientId,
    pub token: AccessToken,
    pub adapter_tx: mpsc::UnboundedSender<AdapterJob>,
    /// D12: QRコード・ランディングページでURLを組み立てるために保持する。
    pub lan_ip: String,
    /// このPCが持っているIPv4アドレス全部（インターフェース名, アドレス）。
    ///
    /// **1つでは足りない。** 家のWiFiに繋いだままPCをアクセスポイントにすると、
    /// PCは2つのアドレスを持つ。QRに載るのは既定の経路の側（家のWiFi）なので、
    /// アクセスポイント側に繋いだiPadからは開けない。だから全部を持っておき、
    /// **この一覧に載っているものだけ**へ切り替えられるようにする。
    /// 一覧外を受け取らないので、外から任意のホストを差し込むことはできない。
    pub lan_ips: Vec<(String, String)>,
}

impl HubState {
    /// `clients`はモジュール内部でのみ構築する（外部からsurface無しで挿し込めないようにする
    /// ため）。main.rsは起動時にこのコンストラクタを通して初期状態を組み立てる。
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        keymaps: BTreeMap<String, Keymap>,
        active_keymap_id: String,
        decks: BTreeMap<String, DeckSetlist>,
        command_registry: hub_core::CommandRegistry,
        surfaces: crate::surface::SurfaceRegistry,
        layouts: BTreeMap<String, crate::layout::Layout>,
        token: AccessToken,
        adapter_tx: mpsc::UnboundedSender<AdapterJob>,
        lan_ip: String,
    ) -> Self {
        Self {
            keymaps,
            active_keymap_id,
            layer_state: LayerState::new(),
            ipad_layer_state: LayerState::new(),
            decks,
            layouts,
            layer_states: BTreeMap::new(),
            command_registry,
            surfaces,
            held_keys: HashMap::new(),
            clients: HashMap::new(),
            next_client_id: 0,
            token,
            adapter_tx,
            lan_ips: vec![("この端末".to_string(), lan_ip.clone())],
            lan_ip,
        }
    }

    /// 起動時に見つけたIPv4を登録する。既定は`lan_ip`のまま変えない。
    pub fn set_hosts(&mut self, hosts: Vec<(String, String)>) {
        if !hosts.is_empty() {
            self.lan_ips = hosts;
        }
    }

    /// QRとURLの組み立て先を切り替える。**一覧に無いアドレスは受け付けない。**
    /// 受け付けたらtrue。
    pub fn set_lan_ip(&mut self, ip: &str) -> bool {
        if self.lan_ips.iter().any(|(_, known)| known == ip) {
            self.lan_ip = ip.to_string();
            return true;
        }
        false
    }

    /// P-005 段階B: keymapIdごとのレイヤー状態を取り出す（無ければ作る）。
    pub fn layer_state_for(&mut self, keymap_id: &str) -> &mut LayerState {
        self.layer_states.entry(keymap_id.to_string()).or_default()
    }

    /// P-005 段階A: `(deckId, slotId)`でスロットを引く。deckIdはクライアントが送る
    /// 「位置ID」の一部であり、実行内容を指定するものではない（不変条件1）。
    pub fn find_deck_slot(&self, deck_id: &str, slot_id: &str) -> Option<&crate::deck::Slot> {
        self.decks.get(deck_id)?.find_slot(slot_id)
    }

    /// D12/D25: `target`（kb-left/kb-right/deck/ipad/trackball/panel）から接続URLを組み立てる。
    /// tokenはHub内で完結させ、クライアント側HTML/JSには一切埋め込まない。
    pub fn connection_url(&self, target: &str) -> Option<String> {
        // P-005: `layout:<layoutId>` でレイアウトごとのURLを作れるようにする。
        // レイアウトが増えても、iPad側は「ランディングページのQRを読む」だけで
        // 目的の画面に飛べる（URLを手で打たなくてよい）。
        if let Some(layout_id) = target.strip_prefix("layout:") {
            if !self.layouts.contains_key(layout_id) {
                return None;
            }
            return Some(format!(
                "http://{}:{}/layout?id={layout_id}&token={}",
                self.lan_ip,
                PORT,
                self.token.value()
            ));
        }
        let path = match target {
            "kb-left" => "/kb?half=left",
            "kb-right" => "/kb?half=right",
            "deck" => "/deck",
            "ipad" => "/ipad",
            "trackball" => "/trackball",
            // T20（P-003 Ver1-a）: 分割面。WSは`surface=ipad`を使うがURLは独立。
            "panel" => "/panel",
            // P-005 段階B: レイアウト面。どのレイアウトを開くかは ?id= で選ぶ。
            "layout" => "/layout",
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

    pub fn register_client(
        &mut self,
        client_id: ClientId,
        tx: mpsc::UnboundedSender<Message>,
        surface: SurfaceKind,
    ) {
        self.clients.insert(client_id, ClientEntry { tx, surface });
    }

    /// P-005 段階C: 押下/解放を台帳に反映する。戻り値は「実際に状態が変わったか」。
    pub fn note_key_hold(&mut self, client_id: ClientId, vk: &str, down: bool) {
        let entry = self.held_keys.entry(client_id).or_default();
        if down {
            entry.insert(vk.to_string());
        } else {
            entry.remove(vk);
        }
        if entry.is_empty() {
            self.held_keys.remove(&client_id);
        }
    }

    /// P-005 段階C: そのクライアントが押しっぱなしにしているキーを取り出して台帳から消す。
    /// 切断時に呼び、返ってきたvkを全部releaseする。
    pub fn take_held_keys(&mut self, client_id: ClientId) -> Vec<String> {
        self.held_keys
            .remove(&client_id)
            .map(|set| set.into_iter().collect())
            .unwrap_or_default()
    }

    pub fn unregister_client(&mut self, client_id: ClientId) {
        self.clients.remove(&client_id);
    }

    pub fn send_to(&self, client_id: ClientId, message: Message) {
        if let Some(entry) = self.clients.get(&client_id) {
            let _ = entry.tx.send(message);
        }
    }

    /// `surface`が一致するクライアント全員へ配信する（分割/Deckとipadを混線させないため）。
    /// Messageはクライアントごとに新規生成する（Message自体をCloneに依存させないため）。
    pub fn broadcast_to(&self, surface: SurfaceKind, text: &str) {
        for entry in self.clients.values() {
            if entry.surface == surface {
                let _ = entry.tx.send(Message::Text(text.to_string().into()));
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> HubState {
        let deck = crate::deck::load_deck_str(
            "test",
            r#"{ "deckId": "default", "grid": { "cols": 1, "rows": 1 }, "pages": [] }"#,
        )
        .expect("empty deck must load");
        let mut decks = BTreeMap::new();
        decks.insert(deck.deck_id.clone(), deck);
        let surfaces = crate::surface::load_surface_registry_str("test", r#"{ "surfaces": [] }"#)
            .expect("empty surface registry must load");
        let (tx, _rx) = mpsc::unbounded_channel::<AdapterJob>();
        HubState::new(
            BTreeMap::new(),
            "none".to_string(),
            decks,
            hub_core::CommandRegistry::new(Vec::<String>::new()),
            surfaces,
            BTreeMap::new(),
            AccessToken::generate(),
            tx,
            "192.168.0.5".to_string(),
        )
    }

    /// T20（P-003 Ver1-a）: 分割面のQR/ランディングページ用targetが解決できること。
    /// ここが欠けると `/api/qr?target=panel` が400を返し、panel.html内のQRモーダルが
    /// 画像切れになる（ブラウザ上はエラーにならず気付きにくいためテストで固定する）。
    #[test]
    fn connection_url_resolves_panel_target() {
        let state = test_state();
        let url = state.connection_url("panel").expect("panel must be a known target");
        assert!(url.starts_with("http://192.168.0.5:8770/panel?token="), "unexpected url: {url}");
    }

    /// ランディングページ（ws.rs::index_page）が並べる全targetが解決できること。
    /// 片方だけ足して片方を忘れる事故を防ぐ。
    #[test]
    fn connection_url_resolves_every_landing_page_target() {
        let state = test_state();
        for target in ["kb-left", "kb-right", "deck", "ipad", "trackball", "panel"] {
            assert!(
                state.connection_url(target).is_some(),
                "landing page target '{target}' must resolve to a URL"
            );
        }
    }

    /// T21の回帰テスト: tg.fireの中のアクションが起動時許可リスト（D5）に載ること。
    /// ここが抜けると、レイヤーは切り替わるのに発火だけがWS実行時に拒否され、
    /// 「画面だけ切り替わってPCのIMEが変わらない」という分かりにくい壊れ方をする。
    #[test]
    fn canonical_command_id_descends_into_tg_fire() {
        let action = Action::TgFire {
            layer: 3,
            fire: Box::new(Action::Chord {
                keys: vec!["ALT".to_string(), "GRAVE".to_string()],
            }),
        };
        assert_eq!(canonical_command_id(&action).as_deref(), Some("chord:ALT+GRAVE"));
    }

    #[test]
    fn connection_url_rejects_unknown_target() {
        let state = test_state();
        assert_eq!(state.connection_url("does-not-exist"), None);
    }
}
