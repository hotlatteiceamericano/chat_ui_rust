use chat_common::message::Message;
use crossterm::event::KeyEvent;

pub enum AppEvents {
    InboundMessage { message: Message },
    KeyEvent { key_event: KeyEvent },
}
