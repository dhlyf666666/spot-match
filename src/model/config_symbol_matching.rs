use chrono::NaiveDateTime;
use sqlx::{query_as, FromRow};
use serde::{Deserialize, Serialize};
use crate::db_pool::{get_postgresql_pool};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ConfigSymbolMatching {
    pub id: i32,
    pub base: String,
    pub quote: String,
    pub is_open: i32,
    pub server: String,
    pub ctime: NaiveDateTime,
    pub mtime: NaiveDateTime,
}

impl ConfigSymbolMatching {
    /// 根据是否开放的状态来查询配置
    pub async fn get_config_by_is_open(
        is_open_value: i32,
    ) -> Result<Vec<ConfigSymbolMatching>, sqlx::Error> {
        // let conn = get_connection().await?;
        let configs = sqlx::query_as::<_, ConfigSymbolMatching>(
            "SELECT * FROM config_symbol_matching WHERE is_open = ?"
        )
            .bind(is_open_value)
            .fetch_all(get_postgresql_pool()?)
            .await?;
        Ok(configs)
    }

    pub async fn get_configs_by_is_open_and_server(
        is_open_value: i32,
        server_value: String,
    ) -> Result<Vec<ConfigSymbolMatching>, sqlx::Error> {
        let configs = sqlx::query_as::<_, ConfigSymbolMatching>(
            "SELECT * FROM config_symbol_matching WHERE is_open = ? AND server = ?"
        )
            .bind(is_open_value)
            .bind(server_value)
            .fetch_all(get_postgresql_pool()?)
            .await?;
        Ok(configs)
    }

    pub async fn get_configs_by_servers(
        servers: Vec<String>
    ) -> Result<Vec<ConfigSymbolMatching>, sqlx::Error> {
        // 如果 servers 为空，直接返回空结果
        if servers.is_empty() {
            return Ok(Vec::new());
        }

        // 动态构建占位符列表，例如 $1, $2, $3...
        let placeholders = (1..=servers.len())
            .map(|i| format!("${}", i))
            .collect::<Vec<_>>()
            .join(", ");

        // 构建动态 SQL 查询
        let query = format!(
            "SELECT * FROM config_symbol_matching WHERE server IN ({})",
            placeholders
        );

        // 创建查询并绑定参数
        let mut query_builder = query_as::<_, ConfigSymbolMatching>(&query);
        for server in servers {
            query_builder = query_builder.bind(server);
        }

        let configs = query_builder.fetch_all(get_postgresql_pool()?).await?;

        Ok(configs)
    }
}
