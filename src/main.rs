use anyhow::Result;
use chat_websocket_service_rust::message::Message;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::{self, client::IntoClientRequest};

use crate::app::App;

pub mod app;
pub mod user;

#[tokio::main]
async fn main() -> Result<()> {
    // todo: ask for user id from the input after cargo run

    let (sending_tx, sending_rx) = mpsc::unbounded_channel::<Message>();
    let (receiving_tx, receiving_rx) = mpsc::unbounded_channel::<Message>();

    spawn_websocket_connections(sending_rx, receiving_tx).await;

    ratatui::run(|terminal| App::new(sending_tx, receiving_rx).run(terminal))
}

async fn spawn_websocket_connections(
    mut sending_rx: UnboundedReceiver<Message>,
    receiving_tx: UnboundedSender<Message>,
) {
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
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

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
    tokio::spawn(async move {
        while let Some(Ok(m)) = ws_receiver.next().await {
            let _result = match m {
                // todo: move each arm into its own function
                tungstenite::Message::Text(text) => {
                    println!("received text messages from websocket server: {:?}", text);
                    let message: Message = serde_json::from_str(text.as_str())
                        .expect("cannot serialize messages received from the websocket server");
                    receiving_tx
                        .send(message)
                        .expect("receiving_tx failed to send message");
                }
                tungstenite::Message::Binary(_binary) => {
                    println!("received binary from websocket server");
                }
                _ => {
                    println!("received unsupported message from websocket");
                }
            };
        }
    });
}
