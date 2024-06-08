use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use serde_json::Value;
use tokio;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

mod order_book;
mod constants;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok(); // Load .env file

    let url = Url::parse("wss://api.ndax.io/WSGateway").expect("Invalid WebSocket URL");

    let mut order_book = order_book::OrderBook::new(10);

    // Connect to the WebSocket server
    let (ws_stream, response) = connect_async(url)
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
                        // println!("Update Event: {}", &json_msg);
                        order_book.initialize(&json_msg);
                    } else if update == constants::UPDATE {
                        println!("Update Event: {}", &json_msg);
                    }
                }
                // Assuming heartbeat messages can be distinguished by a lack of "b" or "a" keys
                else {
                    // Handle the heartbeat
                    println!("Unknown message");
                }
            }
            Ok(_) => (), // Handle other message types if necessary
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
