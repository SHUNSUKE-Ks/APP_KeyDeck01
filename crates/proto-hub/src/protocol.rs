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
