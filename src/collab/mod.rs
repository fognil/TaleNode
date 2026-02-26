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

    #[test]
    fn collab_state_peer_count() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let state = CollabState {
            mode: CollabMode::Hosting,
            peers: vec![
                PeerInfo {
                    username: "A".to_string(),
                    color: [0, 0, 0],
                    selected_nodes: vec![],
                },
                PeerInfo {
                    username: "B".to_string(),
                    color: [0, 0, 0],
                    selected_nodes: vec![],
                },
            ],
            local_username: "Host".to_string(),
            host_addr: "0.0.0.0:9847".to_string(),
            outgoing_tx: tx,
        };
        assert_eq!(state.peer_count(), 2);
    }

    #[test]
    fn collab_mode_serialization() {
        let modes = [CollabMode::Offline, CollabMode::Hosting, CollabMode::Joined];
        for mode in &modes {
            let json = serde_json::to_string(mode).unwrap();
            let loaded: CollabMode = serde_json::from_str(&json).unwrap();
            assert_eq!(*mode, loaded);
        }
    }

    #[test]
    fn server_client_handshake() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (result_tx, result_rx) = std::sync::mpsc::channel();
            let graph_json = serde_json::json!({"nodes":{},"connections":[]});

            // Start server on a random available port
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .unwrap();
            let port = listener.local_addr().unwrap().port();
            drop(listener); // Free the port for the server

            let srv_tx = result_tx.clone();
            let srv_handle = tokio::spawn(async move {
                let _ = server::run_server(port, graph_json, "Host".to_string(), srv_tx).await;
            });

            // Give server time to bind
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Connect client
            let (_out_tx, out_rx) = tokio::sync::mpsc::unbounded_channel();
            let cli_tx = result_tx.clone();
            let cli_handle = tokio::spawn(async move {
                let _ = client::run_client(
                    "127.0.0.1",
                    port,
                    "Guest".to_string(),
                    cli_tx,
                    out_rx,
                )
                .await;
            });

            // Wait for messages
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;

            // Collect all messages received
            let mut messages = Vec::new();
            while let Ok(msg) = result_rx.try_recv() {
                if let crate::app::async_runtime::AsyncResult::CollabMessage(s) = msg {
                    messages.push(s);
                }
            }

            // Should have at least the server listening message and connection
            assert!(
                messages.iter().any(|m| m.contains("listening")),
                "Expected server listening message, got: {:?}",
                messages
            );

            srv_handle.abort();
            cli_handle.abort();
        });
    }
}
