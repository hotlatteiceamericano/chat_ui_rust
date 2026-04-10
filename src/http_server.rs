use chat_common::{auth_response::AuthResponse, login_response::LoginResponse};
use color_eyre::eyre::Context;

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

    pub async fn login(&self, user_email: &str) -> color_eyre::Result<LoginResponse> {
        let response = self
            .client
            .post(self.endpoint("login"))
            .json(&serde_json::json!({"email": user_email}))
            .send()
            .await
            .context("cannot make login request to http server")?
            .text()
            .await
            .context("cannot read login response body")?;

        serde_json::from_str::<LoginResponse>(&response)
            .context(format!("cannot deserialize login response: {}", response))
    }

    pub async fn auth(&self, user_email: &str, otp: &str) -> color_eyre::Result<AuthResponse> {
        let response_text = self
            .client
            .post(self.endpoint("auth"))
            .json(&serde_json::json!({"email": user_email, "otp": otp}))
            .send()
            .await
            .context("cannot make auth request to http server")?
            .text()
            .await
            .context("cannot read auth response body")?;

        serde_json::from_str::<AuthResponse>(&response_text).context(format!(
            "cannot deserialize auth response: {}",
            response_text
        ))
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.url, path)
    }
}
