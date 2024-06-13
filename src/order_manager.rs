use hmac::NewMac;
use hmac::{Hmac, Mac};
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::constants;

// Type alias for the HMAC-SHA256 algorithm
type HmacSha256 = Hmac<Sha256>;

pub struct OrderManager {
    api_url: String,
    api_key: String,
    signature: String,
    user_id: String,
    account_name: String,
    account_id: String,
    client: Client,
}

impl OrderManager {
    pub fn new(
        api_url: &str,
        api_key: &str,
        signature: &str,
        user_id: &str,
        account_name: &str,
        account_id: &str,
    ) -> Self {
        OrderManager {
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            signature: signature.to_string(),
            user_id: user_id.to_string(),
            account_name: account_name.to_string(),
            account_id: account_id.to_string(),
            client: Client::new(),
        }
    }

    pub fn generate_auth_dict(&self) -> HashMap<&str, String> {
        let nonce = self.generate_nonce();
        let raw_signature = format!("{}{}{}", nonce, self.user_id, self.api_key);

        let mut mac: Hmac<Sha256> = HmacSha256::new_from_slice(self.signature.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(raw_signature.as_bytes());

        let signature = hex::encode(mac.finalize().into_bytes());

        let mut auth_info = HashMap::new();
        auth_info.insert("Nonce", nonce);
        auth_info.insert("APIKey", self.api_key.clone());
        auth_info.insert("Signature", signature);
        auth_info.insert("UserId", self.user_id.clone());

        auth_info
    }

    fn generate_nonce(&self) -> String {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis().to_string()
    }

    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    fn get_auth_headers(&self) -> HeaderMap {
        let mut headers = self.get_headers();
        let auth_info = self.generate_auth_dict();
        for (key, value) in auth_info {
            headers.insert(
                HeaderName::from_str(&key).unwrap(),
                HeaderValue::from_str(&value).unwrap(),
            );
        }
        headers
    }

    pub async fn authenticate(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let params = [
            ("APIKey", &self.api_key),
            ("Signature", &self.user_id),
            ("UserId", &self.account_name),
            ("Nonce", &self.generate_nonce()),
        ];

        let url = format!("{}{}", self.api_url, constants::AUTHENTICATE_USER_PATH_URL);
        let response = self
            .client
            .get(&url)
            .headers(self.get_auth_headers())
            .query(&params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response)
    }

    pub async fn get_account_id(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let params = [
            ("OMSId", "1"),
            ("UserId", &self.user_id),
            ("UserName", &self.account_name),
        ];

        let url = format!("{}{}", self.api_url, constants::USER_ACCOUNT_INFOS_PATH_URL);
        let response = self
            .client
            .get(&url)
            .headers(self.get_auth_headers())
            .query(&params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response)
    }

    pub async fn cancel_all_orders(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let query_params = [("OMSId", "1"), ("AccountId", &self.account_id)];

        let url = format!("{}{}", self.api_url, constants::CANCEL_ALL_ORDERS_PATH_URL);
        let response = self
            .client
            .get(&url)
            .headers(self.get_auth_headers())
            .query(&query_params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response)
    }

    pub async fn get_open_orders(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let query_params = [("OMSId", "1"), ("AccountId", &self.account_id)];

        let url = format!("{}{}", self.api_url, constants::GET_OPEN_ORDERS_PATH);
        let response = self
            .client
            .get(&url)
            .headers(self.get_auth_headers())
            .query(&query_params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response)
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// struct Order {
//     id: String,
//     side: String,
//     quantity: f64,
//     price: f64,
//     status: String,
// }

// #[derive(Serialize, Debug)]
// struct OrderRequest {
//     side: String,
//     quantity: f64,
//     price: f64,
// }

// #[derive(Deserialize, Debug)]
// struct OrderResponse {
//     id: String,
//     status: String,
//     filled_quantity: f64,
// }
