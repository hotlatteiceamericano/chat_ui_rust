use chat_common::{message::Message, user::User};
use crossterm::event::KeyEvent;

pub enum AppEvents {
    InboundMessage { message: Message },
    KeyEvent { key_event: KeyEvent },
    UserLookupResult(Result<User, String>),
}
