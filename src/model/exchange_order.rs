use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use crate::db_pool::{get_postgresql_pool};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ExchangeOrder {
    pub order_id: String,
    pub amount: Option<Decimal>,
    pub base_symbol: Option<String>,
    pub canceled_time: Option<i64>,
    pub coin_symbol: Option<String>,
    pub completed_time: Option<i64>,
    pub direction: Option<i32>,
    pub member_id: Option<i64>,
    pub price: Option<Decimal>,
    pub status: Option<i32>,
    pub symbol: Option<String>,
    pub time: Option<i64>,
    pub traded_amount: Option<Decimal>,
    pub turnover: Option<Decimal>,
    pub order_type: Option<i32>,
    pub use_discount: Option<String>,
    pub order_resource: Option<i32>,
}

impl ExchangeOrder {
    /// 根据订单状态查询订单
    pub async fn get_orders_by_status(status_value: i32) -> Result<Vec<ExchangeOrder>, sqlx::Error> {
        let orders = sqlx::query_as::<_, ExchangeOrder>(
            "SELECT * FROM exchange_order WHERE status = ?"
        )
            .bind(status_value)
            .fetch_all(get_postgresql_pool()?)
            .await?;
        Ok(orders)
    }

    /// 根据一组订单状态查询订单
    pub async fn get_orders_by_status_list(status_values: Vec<i32>) -> Result<Vec<ExchangeOrder>, sqlx::Error> {

        // 构建 SQL 查询语句
        let query = format!(
            "SELECT * FROM exchange_order WHERE status IN ({})",
            status_values.iter().map(|_| "?").collect::<Vec<&str>>().join(", ")
        );

        // 构建查询并绑定参数
        let mut query_builder = sqlx::query_as::<_, ExchangeOrder>(&query);
        for status in status_values {
            query_builder = query_builder.bind(status);
        }

        // 执行查询并获取结果
        let orders = query_builder.fetch_all(get_postgresql_pool()?).await?;
        Ok(orders)
    }
}
