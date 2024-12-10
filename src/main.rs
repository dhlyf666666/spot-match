use tklog::{async_info, Format, ASYNC_LOG, LEVEL};
use spot_match::engine::Engine;
use spot_match::model::config_symbol_matching::ConfigSymbolMatching;
use spot_match::network::{get_ip_addresses};
use tokio::signal;

#[tokio::main]
async fn main() {
    // 初始化日志
    ASYNC_LOG
        .set_console(true)
        .set_level(LEVEL::Trace)
        .set_format(Format::LevelFlag | Format::Time | Format::ShortFileName)
        .set_cutmode_by_size("spot_match.log", 10000, 10, false).await;

    async_info!("Starting Order Book Matching System...");

    // 固定的允许的 MAC 地址
    // let allowed_mac = "aa:bb:cc:dd:ee:ff";
    // check_mac_address(allowed_mac);

    let config_symbol_matching = get_config_symbol_matching().await;

    // 创建所有引擎的句柄
    let mut engine_handles = Vec::new();

    // 启动每个交易对的撮合引擎
    for matching in config_symbol_matching {
        let symbol = matching.base.clone() + "/" + &matching.quote;
        let mut engine = Engine::new(symbol, matching.base.clone(), matching.quote.clone());

        // 如果需要从Config中获取数据给engine使用，可以在这里先获取并复制数据，如：
        // let config_data = {
        //     let guard = Config::global();
        //     // 提取所需数据，例如：
        //     guard.some_field.clone()
        // };

        let handle = tokio::spawn(async move {
            let _ = engine.run().await;
        });
        engine_handles.push(handle);
    }

    // 等待中断信号
    match signal::ctrl_c().await {
        Ok(()) => {
            async_info!("Shutdown signal received, initiating graceful shutdown...");
            // 在这里添加优雅关闭的逻辑
        }
        Err(err) => {
            async_info!("Error listening for shutdown signal: {}", err);
        }
    }

    // 等待所有引擎完成
    for handle in engine_handles {
        if let Err(e) = handle.await {
            async_info!("Engine task error: {}", e);
        }
    }
}

async fn get_config_symbol_matching() -> Vec<ConfigSymbolMatching> {

    let interfaces = match get_ip_addresses() {
        Ok(result) => result,
        Err(e) => {
            panic!("获取机器 IP 异常: {}", e);
        }
    };

    for interface in interfaces.clone() {
        async_info!("当前机器ip ", interface);
    }

    if interfaces.is_empty() {
        panic!("获取机器IP为空，程序终止");
    }

    let config_symbol_matching = match ConfigSymbolMatching::get_configs_by_servers(interfaces).await {
        Ok(matching) => matching,
        Err(e) => {
            panic!("获取matching异常，程序终止 {}", e);
        }
    };

    if config_symbol_matching.is_empty() {
        panic!("获取matching为空，程序终止");
    }

    config_symbol_matching
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_order_matching_basic() {
    }

    #[tokio::test]
    async fn test_order_matching_no_trade() {
    }
}
