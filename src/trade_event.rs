// trade_event.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeEvent {
    #[serde(rename = "TradeId")]
    pub trade_id: u64,
    #[serde(rename = "ProductPairCode")]
    pub instrument_id: u64,
    #[serde(rename = "Quantity")]
    pub quantity: f64,
    #[serde(rename = "Price")]
    pub price: f64,
    #[serde(rename = "Order1")]
    pub order_id_1: u64,
    #[serde(rename = "Order2")]
    pub order_id_2: u64,
    #[serde(rename = "TradeTime")]
    pub timestamp: u64,
    #[serde(rename = "Side")]
    pub side: u8,
    #[serde(rename = "TakerSide")]
    pub taker_side: u8,
    #[serde(rename = "isBlockTrade")]
    pub is_block_trade: u8,
    #[serde(rename = "orderClientId")]
    pub client_id: u8,
}
