use std::fmt;
use serde::{Deserialize, Serialize};
use tklog::async_info;
use crate::order::Order;
use crate::trade::Trade;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotLog{
    pub log_type: LogType,
    pub seq_id: u64,
    pub order: Option<Order>,
    pub trade: Option<Trade>
}

impl SpotLog {

    // 打印 SpotLog 的数据
    pub async fn print_data(&self) {
        async_info!("SpotLog Data:");
        async_info!("  log_type: {:?}", self.log_type);
        async_info!("  seq_id: {}", self.seq_id);

        if let Some(ref order) = self.order {
            async_info!("  order: {:?}", order);
        } else {
            async_info!("  order: None");
        }

        if let Some(ref trade) = self.trade {
            async_info!("  trade: {:?}", trade);
        } else {
            async_info!("  trade: None");
        }
    }

}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogType{
    //新订单
    NewOrder,
    //取消订单
    CancelOrder,
    //订单成交
    Trade,
}

impl fmt::Display for LogType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log_type_str = match self {
            LogType::NewOrder => "NewOrder",
            LogType::CancelOrder => "CancelOrder",
            LogType::Trade => "Trade",
        };
        write!(f, "{}", log_type_str)
    }
}
