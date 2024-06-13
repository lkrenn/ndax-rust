use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use serde_json::Value;
use std::env;
use tokio;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

mod constants;
mod order_book;
mod order_manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok(); // Load .env file

    let url = Url::parse(constants::WSS_URL).expect("Invalid WebSocket URL");
    let api_url = Url::parse(constants::REST_URL).expect("Invalid REST URL");

    let api_key = env::var("API_KEY").expect("Invalid API KEY");
    let signature = env::var("SIGNATURE").expect("Invalid Signature");
    let user_id = env::var("USER_ID").expect("Invalid User ID");
    let account_name = env::var("ACCOUNT_NAME").expect("Invalid Account Name");
    let account_id = env::var("ACCOUNT_ID").expect("Invalid Account ID");

    let mut order_book = order_book::OrderBook::new(10);

    let order_manager = order_manager::OrderManager::new(
        &api_url.to_string(),
        &api_key.to_string(),
        &signature.to_string(),
        &user_id.to_string(),
        &account_name.to_string(),
        &account_id.to_string(),
    );

    // match order_manager.authenticate().await {
    //     Ok(orders) => println!("Open orders: {:?}", orders),
    //     Err(e) => println!("Error fetching open orders: {:?}", e),
    // }

    // match order_manager.get_open_orders().await {
    //     Ok(orders) => println!("Open orders: {:?}", orders),
    //     Err(e) => println!("Error fetching open orders: {:?}", e),
    // }

    match order_manager.cancel_all_orders().await {
        Ok(orders) => println!("Cancelled Orders: {:?}", orders),
        Err(e) => println!("Error cancelling orders: {:?}", e),
    }

    // Connect to the WebSocket server
    let (ws_stream, _response) = connect_async(url)
        .await
        .expect("Failed to connect to WebSocket server");

    // Now, correctly split ws_stream into a writer and reader parts
    let (mut write, read) = ws_stream.split();

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

    // let payload = json!({"OMSId":1,
    // "InstrumentId":1,
    // "Depth":100});

    // let message = json!({"m": 0,
    //     "i": 1,
    //     "n":"GetL2Snapshot",
    //     "o":payload.to_string()});

    let payload = json!({"OMSId":1,
        "InstrumentId":1,
        "Depth":10});

    let message = json!({"m": 0,
        "i": 1,
        "n":constants::SUBSCRIBE,
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
                let json_msg: Value = serde_json::from_str(&text)?;
                if let Some(update) = json_msg.get("n") {
                    if update == constants::SUBSCRIBE {
                        order_book.initialize(&json_msg);
                    } else if update == constants::UPDATE {
                        order_book.update(&json_msg);
                        println!("order book: {}", order_book);
                    }
                } else {
                    println!("Unknown message");
                }
            }
            Ok(_) => (), // Handle other message types if necessary
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
