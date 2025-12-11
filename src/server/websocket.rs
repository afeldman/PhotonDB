//! WebSocket support for changefeeds

use axum::extract::ws::{Message, WebSocket};
use tracing::{error, info};

/// Handle WebSocket connection for changefeeds
pub async fn handle_changefeed(mut socket: WebSocket) {
    info!("New changefeed connection");

    // Send initial connection message
    if socket
        .send(Message::Text(r#"{"type":"connected"}"#.to_string()))
        .await
        .is_err()
    {
        error!("Failed to send connection message");
        return;
    }

    // Handle incoming messages
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!(message = %text, "Received changefeed subscription");
                // TODO: Subscribe to changefeed
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!(error = %e, "WebSocket error");
                break;
            }
            _ => {}
        }
    }
}
