use once_cell::sync::Lazy;
use sqlx::{Error, MySql, Pool, PgPool};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use crate::config::Config;

// MySQL 连接池
static MYSQL_POOL: Lazy<Option<Pool<MySql>>> = Lazy::new(|| {
    Config::global()
        .mysql_config
        .as_ref()
        .map(|mysql_config| {
            MySqlPoolOptions::new()
                .max_connections(10)
                .connect_lazy(mysql_config.url.as_str())
                .expect("MySQL 连接池初始化失败")
        })
});

// PostgreSQL 连接池
static POSTGRESQL_POOL: Lazy<Option<PgPool>> = Lazy::new(|| {
    Config::global()
        .postgresql_config
        .as_ref()
        .map(|postgresql_config| {
            PgPoolOptions::new()
                .max_connections(10)
                .connect_lazy(postgresql_config.url.as_str())
                .expect("PostgreSQL 连接池初始化失败")
        })
});

// 获取 MySQL 连接池，如果不存在则打印错误并返回Err
pub fn get_mysql_pool() -> Result<&'static Pool<MySql>, Error> {
    match MYSQL_POOL.as_ref() {
        Some(pool) => Ok(pool),
        None => {
            eprintln!("No MySQL pool available");
            Err(Error::Configuration("No MySQL pool available".into()))
        }
    }
}

// 获取 PostgreSQL 连接池，如果不存在则打印错误并返回Err
pub fn get_postgresql_pool() -> Result<&'static PgPool, Error> {
    match POSTGRESQL_POOL.as_ref() {
        Some(pool) => Ok(pool),
        None => {
            eprintln!("No PostgreSQL pool available");
            Err(Error::Configuration("No PostgreSQL pool available".into()))
        }
    }
}

// 获取一个 MySQL 连接，如果连接池不存在则返回Err
pub async fn get_mysql_connection() -> Result<sqlx::pool::PoolConnection<MySql>, Error> {
    let pool = get_mysql_pool()?;
    let conn = pool.acquire().await?;
    Ok(conn)
}

// 获取一个 PostgreSQL 连接，如果连接池不存在则返回Err
pub async fn get_postgresql_connection() -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>, Error> {
    let pool = get_postgresql_pool()?;
    let conn = pool.acquire().await?;
    Ok(conn)
}
