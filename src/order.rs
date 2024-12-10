use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use flatbuffers::FlatBufferBuilder;
use serde::{Deserialize, Serialize};
use crate::date::current_timestamp;
use crate::fbs::order_generated::order::{Order as FbsOrder, OrderArgs, OrderType as FbsOrderType, Side as FbsSide};

static ORDER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);


// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let side_str = match self {
            Side::Buy => "Buy",
            Side::Sell => "Sell",
        };
        write!(f, "{}", side_str)
    }
}

// 订单类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderType::Limit => write!(f, "Limit"),
            OrderType::Market => write!(f, "Market"),
        }
    }
}

// 订单结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub user_id: u64,
    pub price: f64,      // 市价单可设为0
    pub quantity: f64,
    pub timestamp: u64,  // 毫秒级时间戳
    pub order_type: OrderType,
    pub side: Side,
}

// 假设 OrderType 和 Side 已实现 Display 特征
impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Order {{ id: {}, user_id: {}, price: {:.2}, quantity: {:.2}, timestamp: {}, order_type: {}, side: {} }}",
            self.id,
            self.user_id,
            self.price,
            self.quantity,
            self.timestamp,
            self.order_type,
            self.side
        )
    }
}

impl Order {
    pub fn new(user_id: u64, price: f64, quantity: f64, order_type: OrderType, side: Side) -> Self {
        Order {
            id: ORDER_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            user_id,
            price,
            quantity,
            timestamp: current_timestamp(),
            order_type,
            side,
        }
    }

    /// 从 FlatBuffers 数据解析出 Order 实例
    pub fn parse_order_flatbuffer(data: &[u8]) -> Result<Self, &'static str> {
        let fbs_order = flatbuffers::root::<FbsOrder>(data).map_err(|_| "Failed to parse FlatBuffer data as Order")?;

        // 将 FlatBuffers 数据转换成 Rust 的 Order 实例
        Ok(Order {
            id: fbs_order.id(),
            user_id: fbs_order.user_id(),
            price: fbs_order.price(),
            quantity: fbs_order.quantity(),
            timestamp: fbs_order.timestamp(),
            order_type: match fbs_order.order_type() {
                FbsOrderType::Limit => OrderType::Limit,
                FbsOrderType::Market => OrderType::Market,
                _ => return Err("Invalid order type in FlatBuffer data"),
            },
            side: match fbs_order.side() {
                FbsSide::Buy => Side::Buy,
                FbsSide::Sell => Side::Sell,
                _ => return Err("Invalid side in FlatBuffer data"),
            },
        })
    }

    /// 将 Order 实例序列化为 FlatBuffers 格式
    pub fn to_flatbuffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::with_capacity(1024);

        let order = FbsOrder::create(
            &mut builder,
            &OrderArgs {
                id: self.id,
                user_id: self.user_id,
                price: self.price,
                quantity: self.quantity,
                timestamp: self.timestamp,
                order_type: match self.order_type {
                    OrderType::Limit => FbsOrderType::Limit,
                    OrderType::Market => FbsOrderType::Market,
                },
                side: match self.side {
                    Side::Buy => FbsSide::Buy,
                    Side::Sell => FbsSide::Sell,
                },
            },
        );

        builder.finish(order, None);

        builder.finished_data().to_vec()
    }
}
