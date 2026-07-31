//! WSワイヤープロトコル（D6）。Hub→client: surface.config / layer.state / error の3種のみ。

use proto_keymap::{Edge, Keymap};
use serde::{Deserialize, Serialize};

use crate::deck::DeckSetlist;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "key.press")]
    KeyPress {
        #[serde(rename = "keyId")]
        key_id: String,
        edge: EdgeWire,
    },
    #[serde(rename = "deck.press")]
    DeckPress {
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
    pub momentary: Vec<u8>,
    pub toggled: Vec<u8>,
}

impl From<&proto_keymap::LayerState> for LayerStateWire {
    fn from(state: &proto_keymap::LayerState) -> Self {
        Self {
            momentary: state.momentary().iter().copied().collect(),
            toggled: state.toggled().iter().copied().collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SurfaceConfig<'a> {
    #[serde(rename = "activeKeymapId")]
    pub active_keymap_id: &'a str,
    pub keymap: &'a Keymap,
    pub layer: LayerStateWire,
    pub deck: &'a DeckSetlist,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage<'a> {
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
