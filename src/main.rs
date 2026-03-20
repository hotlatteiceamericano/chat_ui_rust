use std::io::{self, Write};

use chat_common::message::Message;
use color_eyre::eyre::Result;
use crossterm::event::{Event, EventStream};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::{self, client::IntoClientRequest},
};

use crate::{app::App, app_event::AppEvents};

pub mod app;
pub mod app_event;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    print!("Enter your user ID: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let current_user_id: u32 = input.trim().parse().expect("invalid user ID");

    let mut terminal = ratatui::init();

    let (app_tx, app_rx) = mpsc::unbounded_channel::<AppEvents>();
    let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<Message>();
    let (ws_sender, ws_receiver) = connect_websocket().await;

    spawn_inbound_message_task(app_tx.clone(), ws_receiver);

    spawn_outbound_message_task(outbound_rx, ws_sender);

    spawn_terminal_task(app_tx.clone())?;
    let mut app = App::new(current_user_id, app_rx, outbound_tx);
    let result = app.run(&mut terminal).await;

    ratatui::restore();
    result
}

fn spawn_inbound_message_task(
    app_tx: UnboundedSender<AppEvents>,
    mut ws_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) {
    tokio::spawn(async move {
        while let Some(Ok(m)) = ws_receiver.next().await {
            let _result = match m {
                // todo: move each arm into its own function
                tungstenite::Message::Text(text) => {
                    let message: Message = serde_json::from_str(text.as_str())
                        .expect("cannot serialize messages received from the websocket server");
                    app_tx
                        .send(AppEvents::InboundMessage { message })
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

fn spawn_outbound_message_task(
    mut outbound_rx: UnboundedReceiver<Message>,
    mut ws_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>,
) {
    tokio::spawn(async move {
        // await for sending_rx
        while let Some(m) = outbound_rx.recv().await {
            let json = serde_json::to_string(&m).expect("not able to serialize Message to JSON");
            ws_sender
                .send(tungstenite::Message::from(json))
                .await
                .expect("failed to send message to websocket server");
        }
    });
}

fn spawn_terminal_task(app_tx: UnboundedSender<AppEvents>) -> Result<()> {
    let mut reader = EventStream::new();
    tokio::spawn(async move {
        loop {
            match reader.next().await {
                Some(Ok(Event::Key(key_event))) => {
                    app_tx
                        .send(AppEvents::KeyEvent { key_event })
                        .expect("cannot send terminal events to application");
                }

                Some(Ok(_)) => {}
                Some(Err(_)) => {}
                None => {}
            }
        }
    });
    Ok(())
}

async fn connect_websocket() -> (
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>,
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
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

    ws_stream.split()
}
