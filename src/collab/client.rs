use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use super::protocol::CollabMessage;

/// Connect to a collaboration server as a client.
pub async fn run_client(
    host: &str,
    port: u16,
    username: String,
    result_tx: std::sync::mpsc::Sender<crate::app::async_runtime::AsyncResult>,
    mut outgoing_rx: tokio::sync::mpsc::UnboundedReceiver<Message>,
) -> Result<(), String> {
    let url = format!("ws://{host}:{port}");
    let (ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .map_err(|e| format!("Failed to connect to {url}: {e}"))?;

    let _ = result_tx.send(crate::app::async_runtime::AsyncResult::CollabMessage(
        format!("Connected to {url}"),
    ));

    let (mut write, mut read) = ws_stream.split();

    // Send RequestSync
    let sync_req = CollabMessage::RequestSync { username };
    if let Ok(json) = sync_req.to_json() {
        let _ = write.send(Message::text(json)).await;
    }

    // Read incoming messages in a separate task
    let read_tx = result_tx.clone();
    let read_handle = tokio::spawn(async move {
        while let Some(msg_result) = read.next().await {
            let msg = match msg_result {
                Ok(m) => m,
                Err(_) => break,
            };
            if let Message::Text(text) = msg {
                let _ = read_tx.send(
                    crate::app::async_runtime::AsyncResult::CollabMessage(
                        text.to_string(),
                    ),
                );
            }
        }
    });

    // Forward outgoing messages
    let write_handle = tokio::spawn(async move {
        while let Some(msg) = outgoing_rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = read_handle => {},
        _ = write_handle => {},
    }

    let _ = result_tx.send(crate::app::async_runtime::AsyncResult::CollabMessage(
        "Disconnected from server".to_string(),
    ));

    Ok(())
}
