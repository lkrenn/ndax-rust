use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use hex::{decode, encode};
use hmac::{Hmac, Mac, NewMac};
use serde_json::json;
use sha2::Sha256;
use std::collections::HashMap;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

type HmacSha256 = Hmac<Sha256>;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok(); // Load .env file
    let public_key = std::env::var("PUBLIC_KEY").expect("PUBLIC_KEY not set in .env file");
    let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY not set in .env file");
    let user_id = std::env::var("USER_ID").expect("USER_ID not set in .env file");

    let url = Url::parse("wss://api.ndax.io/WSGateway").expect("Invalid WebSocket URL");

    // Connect to the WebSocket server
    let (ws_stream, response) = connect_async(url)
        .await
        .expect("Failed to connect to WebSocket server");

    // Now, correctly split ws_stream into a writer and reader parts
    let (mut write, read) = ws_stream.split();

    let nonce = generate_nonce().to_string();

    let signature = generate_signature(&nonce, &user_id, &public_key, &private_key);

    // // Define the payload
    // let payload = json!({
    //     "APIKey": public_key,
    //     "Signature": signature,
    //     "UserId": user_id,
    //     "Nonce": nonce
    // });

    // // Construct the message
    // let message = json!({
    //     "m": 0,
    //     "i": 1,
    //     "n": "AuthenticateUser",
    //     "o": payload.to_string()
    // });

    // // Send the message as a text frame
    // write
    //     .send(Message::Text(message.to_string()))
    //     .await
    //     .expect("Failed to send message");

    let payload = json!({"OMSId":1,
    "InstrumentId":1,
    "Depth":100});

    let message = json!({"m": 0,
        "i": 1,
        "n":"GetL2Snapshot",
        "o":payload.to_string()});

    // Send the message as a text frame
    write
        .send(Message::Text(message.to_string()))
        .await
        .expect("Failed to send message");

    let mut read = read;
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                println!("Received message: {}", text);
                // Optionally, process the message or parse it as JSON here
                // let msg: serde_json::Value = serde_json::from_str(&text).unwrap();
                // println!("Parsed message: {:?}", msg);
            }
            Ok(Message::Binary(bin)) => {
                println!("Received binary data: {:?}", bin);
            }
            Ok(_) => (), // Handle other message types if necessary
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break; // or handle the error as required
            }
        }
    }
}

fn generate_nonce() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

fn generate_signature(nonce: &str, user_id: &str, api_key: &str, secret: &str) -> String {
    let data = format!("{}{}{}", nonce, user_id, api_key);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize().into_bytes();
    let signature_hex = hex::encode(result);
    return signature_hex;
}
