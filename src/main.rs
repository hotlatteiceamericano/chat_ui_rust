use anyhow::Result;
use chat_websocket_service_rust::message::Message;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio_tungstenite::tungstenite::{
    self, client::IntoClientRequest, handshake::client::generate_key, http::Request,
};

use crate::app::App;

pub mod app;
pub mod user;

#[tokio::main]
async fn main() -> Result<()> {
    // todo: where should I ask for login id? from the first question waiting for user to enter?

    // create message sending channel
    // create message receiving channel
    let (sending_tx, sending_rx) = mpsc::unbounded_channel::<Message>();
    let (receiving_tx, receiving_rx) = mpsc::unbounded_channel::<Message>();

    spawn_websocket_connections(sending_rx).await;

    ratatui::run(|terminal| App::new(sending_tx).run(terminal))
}

async fn spawn_websocket_connections(mut sending_rx: UnboundedReceiver<Message>) {
    let uri = "ws://127.0.0.1:3000/ws";
    let mut request = uri.into_client_request().expect("failed to build request");
    let headers = request.headers_mut();
    headers.insert(
        "Authorization",
        "".parse().expect("failed to parse Authorization header"),
    );
    headers.insert(
        "User-Id",
        "0".parse().expect("failed to parse user id header"),
    );
    let (ws_stream, _response) = tokio_tungstenite::connect_async(request)
        .await
        .expect("not able to connect to websocket server");
    let (mut ws_sender, ws_receiver) = ws_stream.split();

    // spawn a task to wait for sending receiver (outbound)
    tokio::spawn(async move {
        // await for sending_rx
        while let Some(m) = sending_rx.recv().await {
            let json = serde_json::to_string(&m).expect("not able to serialize Message to JSON");
            ws_sender
                .send(tungstenite::Message::from(json))
                .await
                .expect("failed to send message to websocket server");
        }
    });
    // spawn another task to wait for the websocket receiver (inbound)
    // await for ws_receiver

    // lastly, have app to own the sending sender, and the receiving receiver
}
