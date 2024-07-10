use csv::WriterBuilder;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;
use tokio;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

mod entities;
use entities::trade_event::TradeEvent;

mod constants;
mod exchange_manager;
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

    // match order_manager.get_account_id().await {
    //     Ok(result) => println!("account id: {:?}", result),
    //     Err(e) => println!("Error getting accountId; {:?}", e),
    // }

    // match order_manager.authenticate().await {
    //     Ok(orders) => println!("Open orders: {:?}", orders),
    //     Err(e) => println!("Error fetching open orders: {:?}", e),
    // }

    // match order_manager.get_open_orders().await {
    //     Ok(orders) => println!("Open orders: {:?}", orders),
    //     Err(e) => println!("Error fetching open orders: {:?}", e),
    // }

    // match order_manager.cancel_all_orders().await {
    //     Ok(orders) => println!("Cancelled Orders: {:?}", orders),
    //     Err(e) => println!("Error cancelling orders: {:?}", e),
    // }

    let exchange_manager = exchange_manager::ExchangeManager::new(&api_url.to_string());

    // match exchange_manager.get_assets().await {
    //     Ok(assets) => println!("Asset codes: {:?}", assets),
    //     Err(e) => println!("Error fetching asset codes: {:?}", e),
    // }

    // Connect to the WebSocket server
    let (mut ws_stream, _response) = connect_async(url)
        .await
        .expect("Failed to connect to WebSocket server");

    // Now, correctly split ws_stream into a writer and reader parts
    let (mut write, read) = ws_stream.split();

    // Use Arc<Mutex<>> to share write between tasks
    let write = Arc::new(Mutex::new(write));
    let write_clone = Arc::clone(&write);

    // Spawn a task to send a ping message every 30 minutes
    let ping_interval = Duration::from_secs(5); // 30 minutes
    let ping_task = tokio::spawn(async move {
        let mut interval = interval(ping_interval);
        loop {
            interval.tick().await;
            let ping_message = json!({
                "m": 0,
                "i": 1,
                "n": "Ping",
            })
            .to_string();
            let mut write = write_clone.lock().await;
            if let Err(e) = write.send(Message::Text(ping_message)).await {
                eprintln!("Error sending ping: {}", e);
                break;
            }
        }
    });

    let mut write = write.lock().await;

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

    let trades_subscribe_payload = json!({"OMSId":1,
        "InstrumentId":1,
        "IncludeLastCount":10});

    let message = json!({"m": 0,
        "i": 1,
        "n":constants::SUBSCRIBE_TRADES,
        "o":trades_subscribe_payload.to_string()});

    // Send the message as a text frame
    write
        .send(Message::Text(message.to_string()))
        .await
        .expect("Failed to send message");

    let usdc_trades_subscribe_payload = json!({"OMSId":1,
        "InstrumentId":90,
        "IncludeLastCount":10});

    let message = json!({"m": 0,
        "i": 1,
        "n":constants::SUBSCRIBE_TRADES,
        "o":usdc_trades_subscribe_payload.to_string()});

    // Send the message as a text frame
    write
        .send(Message::Text(message.to_string()))
        .await
        .expect("Failed to send message");

    // let order_book_subscribe_payload = json!({"OMSId":1,
    //     "InstrumentId":1,
    //     "Depth":10});

    // let message = json!({"m": 0,
    //     "i": 1,
    //     "n":constants::SUBSCRIBE,
    //     "o":payload.to_string()});

    let mut read = read;
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => {
                match msg {
                    Message::Pong(_) => {
                        println!("Pong");
                    }
                    Message::Binary(bin) => {
                        println!("Received binary: {:?}", bin);
                    }
                    Message::Ping(ping) => {
                        println!("Received ping: {:?}", ping);
                        if let Err(e) = write.send(Message::Pong(ping)).await {
                            eprintln!("Error sending pong: {}", e);
                            break;
                        }
                    }
                    Message::Close(_) => {
                        println!("Received close message");
                        break;
                    }
                    Message::Text(text) => {
                        let json_msg: Value = serde_json::from_str(&text.to_string())?;
                        if let Some(update) = json_msg.get("n") {
                            println!("Update detected: {}", update);
                            if update == constants::SUBSCRIBE {
                                order_book.initialize(&json_msg);
                            } else if update == constants::UPDATE {
                                order_book.update(&json_msg);
                                println!("order book: {}", order_book);
                            } else if update == constants::SUBSCRIBE_TRADES {
                                println!("subscribe trades: {}", json_msg);
                            } else if update == constants::UPDATE_TRADES {
                                println!("Trade detected:{}", json_msg);
                                // Parse the JSON data
                                // Check if the field "o" exists
                                if let Some(o_field) = json_msg.get("o") {
                                    // If it's a string, parse it again as JSON to get the array
                                    if let Some(o_str) = o_field.as_str() {
                                        let o_array: Value = serde_json::from_str(o_str)?;
                                        if let Some(array) = o_array.as_array() {
                                            for trade_array in array {
                                                if let Some(trade_data) = trade_array.as_array() {
                                                    let trade_event = TradeEvent {
                                                        trade_id: trade_data[0].as_u64().unwrap(),
                                                        instrument_id: trade_data[1]
                                                            .as_u64()
                                                            .unwrap(),
                                                        quantity: trade_data[2].as_f64().unwrap(),
                                                        price: trade_data[3].as_f64().unwrap(),
                                                        order_id_1: trade_data[4].as_u64().unwrap(),
                                                        order_id_2: trade_data[5].as_u64().unwrap(),
                                                        timestamp: trade_data[6].as_u64().unwrap(),
                                                        side: trade_data[7].as_u64().unwrap() as u8,
                                                        taker_side: trade_data[8].as_u64().unwrap()
                                                            as u8,
                                                        is_block_trade: trade_data[9]
                                                            .as_u64()
                                                            .unwrap()
                                                            as u8,
                                                        client_id: trade_data[10].as_u64().unwrap()
                                                            as u8,
                                                    };

                                                    // Append to CSV
                                                    append_to_csv("trades.csv", &trade_event)?;
                                                }
                                            }
                                        } else {
                                            println!("'o' field is not an array after parsing.");
                                        }
                                    } else {
                                        println!("'o' field is not a string.");
                                    }
                                } else {
                                    println!("'o' field does not exist.");
                                }
                            }
                        }
                    }
                }
            }
            Ok(_) => (), // Handle other message types if necessary
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

fn append_to_csv<P: AsRef<Path>>(path: P, trade_event: &TradeEvent) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)?;

    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);
    wtr.serialize(trade_event)?;
    wtr.flush()?;
    Ok(())
}
