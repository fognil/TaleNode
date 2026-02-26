use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message;

use super::protocol::CollabMessage;
use super::PeerInfo;

struct PeerConnection {
    tx: tokio::sync::mpsc::UnboundedSender<Message>,
    username: String,
}

/// Shared server state.
struct ServerState {
    peers: RwLock<HashMap<SocketAddr, PeerConnection>>,
    graph_json: RwLock<serde_json::Value>,
}

/// Start a WebSocket collaboration server.
pub async fn run_server(
    port: u16,
    initial_graph: serde_json::Value,
    _host_username: String,
    result_tx: std::sync::mpsc::Sender<crate::app::async_runtime::AsyncResult>,
) -> Result<(), String> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind {addr}: {e}"))?;

    let state = Arc::new(ServerState {
        peers: RwLock::new(HashMap::new()),
        graph_json: RwLock::new(initial_graph),
    });

    let _ = result_tx.send(crate::app::async_runtime::AsyncResult::CollabMessage(
        format!("Server listening on {addr}"),
    ));

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(v) => v,
            Err(_) => continue,
        };
        let st = Arc::clone(&state);
        let tx = result_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer_addr, st, tx).await {
                eprintln!("Connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<ServerState>,
    result_tx: std::sync::mpsc::Sender<crate::app::async_runtime::AsyncResult>,
) -> Result<(), String> {
    let ws = tokio_tungstenite::accept_async(stream)
        .await
        .map_err(|e| format!("WebSocket handshake failed: {e}"))?;

    let (mut write, mut read) = ws.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Register peer
    {
        let mut peers = state.peers.write().await;
        peers.insert(
            addr,
            PeerConnection {
                tx: tx.clone(),
                username: format!("peer_{}", addr.port()),
            },
        );
    }

    // Send full sync
    {
        let graph = state.graph_json.read().await;
        let peers_list = build_peer_list(&state).await;
        let sync_msg = CollabMessage::FullSync {
            graph_json: graph.clone(),
            peers: peers_list,
        };
        if let Ok(json) = sync_msg.to_json() {
            let _ = write.send(Message::text(json)).await;
        }
    }

    let _ = result_tx.send(crate::app::async_runtime::AsyncResult::CollabMessage(
        format!("Peer connected: {addr}"),
    ));

    // Forward outgoing messages in a separate task
    let write_handle = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Read and broadcast incoming messages
    while let Some(msg_result) = read.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(_) => break,
        };
        if let Message::Text(text) = msg {
            let text_str = text.to_string();
            broadcast_except(&state, addr, &text_str).await;
            let _ = result_tx.send(
                crate::app::async_runtime::AsyncResult::CollabMessage(text_str),
            );
        }
    }

    // Cleanup
    {
        let mut peers = state.peers.write().await;
        peers.remove(&addr);
    }

    let leave_msg = CollabMessage::PeerLeft {
        username: format!("peer_{}", addr.port()),
    };
    if let Ok(json) = leave_msg.to_json() {
        broadcast_all(&state, &json).await;
    }

    write_handle.abort();
    Ok(())
}

async fn build_peer_list(state: &ServerState) -> Vec<PeerInfo> {
    state
        .peers
        .read()
        .await
        .values()
        .map(|p| PeerInfo {
            username: p.username.clone(),
            color: [100, 149, 237],
            selected_nodes: vec![],
        })
        .collect()
}

async fn broadcast_except(state: &ServerState, skip: SocketAddr, text: &str) {
    let peers = state.peers.read().await;
    for (peer_addr, peer) in peers.iter() {
        if *peer_addr != skip {
            let _ = peer.tx.send(Message::text(text));
        }
    }
}

async fn broadcast_all(state: &ServerState, text: &str) {
    let peers = state.peers.read().await;
    for peer in peers.values() {
        let _ = peer.tx.send(Message::text(text));
    }
}
