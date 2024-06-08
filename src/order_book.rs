use hmac::digest::generic_array::typenum::array;
use serde_json::Value;
use ron::from_str;


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
        pub fn handle_bid(order: &Value){
            let aaa = "AAA";
        }
        pub fn handle_ask(order: &Value){
            let aaa = "AAA";
        }

        let orders: &str = snapshot.get("o").unwrap().as_str();

        let parsed: Vec<f32> = from_str(orders).unwrap();
        // println!("parsed: {}", parsed);

        println!("orders: {:?}", orders);

        // Iterate through the orders and separate them
        // for order in orders {
        //     println!("Order received: {:?}", order);
            // The 6th element in the array seems to indicate type, assuming the 7th item in sub-array is the type (1 or 2)
            // if let Some(order_type) = order.get(6).and_then(|x| x.as_i64()) {
            //     match order_type {
            //         1 => handle_bid(order),
            //         0 => handle_ask(order),
            //         _ => println!("Unknown order type: {}", order_type),
            //     }
            // }
        // }
        
        // Sort bids and asks
        self.bids
            .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
        self.asks
            .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
    }
}
