use reqwest::Client;
use std::error::Error;

use crate::constants;

pub struct ExchangeManager {
    api_url: String,
    client: Client,
}

impl ExchangeManager {
    pub fn new(api_url: &str) -> Self {
        ExchangeManager {
            api_url: api_url.to_string(),
            client: Client::new(),
        }
    }
    pub async fn get_assets(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let url = format!("{}{}", self.api_url, constants::ASSETS);
        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response)
    }
}
