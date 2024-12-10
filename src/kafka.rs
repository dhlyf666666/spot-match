use std::time::Duration;

use futures::future;

use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{Consumer, DefaultConsumerContext};
use rdkafka::error::{KafkaError};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::get_rdkafka_version;
// 修改这里：使用新的Message引入路径
use rdkafka::message::Message;

use tklog::{async_error, async_info};

// 使用默认的ConsumerContext
pub type LoggingConsumer = StreamConsumer<DefaultConsumerContext>;

pub fn create_consumer(brokers: &str, group_id: &str, topic: &str) -> Result<LoggingConsumer, KafkaError> {
    let (_, version) = get_rdkafka_version();
    tokio::spawn(async move {
        async_info!("rd_kafka_version: ", version);
    });

    let consumer: LoggingConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "5000")
        .set("enable.auto.offset.store", "false")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()?;

    // 尝试订阅指定的主题
    match consumer.subscribe(&[topic]) {
        Ok(_) => Ok(consumer),
        Err(e) => {
            eprintln!("Failed to subscribe to specified topic: {}", e);
            Err(e)
        }
    }

}

pub fn create_producer(brokers: &str) -> Option<FutureProducer> {
    let (_, version) = get_rdkafka_version();

    tokio::spawn(async move {
        async_info!("rd_kafka_version: ", version);
    });

    match ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("queue.buffering.max.ms", "0")
        .create()
    {
        Ok(producer) => Some(producer),
        Err(e) => {
            tokio::spawn(async move {
                async_error!("Producer creation failed: {}", e);
            });
            None
        }
    }
}

#[allow(dead_code)]
async fn process_messages(
    brokers: &str,
    group_id: &str,
    input_topic: &str,
    output_topics: &[&str],
) {
    let (_, version) = get_rdkafka_version();
    async_info!("rd_kafka_version: ", version);

    let consumer = match create_consumer(brokers, group_id, input_topic){
        Ok(consumer) => consumer,
        Err(e) => {
            // 在 Consumer 创建失败时采取措施，例如终止流程或重试
            async_error!("Failed to create consumer, terminating process. ", e);
            return;
        }
    };

    let producer = match create_producer(brokers) {
        Some(producer) => producer,
        None => {
            async_error!("Failed to create producer, terminating process.");
            return;
        }
    };

    loop {
        match consumer.recv().await {
            Err(e) => {
                async_error!("Kafka error: {}", e);
            }
            Ok(m) => {
                let result = future::try_join_all(output_topics.iter().map(|output_topic| {
                    let mut record = FutureRecord::to(output_topic);
                    if let Some(p) = m.payload() {
                        record = record.payload(p);
                    }
                    if let Some(k) = m.key() {
                        record = record.key(k);
                    }
                    producer.send(record, Duration::from_secs(1))
                })).await;

                match result {
                    Ok(_) => {
                        // 消息成功发送到所有输出主题
                        async_info!("Message successfully delivered to all output topics.");
                    }
                    Err(e) => {
                        // 处理消息传送失败的情况
                        async_error!("{}", format!("Message delivery failed for some topics: {:?}", e));

                        // 重试逻辑示例：重试发送到所有主题（可以根据需要调整重试次数或间隔时间）
                        for output_topic in output_topics {
                            let mut record = FutureRecord::to(output_topic);
                            if let Some(p) = m.payload() {
                                record = record.payload(p);
                            }
                            if let Some(k) = m.key() {
                                record = record.key(k);
                            }

                            match producer.send(record, Duration::from_secs(1)).await {
                                Ok(_) => async_info!("Retried message successfully delivered to topic: {}", output_topic),
                                Err(e) => {
                                    let err_str = format!("{:?}", e);
                                    async_error!("Retried message delivery failed for topic {}: {}", output_topic, err_str);
                                }
                            }
                        }
                    }
                }

                if let Err(e) = consumer.store_offset_from_message(&m) {
                    async_error!("Error while storing offset: {}", e);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_process_messages() {
    let brokers = "localhost:9092";
    let group_id = "test_group";
    let input_topic = "test_input";
    let output_topics = ["test_output1", "test_output2"];

    process_messages(brokers, group_id, input_topic, &output_topics).await;
}
