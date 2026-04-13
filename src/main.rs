use std::{
    io::{self, Write},
    sync::Arc,
};

use chat_common::message::Message;
use color_eyre::eyre::{Context, Result};
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

use crate::{app::App, app_event::AppEvents, http_server::HttpServer};

pub mod app;
pub mod app_event;
pub mod http_server;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let log_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".chat_ui_rust");
    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    tracing::info!("application starting");

    let http_server_url = std::env::var("HTTP_SERVER_URL").expect("HTTP server url required");

    color_eyre::install()?;

    print!("Enter your email: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let user_email = input.trim().to_string();

    let http_server = Arc::new(HttpServer::new(http_server_url));
    let login_response = http_server
        .login(&user_email)
        .await
        .expect("cannot make login request");

    print!("Enter the OTP: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let otp = input.trim().to_string();
    let auth_response = http_server
        .auth(&user_email, &otp)
        .await
        .expect("cannot make auth request");

    let (app_tx, app_rx) = mpsc::unbounded_channel::<AppEvents>();
    let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<Message>();
    let (ws_sender, ws_receiver) = connect_websocket(
        &auth_response.websocket_url,
        &login_response.user_id,
        &auth_response.jwt_token,
    )
    .await?;

    spawn_inbound_message_task(app_tx.clone(), ws_receiver);

    spawn_outbound_message_task(outbound_rx, ws_sender);

    spawn_terminal_task(app_tx.clone())?;
    let mut app = App::new(
        login_response.user_id,
        app_rx,
        app_tx.clone(),
        outbound_tx,
        http_server.clone(),
    );

    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal).await;

    ratatui::restore();

    if let Err(e) = &result {
        eprintln!("{:?}", e);
    }
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

async fn connect_websocket(
    websocket_server_url: &str,
    user_id: &str,
    jwt: &str,
) -> color_eyre::Result<(
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>,
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
)> {
    let mut request = websocket_server_url
        .into_client_request()
        .expect("failed to build request");
    let headers = request.headers_mut();
    headers.insert(
        "Authorization",
        format!("Bearer {}", jwt)
            .parse()
            .expect("failed to insert Authorization to request header"),
    );
    headers.insert("User-Id", user_id.parse()?);

    let (ws_stream, _response) = tokio_tungstenite::connect_async(request)
        .await
        .context("failed to connect to websocket server")?;

    Ok(ws_stream.split())
}
