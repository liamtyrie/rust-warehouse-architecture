use thiserror::Error;

#[derive(Debug, Error)]
pub enum KafkaError {
    #[error("Failed to create Kafka Client: {0}")]
    ClientCreation(String),
    #[error("Failed to send message: {0}")]
    MessageSend(String),
    #[error("Message delivery failed: {0}")]
    MessageDelivery(String),
    #[error("Failed to Deserialize message: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("Connection timeout: {0}")]
    Timeout(String),
}

pub type KafkaResult<T> = Result<T, KafkaError>;
