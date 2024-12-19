pub mod config;
pub mod consumer;
pub mod producer;

use config::{FulfillmentConfig, InboundConfig};
pub use consumer::{EventConsumer, MessageHandler};
pub use producer::EventProducer;

use common_error::error::KafkaResult;

use async_trait::async_trait;

pub struct MessagePrinter {}

impl MessagePrinter {
    fn new() -> Box<Self> {
        Box::new(MessagePrinter {})
    }
}

#[async_trait]
impl MessageHandler for MessagePrinter {
    async fn handle(&self, key: &[u8], payload: &[u8]) -> KafkaResult<()> {
        println!("Key: {}", String::from_utf8_lossy(key));
        println!("Payload: {}", String::from_utf8_lossy(payload));

        Ok(())
    }
}

#[tokio::main]
async fn main() -> KafkaResult<()> {
    tracing_subscriber::fmt::init();

    let inbound_config = InboundConfig::new();
    let fulfillment_config = FulfillmentConfig::new();

    let inbound_consumer = EventConsumer::new(inbound_config, MessagePrinter::new())?;
    let fulfillment_consumer = EventConsumer::new(fulfillment_config, MessagePrinter::new())?;

    tokio::try_join!(inbound_consumer.start(), fulfillment_consumer.start())?;

    Ok(())
}
