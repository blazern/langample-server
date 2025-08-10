use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    http_client: Client,
    chatgpt_key: String,
}

impl AppState {
    pub fn new(chatgpt_key: String) -> Result<Self, reqwest::Error> {
        let http_client = Client::builder().timeout(Duration::from_secs(30)).build()?;
        Ok(Self {
            http_client,
            chatgpt_key,
        })
    }

    pub fn http_client(&self) -> &Client {
        &self.http_client
    }

    pub fn chatgpt_key(&self) -> &str {
        &self.chatgpt_key
    }
}
