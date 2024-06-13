use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    price: f64,
    volume: f64,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    depth: usize,
    bids: Vec<Level>,
    asks: Vec<Level>,
}

impl OrderBook {
    pub fn new(depth: usize) -> Self {
        OrderBook {
            depth,
            bids: Vec::with_capacity(depth),
            asks: Vec::with_capacity(depth),
        }
    }

    // Initializes the order book with a snapshot
    pub fn initialize(&mut self, snapshot: &Value) {
        if let Some(order_string) = snapshot.get("o").and_then(Value::as_str) {
            if let Ok(orders) = serde_json::from_str::<Vec<Vec<Value>>>(order_string) {
                println!("Orders: {:?}", orders);
                for order in orders.iter() {
                    if let Some(order_type) = order.last().and_then(|v| v.as_i64()) {
                        if order_type == 0 {
                            // This is a bid
                            if let (Some(price), Some(volume)) = (
                                order.get(6).and_then(|v| v.as_f64()),
                                order.get(8).and_then(|v| v.as_f64()),
                            ) {
                                self.bids.push(Level { price, volume });
                            }
                        } else if order_type == 1 {
                            // This is an ask
                            if let (Some(price), Some(volume)) = (
                                order.get(6).and_then(|v| v.as_f64()),
                                order.get(8).and_then(|v| v.as_f64()),
                            ) {
                                self.asks.push(Level { price, volume });
                            }
                        } else {
                            println!("Failed to parse 'o' into an array");
                        }
                    } else {
                        println!("'o' is not a string");
                    }
                }

                // Sort bids and asks
                self.bids
                    .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
                self.asks
                    .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
            }
        }
    }

    pub fn update(&mut self, update: &serde_json::Value) {
        if let Some(update_string) = update.get("o").and_then(Value::as_str) {
            if let Ok(update_data) = serde_json::from_str::<Vec<Vec<Value>>>(update_string) {
                for order in update_data {
                    if let Some(order_type) = order.last().and_then(|v| v.as_i64()) {
                        let price = order.get(6).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let volume = order.get(8).and_then(|v| v.as_f64()).unwrap_or(0.0);

                        if order_type == 0 {
                            // This is a bid
                            if volume == 0.0 {
                                // Delete the price level with 0 volume
                                self.bids.retain(|b| b.price != price);
                            } else {
                                // Check if the price level exists and update or insert accordingly
                                match self.bids.iter_mut().find(|b| b.price == price) {
                                    Some(existing_bid) => existing_bid.volume = volume, // Update existing
                                    None => {
                                        // Insert new price level in sorted order
                                        let new_bid = Level { price, volume };
                                        self.bids.push(new_bid);
                                        self.bids
                                            .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
                                    }
                                }
                            }
                        } else if order_type == 1 {
                            // This is an ask
                            if volume == 0.0 {
                                // Delete the price level with 0 volume
                                self.asks.retain(|a| a.price != price);
                            } else {
                                // Check if the price level exists and update or insert accordingly
                                match self.asks.iter_mut().find(|a| a.price == price) {
                                    Some(existing_ask) => existing_ask.volume = volume, // Update existing
                                    None => {
                                        // Insert new price level in sorted order
                                        let new_ask = Level { price, volume };
                                        self.asks.push(new_ask);
                                        self.asks
                                            .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        self.truncate_to_depth();
    }

    fn truncate_to_depth(&mut self) {
        // Truncate asks to the specified depth
        if self.asks.len() > self.depth {
            self.asks.truncate(self.depth);
        }
        // Truncate bids to the specified depth
        if self.bids.len() > self.depth {
            self.bids.truncate(self.depth);
        }

        // Since we may have inserted a new price level, ensure the order book is sorted
        self.asks
            .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        self.bids
            .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.5} ({:.8})", self.price, self.volume)
    }
}

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Order Book:")?;
        writeln!(
            f,
            "           {:<12} {:<11} | {:<12 } {}",
            "Bid", "Depth", "Ask", "Depth"
        )?;
        for i in 0..self.depth {
            let bid_level = self
                .bids
                .get(i)
                .map_or("".to_string(), |level| format!("{}", level));
            let ask_level = self
                .asks
                .get(i)
                .map_or("".to_string(), |level| format!("{}", level));
            writeln!(
                f,
                "{:<10} {:<20} | {:<10} {}",
                i + 1,
                bid_level,
                ask_level,
                i + 1
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn get_snapshot() -> Value {
        serde_json::json!({
            "i": 1,
            "m": 1,
            "n": "SubscribeLevel2",
            "o": "[[1,1718003785385,0,5711.80000,1,0,1,8.13439401,1],
                  [2,1718003785385,0,5712.20000,1,0,1,2.00000000,1],
                  [3,1718003785385,0,5712.80000,1,0,1,0.30000000,1],
                  [4,1718003785385,0,5713.00000,1,0,1,3.29800000,1],
                  [5,1718003785385,0,5713.10000,1,0,1,1.00000000,1],
                  [6,1718003785385,0,5713.90000,1,0,1,1.00000000,1],
                  [7,1718003785385,0,5714.70000,1,0,1,0.50000000,1],
                  [8,1718003785385,0,5715.20000,1,0,1,1.00000000,1],
                  [9,1718003785385,0,5716.60000,1,0,1,1.22700000,1],
                  [10,1718003785385,0,5716.80000,1,0,1,0.35000000,1],
                  [11,1718003785385,0,5711.70000,1,0,0,0.00749800,0],
                  [12,1718003785385,0,5709.20000,1,0,0,3.30000000,0],
                  [13,1718003785385,0,5708.30000,1,0,0,0.75483907,0],
                  [14,1718003785385,0,5708.20000,1,0,0,5.00000000,0],
                  [15,1718003785385,0,5707.80000,1,0,0,2.50000000,0],
                  [16,1718003785385,0,5707.40000,1,0,0,4.33000000,0],
                  [17,1718003785385,0,5707.00000,1,0,0,0.00200000,0],
                  [18,1718003785385,0,5706.90000,1,0,0,1.17300000,0],
                  [19,1718003785385,0,5706.40000,1,0,0,0.85600000,0],
                  [20,1718003785385,0,5706.30000,1,0,0,1.00000000,0]]"
        })
    }

    fn get_update1() -> Value {
        serde_json::json!({
            "i": 140,
            "m": 3,
            "n": "Level2UpdateEvent",
            "o": "[[261,1718007168597,2,5709.20000,0,0,1,3.00000000,0],
                  [262,1718007168597,1,5708.20000,2,0,1,0.00000000,0],
                  [263,1718007169610,0,5705.90000,1,0,1,7.62400000,0]]"
        })
    }

    fn get_update2() -> Value {
        serde_json::json!({
            "i": 141,
            "m": 3,
            "n": "Level2UpdateEvent",
            "o": "[[264,1718007169611,2,5709.20000,0,0,1,8.00000000,0],
                  [265,1718007169612,1,5709.40000,0,0,1,0.30000000,0]]"
        })
    }

    fn get_update3() -> Value {
        serde_json::json!({
            "i": 142,
            "m": 3,
            "n": "Level2UpdateEvent",
            "o": "[[266,1718007169613,1,5708.30000,0,0,1,0.00000000,0],
                  [267,1718007169614,2,5705.90000,0,0,1,7.62400000,0]]"
        })
    }

    fn get_expected_order_book1() -> Value {
        serde_json::json!({
            "i": 1,
            "m": 1,
            "n": "SubscribeLevel2",
            "o": "[[1,1718003785385,0,5711.80000,1,0,1,8.13439401,1],
                  [2,1718003785385,0,5712.20000,1,0,1,2.00000000,1],
                  [3,1718003785385,0,5712.80000,1,0,1,0.30000000,1],
                  [4,1718003785385,0,5713.00000,1,0,1,3.29800000,1],
                  [5,1718003785385,0,5713.10000,1,0,1,1.00000000,1],
                  [6,1718003785385,0,5713.90000,1,0,1,1.00000000,1],
                  [7,1718003785385,0,5714.70000,1,0,1,0.50000000,1],
                  [8,1718003785385,0,5715.20000,1,0,1,1.00000000,1],
                  [9,1718003785385,0,5716.60000,1,0,1,1.22700000,1],
                  [10,1718003785385,0,5716.80000,1,0,1,0.35000000,1],
                  [11,1718003785385,0,5711.70000,1,0,0,0.00749800,0],
                  [12,1718003785385,0,5709.20000,1,0,0,3.00000000,0],
                  [13,1718003785385,0,5708.30000,1,0,0,0.75483907,0],
                  [14,1718003785385,0,5707.80000,1,0,0,2.50000000,0],
                  [15,1718003785385,0,5707.40000,1,0,0,4.33000000,0],
                  [16,1718003785385,0,5707.00000,1,0,0,0.00200000,0],
                  [17,1718003785385,0,5706.90000,1,0,0,1.17300000,0],
                  [18,1718003785385,0,5706.40000,1,0,0,0.85600000,0],
                  [19,1718003785385,0,5706.30000,1,0,0,1.00000000,0],
                  [20,1718003785385,0,5705.90000,1,0,0,7.62400000,0]]"
        })
    }

    fn get_expected_order_book2() -> Value {
        serde_json::json!({
            "i": 2,
            "m": 1,
            "n": "SubscribeLevel2",
            "o": "[[1,1718003785385,0,5711.80000,1,0,1,8.13439401,1],
                  [2,1718003785385,0,5712.20000,1,0,1,2.00000000,1],
                  [3,1718003785385,0,5712.80000,1,0,1,0.30000000,1],
                  [4,1718003785385,0,5713.00000,1,0,1,3.29800000,1],
                  [5,1718003785385,0,5713.10000,1,0,1,1.00000000,1],
                  [6,1718003785385,0,5713.90000,1,0,1,1.00000000,1],
                  [7,1718003785385,0,5714.70000,1,0,1,0.50000000,1],
                  [8,1718003785385,0,5715.20000,1,0,1,1.00000000,1],
                  [9,1718003785385,0,5716.60000,1,0,1,1.22700000,1],
                  [10,1718003785385,0,5716.80000,1,0,1,0.35000000,1],
                  [11,1718003785385,0,5711.70000,1,0,0,0.00749800,0],
                  [12,1718003785385,0,5709.40000,1,0,0,0.30000000,0],
                  [13,1718003785385,0,5709.20000,1,0,0,8.00000000,0],
                  [14,1718003785385,0,5708.30000,1,0,0,0.75483907,0],
                  [15,1718003785385,0,5707.80000,1,0,0,2.50000000,0],
                  [16,1718003785385,0,5707.40000,1,0,0,4.33000000,0],
                  [17,1718003785385,0,5707.00000,1,0,0,0.00200000,0],
                  [18,1718003785385,0,5706.90000,1,0,0,1.17300000,0],
                  [19,1718003785385,0,5706.40000,1,0,0,0.85600000,0],
                  [20,1718003785385,0,5706.30000,1,0,0,1.00000000,0]]"
        })
    }

    fn get_expected_order_book3() -> Value {
        serde_json::json!({
            "i": 3,
            "m": 1,
            "n": "SubscribeLevel2",
            "o": "[[1,1718003785385,0,5711.80000,1,0,1,8.13439401,1],
                  [2,1718003785385,0,5712.20000,1,0,1,2.00000000,1],
                  [3,1718003785385,0,5712.80000,1,0,1,0.30000000,1],
                  [4,1718003785385,0,5713.00000,1,0,1,3.29800000,1],
                  [5,1718003785385,0,5713.10000,1,0,1,1.00000000,1],
                  [6,1718003785385,0,5713.90000,1,0,1,1.00000000,1],
                  [7,1718003785385,0,5714.70000,1,0,1,0.50000000,1],
                  [8,1718003785385,0,5715.20000,1,0,1,1.00000000,1],
                  [9,1718003785385,0,5716.60000,1,0,1,1.22700000,1],
                  [10,1718003785385,0,5716.80000,1,0,1,0.35000000,1],
                  [11,1718003785385,0,5711.70000,1,0,0,0.00749800,0],
                  [12,1718003785385,0,5709.40000,1,0,0,0.30000000,0],
                  [13,1718003785385,0,5709.20000,1,0,0,8.00000000,0],
                  [14,1718003785385,0,5707.80000,1,0,0,2.50000000,0],
                  [15,1718003785385,0,5707.40000,1,0,0,4.33000000,0],
                  [16,1718003785385,0,5707.00000,1,0,0,0.00200000,0],
                  [17,1718003785385,0,5706.90000,1,0,0,1.17300000,0],
                  [18,1718003785385,0,5706.40000,1,0,0,0.85600000,0],
                  [19,1718003785385,0,5706.30000,1,0,0,1.00000000,0],
                  [20,1718003785385,0,5705.90000,1,0,0,7.62400000,0]]"
        })
    }

    #[test]
    fn test_order_book_snapshot() {
        let snapshot = get_snapshot();
        let mut order_book = OrderBook::new(10);
        order_book.initialize(&snapshot);
        println!("Order Book Snapshot: {:?}", order_book);
        assert_eq!(order_book.bids.len(), 10);
        assert_eq!(order_book.asks.len(), 10);
    }

    #[test]
    fn test_order_book_update() {
        // Initialize the OrderBook with a known snapshot.
        let mut order_book = OrderBook::new(10);
        let initial_snapshot = get_snapshot();

        order_book.initialize(&initial_snapshot);

        // Apply updates to the OrderBook.
        let updates1 = get_update1();
        order_book.update(&updates1);

        // Verify that the OrderBook now matches the expected output.
        let mut expected_order_book = OrderBook::new(10);
        expected_order_book.initialize(&get_expected_order_book1());

        assert_eq!(order_book.asks, expected_order_book.asks);
        assert_eq!(order_book.bids, expected_order_book.bids);

        // Apply another update to the OrderBook
        let updates2 = get_update2();
        order_book.update(&updates2);

        expected_order_book = OrderBook::new(10);
        expected_order_book.initialize(&get_expected_order_book2());

        assert_eq!(order_book.asks, expected_order_book.asks);
        assert_eq!(order_book.bids, expected_order_book.bids);

        // Apply another update to the OrderBook
        let updates3 = get_update3();
        order_book.update(&updates3);

        expected_order_book = OrderBook::new(10);
        expected_order_book.initialize(&get_expected_order_book3());

        assert_eq!(order_book.asks, expected_order_book.asks);
        assert_eq!(order_book.bids, expected_order_book.bids);
    }
}
