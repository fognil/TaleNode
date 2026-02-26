use serde::{Deserialize, Serialize};

use super::operations::CollabOp;
use super::PeerInfo;

/// Messages exchanged over WebSocket between host and clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollabMessage {
    /// Full graph state sync (sent on connect).
    FullSync {
        graph_json: serde_json::Value,
        peers: Vec<PeerInfo>,
    },
    /// A single operation applied to the graph.
    Operation {
        sender: String,
        op: CollabOp,
        timestamp: u64,
    },
    /// Acknowledgement of an operation.
    Ack {
        op_index: u64,
    },
    /// A new peer has joined.
    PeerJoined(PeerInfo),
    /// A peer has left.
    PeerLeft {
        username: String,
    },
    /// Cursor/selection update from a peer.
    CursorUpdate {
        username: String,
        selected_nodes: Vec<uuid::Uuid>,
    },
    /// Client requests full sync from host.
    RequestSync {
        username: String,
    },
}

impl CollabMessage {
    /// Serialize to JSON string for WebSocket transport.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("Serialize error: {e}"))
    }

    /// Deserialize from JSON string.
    #[allow(dead_code)]
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| format!("Deserialize error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_peer_joined() {
        let msg = CollabMessage::PeerJoined(PeerInfo {
            username: "Bob".to_string(),
            color: [0, 255, 0],
            selected_nodes: vec![],
        });
        let json = msg.to_json().unwrap();
        let loaded = CollabMessage::from_json(&json).unwrap();
        if let CollabMessage::PeerJoined(p) = loaded {
            assert_eq!(p.username, "Bob");
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn roundtrip_ack() {
        let msg = CollabMessage::Ack { op_index: 42 };
        let json = msg.to_json().unwrap();
        let loaded = CollabMessage::from_json(&json).unwrap();
        if let CollabMessage::Ack { op_index } = loaded {
            assert_eq!(op_index, 42);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn roundtrip_operation() {
        let msg = CollabMessage::Operation {
            sender: "Alice".to_string(),
            op: CollabOp::MoveNode {
                node_id: uuid::Uuid::new_v4(),
                position: [100.0, 200.0],
            },
            timestamp: 1234567890,
        };
        let json = msg.to_json().unwrap();
        let loaded = CollabMessage::from_json(&json).unwrap();
        assert!(matches!(loaded, CollabMessage::Operation { .. }));
    }

    #[test]
    fn invalid_json_returns_error() {
        assert!(CollabMessage::from_json("not json").is_err());
    }

    #[test]
    fn roundtrip_full_sync() {
        let graph_json = serde_json::json!({
            "nodes": {},
            "connections": [],
            "variables": [],
            "characters": []
        });
        let msg = CollabMessage::FullSync {
            graph_json: graph_json.clone(),
            peers: vec![
                PeerInfo {
                    username: "Host".to_string(),
                    color: [255, 0, 0],
                    selected_nodes: vec![],
                },
                PeerInfo {
                    username: "Guest".to_string(),
                    color: [0, 255, 0],
                    selected_nodes: vec![uuid::Uuid::new_v4()],
                },
            ],
        };
        let json = msg.to_json().unwrap();
        let loaded = CollabMessage::from_json(&json).unwrap();
        if let CollabMessage::FullSync { peers, .. } = loaded {
            assert_eq!(peers.len(), 2);
            assert_eq!(peers[0].username, "Host");
            assert_eq!(peers[1].selected_nodes.len(), 1);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn roundtrip_cursor_update() {
        let ids = vec![uuid::Uuid::new_v4(), uuid::Uuid::new_v4()];
        let msg = CollabMessage::CursorUpdate {
            username: "Eve".to_string(),
            selected_nodes: ids.clone(),
        };
        let json = msg.to_json().unwrap();
        let loaded = CollabMessage::from_json(&json).unwrap();
        if let CollabMessage::CursorUpdate {
            username,
            selected_nodes,
        } = loaded
        {
            assert_eq!(username, "Eve");
            assert_eq!(selected_nodes, ids);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn roundtrip_request_sync() {
        let msg = CollabMessage::RequestSync {
            username: "NewPeer".to_string(),
        };
        let json = msg.to_json().unwrap();
        let loaded = CollabMessage::from_json(&json).unwrap();
        if let CollabMessage::RequestSync { username } = loaded {
            assert_eq!(username, "NewPeer");
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn roundtrip_peer_left() {
        let msg = CollabMessage::PeerLeft {
            username: "Quitter".to_string(),
        };
        let json = msg.to_json().unwrap();
        let loaded = CollabMessage::from_json(&json).unwrap();
        if let CollabMessage::PeerLeft { username } = loaded {
            assert_eq!(username, "Quitter");
        } else {
            panic!("Wrong variant");
        }
    }
}
