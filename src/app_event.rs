use chat_websocket_service_rust::message::Message;

pub enum AppEvents {
    InboundMessage { message: Message },
    Terminal,
}
