use crate::streaming::Broadcast;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<crate::web_server::AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state.broadcast))
}

async fn handle_socket(socket: WebSocket, broadcast: Broadcast) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = broadcast.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&event) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    let mut recv_task =
        tokio::spawn(async move { while let Some(Ok(_)) = receiver.next().await {} });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
