use std::fmt;
use serde::{Deserialize, Serialize};

// 交易结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub buy_order_id: u64,
    pub sell_order_id: u64,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: u64,
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Trade {{ buy_order_id: {}, sell_order_id: {}, price: {:.2}, quantity: {:.2}, timestamp: {} }}",
            self.buy_order_id,
            self.sell_order_id,
            self.price,
            self.quantity,
            self.timestamp
        )
    }
}
