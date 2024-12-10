use std::fmt;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Topic {
    //新订单
    SpotNewOrder,
    //撮合结果
    SpotMatchResult
}

impl fmt::Display for Topic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let topic_str = match self {
            Topic::SpotNewOrder => "SpotNewOrder",
            Topic::SpotMatchResult => "SpotMatchResult",
        };
        write!(f, "{}", topic_str)
    }
}
