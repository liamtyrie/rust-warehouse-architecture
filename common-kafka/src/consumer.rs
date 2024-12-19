use crate::config::KafkaConfigTrait;
use async_trait::async_trait;
use common_error::error::{KafkaError, KafkaResult};
use futures::StreamExt;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::ClientConfig;
use std::time::Duration;

#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, key: &[u8], payload: &[u8]) -> KafkaResult<()>;
}

pub struct EventConsumer {
    consumer: StreamConsumer,
    handler: Box<dyn MessageHandler>,
    max_retries: u32,
}

impl EventConsumer {
    pub fn new<T: KafkaConfigTrait>(
        config: T,
        handler: Box<dyn MessageHandler>,
    ) -> KafkaResult<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", config.brokers())
            .set("group.id", config.group_id())
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "6000")
            .set("max.poll.interval.ms", "300000")
            .create()
            .map_err(|e| KafkaError::ClientCreation(e.to_string()))?;

        consumer
            .subscribe(&[config.topic()])
            .map_err(|e| KafkaError::ClientCreation(e.to_string()))?;

        Ok(EventConsumer {
            consumer,
            handler,
            max_retries: config.max_retries(),
        })
    }

    pub async fn start(&self) -> KafkaResult<()> {
        let mut message_stream = self.consumer.stream();

        while let Some(message_result) = message_stream.next().await {
            match message_result {
                Ok(message) => {
                    let key = message.key().unwrap_or_default();
                    let payload = message.payload().unwrap_or_default();

                    match self.process_with_retry(key, payload).await {
                        Ok(_) => {
                            self.consumer
                                .commit_message(&message, CommitMode::Async)
                                .map_err(|e| KafkaError::MessageDelivery(e.to_string()))?;
                        }
                        Err(e) => {
                            tracing::error!("Failed to process message: {}", e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error receiving message: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
        Ok(())
    }

    async fn process_with_retry(&self, key: &[u8], payload: &[u8]) -> KafkaResult<()> {
        let mut retries = 0;
        let mut backoff = Duration::from_millis(100);

        while retries < self.max_retries {
            match self.handler.handle(key, payload).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    tracing::warn!("Retry {} failed: {}", retries, e);
                    retries += 1;
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                }
            }
        }

        Err(KafkaError::MessageDelivery("Max retries exceeded".into()))
    }
}
