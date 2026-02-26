pub mod client;
pub mod operations;
pub mod protocol;
pub mod server;

use serde::{Deserialize, Serialize};

/// Current collaboration mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CollabMode {
    #[default]
    Offline,
    Hosting,
    Joined,
}

/// Info about a connected peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub username: String,
    pub color: [u8; 3],
    /// Node IDs currently selected by this peer.
    #[serde(default)]
    pub selected_nodes: Vec<uuid::Uuid>,
}

/// Top-level collaboration state.
#[derive(Debug, Clone)]
pub struct CollabState {
    pub mode: CollabMode,
    pub peers: Vec<PeerInfo>,
    pub local_username: String,
    pub host_addr: String,
    /// Channel sender for outgoing operations to the network task.
    #[allow(dead_code)]
    pub outgoing_tx: std::sync::mpsc::Sender<protocol::CollabMessage>,
}

impl CollabState {
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_offline() {
        assert_eq!(CollabMode::default(), CollabMode::Offline);
    }

    #[test]
    fn peer_info_serialization() {
        let p = PeerInfo {
            username: "Alice".to_string(),
            color: [255, 0, 0],
            selected_nodes: vec![],
        };
        let json = serde_json::to_string(&p).unwrap();
        let loaded: PeerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.username, "Alice");
        assert_eq!(loaded.color, [255, 0, 0]);
    }
}
