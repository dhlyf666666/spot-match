use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_yaml::Error as YamlError;
use std::fs;
use std::path::Path;
use std::sync::Arc;

static GLOBAL_CONFIG: Lazy<Arc<Config>> = Lazy::new(|| {
    let config = Config::load().expect("加载配置失败");
    Arc::new(config)
});

#[derive(Debug, Deserialize)]
pub struct Config {
    pub kafka_config: KafkaConfig,         // 替换 RabbitMQ 配置为 Kafka 配置
    pub mysql_config: Option<MysqlConfig>,
    pub postgresql_config: Option<PostgresqlConfig>,
}

#[derive(Debug, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,                  // Kafka 的 broker 列表，例如 "localhost:9092"
}

#[derive(Debug, Deserialize)]
pub struct MysqlConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct PostgresqlConfig {
    pub url: String,
}

impl Config {
    /// 从指定路径加载配置文件。如果找不到配置文件，则加载默认配置
    pub fn load() -> Result<Self, YamlError> {
        let exe_path = std::env::current_exe().expect("获取可执行文件路径失败");
        let exe_dir = exe_path.parent().expect("获取可执行文件目录失败");
        let config_path = exe_dir.join("config.yaml");

        if config_path.exists() {
            let config_content = fs::read_to_string(&config_path)
                .expect("读取 config.yaml 失败");
            serde_yaml::from_str(&config_content)
        } else {
            let default_config_path = Path::new("src/config.yaml");
            let default_content = fs::read_to_string(default_config_path)
                .expect("读取 src/config.yaml 失败");
            serde_yaml::from_str(&default_content)
        }
    }

    pub fn global() -> Arc<Config> {
        Arc::clone(&GLOBAL_CONFIG)
    }
}

/// 示例：如何使用全局配置
#[allow(dead_code)]
fn example_usage() {
    let config = Config::global();

    println!("RabbitMQ Brokers: {}", config.kafka_config.brokers);

    if let Some(mysql_config) = &config.mysql_config {
        println!("MySQL URL: {}", mysql_config.url);
    } else {
        println!("MySQL 配置不存在");
    }

    if let Some(postgresql_config) = &config.postgresql_config {
        println!("PostgreSQL URL: {}", postgresql_config.url);
    } else {
        println!("PostgreSQL 配置不存在");
    }
}
