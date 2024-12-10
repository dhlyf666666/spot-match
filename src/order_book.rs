use std::cmp::Reverse;
use std::collections::BTreeMap;
use ordered_float::OrderedFloat;
use tklog::async_info;
use crate::date::current_timestamp;
use crate::order::{Order, OrderType, Side};
use crate::spot_log::{LogType, SpotLog};
use crate::trade::Trade;


// 订单簿结构体
pub struct OrderBook {
    // 买单：按价格降序排列
    bids: BTreeMap<Reverse<OrderedFloat<f64>>, Vec<Order>>,
    // 卖单：按价格升序排列
    asks: BTreeMap<OrderedFloat<f64>, Vec<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    // 添加订单并执行撮合
    pub fn add_order(&mut self, order: Order) -> Vec<SpotLog> {
        let mut spot_log = Vec::new();
        let mut remaining_quantity = order.quantity;

        match order.side {
            Side::Buy => {
                // Collect prices to iterate to avoid modifying the BTreeMap during iteration
                let prices_to_iterate: Vec<OrderedFloat<f64>> = self.asks.keys().cloned().collect();
                let mut to_remove_prices = Vec::new();

                for price in prices_to_iterate {
                    if remaining_quantity <= 0.0 {
                        break;
                    }

                    // For limit orders, only match if buy price >= sell price
                    if order.order_type == OrderType::Limit && order.price < price.into_inner() {
                        break;
                    }

                    // Use a separate block to limit the mutable borrow scope
                    if let Some(orders_at_price) = self.asks.get_mut(&price) {
                        // Use a separate mutable borrow for the orders vector
                        let mut i = 0;
                        while i < orders_at_price.len() && remaining_quantity > 0.0 {
                            let sell_order = &mut orders_at_price[i];
                            let trade_price = sell_order.price;
                            let trade_quantity = remaining_quantity.min(sell_order.quantity);

                            let trade = Trade {
                                buy_order_id: order.id,
                                sell_order_id: sell_order.id,
                                price: trade_price,
                                quantity: trade_quantity,
                                timestamp: current_timestamp(),
                            };

                            spot_log.push(SpotLog {
                                log_type: LogType::Trade,
                                seq_id: 1,
                                order: None,
                                trade: Some(trade.clone()), // Clone to move into the async task
                            });

                            let order_id = order.id;
                            let sell_order_id = sell_order.id;
                            tokio::spawn(async move {
                                async_info!(
                                    "Matched Buy Order ID: {}, Sell Order ID: {}, Price: {:.2}, Quantity: {:.2}",
                                    order_id, sell_order_id, trade_price, trade_quantity
                                );
                            });

                            remaining_quantity -= trade_quantity;
                            sell_order.quantity -= trade_quantity;

                            if sell_order.quantity <= 0.0 {
                                // Remove the sell order if fully matched
                                orders_at_price.remove(i);
                            } else {
                                i += 1;
                            }
                        }

                        // Mark the price for removal if no orders remain
                        if orders_at_price.is_empty() {
                            to_remove_prices.push(price);
                        }
                    }
                }

                // Remove empty price levels after iteration
                for price in to_remove_prices {
                    self.asks.remove(&price);
                }

                // If there's remaining quantity and it's a limit order, add to bids
                if remaining_quantity > 0.0 && order.order_type == OrderType::Limit {
                    let buy_order = Order {
                        quantity: remaining_quantity,
                        ..order.clone()
                    };

                    tokio::spawn(async move {
                        async_info!(
                            "Added Buy Order to Order Book - ID: {}, User: {}, Price: {:.2}, Quantity: {:.2}",
                            buy_order.id, buy_order.user_id, buy_order.price, buy_order.quantity
                        );
                    });

                    self.bids
                        .entry(Reverse(OrderedFloat::from(buy_order.price)))
                        .or_insert_with(Vec::new)
                        .push(buy_order);
                }
            }
            Side::Sell => {
                // Similar restructuring for the Sell side
                let prices_to_iterate: Vec<Reverse<OrderedFloat<f64>>> =
                    self.bids.keys().cloned().collect();
                let mut to_remove_prices = Vec::new();

                for reverse_price in prices_to_iterate {
                    if remaining_quantity <= 0.0 {
                        break;
                    }

                    let price = reverse_price.0.into_inner();

                    // For limit orders, only match if sell price <= buy price
                    if order.order_type == OrderType::Limit && order.price > price {
                        break;
                    }

                    if let Some(orders_at_price) = self.bids.get_mut(&reverse_price) {
                        let mut i = 0;
                        while i < orders_at_price.len() && remaining_quantity > 0.0 {
                            let buy_order = &mut orders_at_price[i];
                            let trade_price = price;
                            let trade_quantity = remaining_quantity.min(buy_order.quantity);

                            let trade = Trade {
                                buy_order_id: buy_order.id,
                                sell_order_id: order.id,
                                price: trade_price,
                                quantity: trade_quantity,
                                timestamp: current_timestamp(),
                            };

                            spot_log.push(SpotLog {
                                log_type: LogType::Trade,
                                seq_id: 1,
                                order: None,
                                trade: Some(trade.clone()),
                            });

                            let order_id = order.id;
                            let buy_order_id = buy_order.id;
                            tokio::spawn(async move {
                                async_info!(
                                    "Matched Sell Order ID: {}, Buy Order ID: {}, Price: {:.2}, Quantity: {:.2}",
                                    order_id, buy_order_id, trade_price, trade_quantity
                                );
                            });

                            remaining_quantity -= trade_quantity;
                            buy_order.quantity -= trade_quantity;

                            if buy_order.quantity <= 0.0 {
                                // Remove the buy order if fully matched
                                orders_at_price.remove(i);
                            } else {
                                i += 1;
                            }
                        }

                        // Mark the price for removal if no orders remain
                        if orders_at_price.is_empty() {
                            to_remove_prices.push(reverse_price);
                        }
                    }
                }

                // Remove empty price levels after iteration
                for reverse_price in to_remove_prices {
                    self.bids.remove(&reverse_price);
                }

                // If there's remaining quantity and it's a limit order, add to asks
                if remaining_quantity > 0.0 && order.order_type == OrderType::Limit {
                    let sell_order = Order {
                        quantity: remaining_quantity,
                        ..order.clone()
                    };

                    tokio::spawn(async move {
                        async_info!(
                            "Added Sell Order to Order Book - ID: {}, User: {}, Price: {:.2}, Quantity: {:.2}",
                            sell_order.id, sell_order.user_id, sell_order.price, sell_order.quantity
                        );
                    });

                    self.asks
                        .entry(OrderedFloat::from(sell_order.price))
                        .or_insert_with(Vec::new)
                        .push(sell_order);
                }
            }
        }

        spot_log
    }

    pub fn len(&self) -> usize {
        self.bids.len() + self.asks.len()
    }

    pub async fn print_order_book(&self) {
        async_info!("=== Order Book ===");
        async_info!("--- Bids ---");
        for (Reverse(OrderedFloat(price)), orders) in &self.bids {
            for order in orders {
                async_info!(
                    "Order ID: {}, User ID: {}, Price: {}, Quantity: {}, Order Type: {}",
                    order.id, order.user_id, price, order.quantity, order.order_type
                );
            }
        }
        async_info!("--- Asks ---");
        for (&OrderedFloat(price), orders) in &self.asks {
            for order in orders {
                async_info!(
                    "Order ID: {}, User ID: {}, Price: {}, Quantity: {}, Order Type: {}",
                    order.id, order.user_id, price, order.quantity, order.order_type
                );
            }
        }
        async_info!("====================\n");
    }
}
