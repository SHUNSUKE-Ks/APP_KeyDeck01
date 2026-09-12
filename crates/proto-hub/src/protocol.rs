//! WSワイヤープロトコル（D6）。Hub→client: surface.config / layer.state / error の3種のみ。

use proto_keymap::{Edge, Keymap};
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use crate::deck::DeckSetlist;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// P-005 段階B: `keymapId`を追加（省略時は面ごとの既定＝従来動作）。1画面に
    /// 複数のキーボード部品を置けるようになったため、どの盤面のキーかを名指しする必要がある。
    /// keymapIdもkeyIdと同じ「位置ID」なので不変条件1には抵触しない
    /// （実行内容を決めるのは相変わらずHub側のJSONだけ）。
    #[serde(rename = "key.press")]
    KeyPress {
        #[serde(rename = "keymapId", default)]
        keymap_id: Option<String>,
        #[serde(rename = "keyId")]
        key_id: String,
        edge: EdgeWire,
    },
    /// P-005 段階A: `deckId`を追加（省略時は`DEFAULT_DECK_ID`）。Deckが複数になったため
    /// slotIdだけでは一意に決まらない。deckIdもslotIdと同じ「位置ID」なので、
    /// 不変条件1（クライアントが送ってよいのは位置IDのみ）には抵触しない。
    #[serde(rename = "deck.press")]
    DeckPress {
        #[serde(rename = "deckId", default)]
        deck_id: Option<String>,
        #[serde(rename = "slotId")]
        slot_id: String,
    },
    /// T12（D28）: トラックボール等、連続値を出す面の共通メッセージ（§3.2）。
    /// `spin`/`active`は必須だがHubは使わない（読み捨てる。将来の3D面/パッド面のためのみ）。
    #[serde(rename = "surface.state")]
    SurfaceState {
        #[serde(rename = "surfaceId")]
        surface_id: String,
        delta: DeltaWire,
        #[allow(dead_code)]
        spin: SpinWire,
        #[allow(dead_code)]
        active: bool,
    },
    /// T18（brief/keydeck_trackball_gestures_v0.7.md §2.1）: discreteジェスチャー
    /// （タップ・ダブルタップ・長押し・Esc）。`edge`はhold系(hold1)のみ必須、
    /// one-shot系(tap1/dtap1/tap2/tap3)は省略する。
    #[serde(rename = "surface.gesture")]
    SurfaceGesture {
        #[serde(rename = "surfaceId")]
        surface_id: String,
        #[serde(rename = "gestureId")]
        gesture_id: String,
        #[serde(default)]
        edge: Option<EdgeWire>,
    },
}

/// §3.2: `delta.dx`/`dy`はf64で受け取り、Hub側（ws.rs）で丸め・クランプする。
#[derive(Debug, Deserialize)]
pub struct DeltaWire {
    pub dx: f64,
    pub dy: f64,
}

/// §3.2: 累積姿勢（クォータニオン）。今回Hubは読み捨てる（将来の3D面向け）。
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SpinWire {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeWire {
    Down,
    Up,
}

impl From<EdgeWire> for Edge {
    fn from(value: EdgeWire) -> Self {
        match value {
            EdgeWire::Down => Edge::Down,
            EdgeWire::Up => Edge::Up,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LayerStateWire {
    /// P-005 段階B: どの盤面のレイヤー状態かを名乗る。`/layout`面は複数のキーボードを
    /// 同時に描くため、これが無いとどれを塗り替えるべきか分からない。
    /// 既存面（kb/ipad/panel）はこの値を無視して従来どおり動く。
    #[serde(rename = "keymapId", skip_serializing_if = "Option::is_none")]
    pub keymap_id: Option<String>,
    pub momentary: Vec<u8>,
    pub toggled: Vec<u8>,
}

impl From<&proto_keymap::LayerState> for LayerStateWire {
    fn from(state: &proto_keymap::LayerState) -> Self {
        Self {
            keymap_id: None,
            momentary: state.momentary().iter().copied().collect(),
            toggled: state.toggled().iter().copied().collect(),
        }
    }
}

impl LayerStateWire {
    pub fn for_keymap(keymap_id: &str, state: &proto_keymap::LayerState) -> Self {
        Self { keymap_id: Some(keymap_id.to_string()), ..Self::from(state) }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SurfaceConfig<'a> {
    #[serde(rename = "activeKeymapId")]
    pub active_keymap_id: &'a str,
    pub keymap: &'a Keymap,
    pub layer: LayerStateWire,
    /// P-005 段階A: 全Deckを`deckId`をキーにして配る。クライアントはURL（`?deck=`）で
    /// どれを描くかを選ぶ。1本のWSで複数の部品を同時に描けるようにするため。
    pub decks: &'a BTreeMap<String, DeckSetlist>,
    /// P-005 段階B: `/layout`面が使う。全keymap・全レイアウト・盤面ごとのレイヤー状態。
    /// 既存面はこれらを見ない（従来の`keymap`/`layer`をそのまま使う）ので無影響。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keymaps: Option<&'a BTreeMap<String, Keymap>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layouts: Option<&'a BTreeMap<String, crate::layout::Layout>>,
    #[serde(rename = "layerStates", skip_serializing_if = "Option::is_none")]
    pub layer_states: Option<BTreeMap<String, LayerStateWire>>,
    /// 見た目のテーマ名。**Hubが1つだけ持ち、全部の端末へ同じものを配る。**
    /// 端末ごとに違う見た目にできると、どれが今の設定か分からなくなるため。
    pub theme: &'a str,
    /// 最初に出す board。決めていなければ付かない。
    #[serde(rename = "defaultLayout", skip_serializing_if = "Option::is_none")]
    pub default_layout: Option<&'a str>,
}

/// 表示するboardを切り替えろ、という知らせ。
/// `layoutId`が無ければ「最初に出すboard（既定）へ戻れ」。
#[derive(Debug, Clone, Serialize)]
pub struct LayoutSwitchWire {
    #[serde(rename = "layoutId", skip_serializing_if = "Option::is_none")]
    pub layout_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage<'a> {
    #[serde(rename = "layout.switch")]
    LayoutSwitch(LayoutSwitchWire),
    #[serde(rename = "surface.config")]
    SurfaceConfig(SurfaceConfig<'a>),
    #[serde(rename = "layer.state")]
    LayerState(LayerStateWire),
    #[serde(rename = "error")]
    Error {
        code: &'a str,
        cause: String,
        context: serde_json::Value,
    },
}
