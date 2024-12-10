use std::sync::Arc;
use rdkafka::consumer::Consumer;
use rdkafka::message::BorrowedMessage;
use rdkafka::Message;
use tklog::{async_error, async_info};
use tokio::sync::{mpsc, Mutex};
use anyhow::Result;

use crate::config::Config;
use crate::kafka::{create_consumer};
use crate::order_book::OrderBook;
use crate::spot_log::SpotLog;
use crate::topic::Topic;

#[allow(dead_code)]
pub struct Engine {
    symbol: String,
    base_coin: String,
    quote_coin: String,
    order_book: Arc<Mutex<OrderBook>>,
    // 增加缓冲区大小，减少背压
    spot_log_sender: mpsc::Sender<SpotLog>,
    spot_log_receiver: Option<mpsc::Receiver<SpotLog>>,
}

impl Engine {
    pub fn new(symbol: String, base_coin: String, quote_coin: String) -> Self {
        // 增大channel容量，避免消息堆积导致的背压
        let (spot_log_sender, spot_log_receiver) = mpsc::channel(100_000);

        Engine {
            symbol,
            base_coin,
            quote_coin,
            order_book: Arc::new(Mutex::new(OrderBook::new())),
            spot_log_sender,
            spot_log_receiver: Some(spot_log_receiver),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // 启动消息处理器
        self.start_message_processor().await;

        // 启动消费者
        self.start_consumer().await;

        Ok(())
    }

    // 将消费者逻辑拆分出来
    async fn start_consumer(&mut self) {
        // 这里从全局配置中获取 brokers 等数据，但立即克隆出来，以避免持有 MutexGuard
        let brokers = {
            let config = Config::global();
            config.kafka_config.brokers.clone()
        };

        let group_id = format!("{}_{}_group", self.base_coin, self.quote_coin);
        let topic = format!("{}_{}_{}", self.base_coin, self.quote_coin, Topic::SpotNewOrder);

        async_info!("group_id   ", group_id.clone());
        async_info!("group_id   ", topic.clone());



        let consumer = match create_consumer(brokers.as_str(), &group_id, &topic) {
            Ok(consumer) => consumer,
            Err(e) => {
                async_error!("Failed to create consumer: {}", e);
                return;
            }
        };

        // 消费消息的主循环
        loop {
            match consumer.recv().await {
                Ok(m) => {
                    if let Err(e) = self.process_kafka_message(&consumer, &m).await {
                        async_error!("Error processing message: {}", e);
                    }
                },
                Err(e) => {
                    async_error!("Kafka receive error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    // 消息处理器
    async fn start_message_processor(&mut self) {
        let spot_log_receiver = self.spot_log_receiver.take()
            .expect("Receiver should exist");
        let order_book = self.order_book.clone();

        tokio::spawn(async move {
            // let mut batch = Vec::with_capacity(100); // 批量处理缓冲区
            let mut receiver = spot_log_receiver;

            while let Some(spot_log) = receiver.recv().await {
                if let Some(order) = spot_log.order {
                    let mut order_book_guard = order_book.lock().await;
                    order_book_guard.add_order(order);
                }
            }

        });
    }

    // 处理单个Kafka消息
    async fn process_kafka_message<C: Consumer>(
        &self,
        consumer: &C,
        message: &BorrowedMessage<'_>
    ) -> Result<()> {
        if let Some(payload) = message.payload() {
            let spot_log: SpotLog = serde_json::from_slice(payload)?;

            self.spot_log_sender.send(spot_log).await?;

            self.print_order_book().await;
            if let Err(e) = consumer.store_offset_from_message(message) {
                async_error!("Failed to store offset: ", e);
            }
        }
        Ok(())
    }

    pub async fn print_order_book(&self) {
        let order_book = self.order_book.lock().await;
        order_book.print_order_book().await;
    }
}
