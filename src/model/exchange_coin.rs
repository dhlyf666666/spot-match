use rust_decimal::Decimal;
use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use crate::db_pool::get_postgresql_pool;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ExchangeCoin {
    pub symbol: String,
    pub base_coin_scale: i32,
    pub base_symbol: Option<String>,
    pub coin_scale: i32,
    pub coin_symbol: Option<String>,
    pub enable: i32,
    pub fee: Option<Decimal>,
    pub sort: i32,
    pub enable_market_buy: i32,
    pub enable_market_sell: i32,
    pub min_sell_price: Option<Decimal>,
    pub flag: i32,
    pub max_trading_order: i32,
    pub max_trading_time: i32,
    pub instrument: Option<String>,
    pub min_turnover: Option<Decimal>,
    pub max_volume: Option<Decimal>,
    pub min_volume: Option<Decimal>,
    pub zone: i32,
    pub clear_time: Option<String>,
    pub end_time: Option<String>,
    pub publish_price: Option<Decimal>,
    pub publish_type: i32,
    pub start_time: Option<String>,
    pub exchangeable: i32,
    pub publish_amount: Option<Decimal>,
    pub visible: i32,
    pub max_buy_price: Option<Decimal>,
    pub robot_type: i32,
    pub enable_buy: i32,
    pub enable_sell: i32,
}

impl ExchangeCoin {
    pub async fn get_exchange_coins_by_exchangeable(
        exchangeable_value: i32,
    ) -> Result<Vec<ExchangeCoin>, sqlx::Error> {
        // let mut conn = get_connection().await?;
        let coins = sqlx::query_as::<_, ExchangeCoin>("SELECT * FROM exchange_coin WHERE exchangeable = ?")
            .bind(exchangeable_value)
            .fetch_all(get_postgresql_pool()?)
            .await?;
        Ok(coins)
    }
}
