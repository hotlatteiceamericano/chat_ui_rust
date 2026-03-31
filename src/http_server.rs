use chat_common::{auth_response::AuthResponse, login_response::LoginResponse};

pub struct HttpServer {
    url: String,
    client: reqwest::Client,
}

impl HttpServer {
    pub fn new(http_url: String) -> Self {
        Self {
            url: http_url,
            client: reqwest::Client::new(),
        }
    }

    pub fn login(&self, user_id: &str) -> LoginResponse {
        // making http request in sync
        LoginResponse {
            plain_otp: String::from(""),
        }
    }

    pub fn auth(&self, otp: &str) -> AuthResponse {
        AuthResponse {
            websocket_url: String::from(""),
            jwt_token: String::from(""),
        }
    }
}
