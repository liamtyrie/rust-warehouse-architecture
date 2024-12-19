pub mod config;
pub mod consumer;
pub mod producer;

use config::{FulfillmentConfig, InboundConfig};
pub use consumer::{EventConsumer, MessageHandler};
pub use producer::EventProducer;

use common_error::error::KafkaResult;

use async_trait::async_trait;

/// Simple placeholder struct, that allows you to create a heap-allocated instance of MessagePrinter wrapped in a Box.
/// This has been done this way so that the object has a stable memory address, and the ownership and allocation are
/// flexible
///
/// Using a box isn't completely necessary here, but if we are going to be storing large data later on or using
/// a polymorphic hierarchy

pub struct MessagePrinter {}

impl MessagePrinter {
    fn new() -> Box<Self> {
        Box::new(MessagePrinter {})
    }
}

/// Rust doesn't support async functions directly within a trait definition, due to lifetime constraints within the language.
/// The async_trait macro works around this. It will rewrite the trait and its impl in a way that makes it work with async methods.
///
/// the handle method has 2 params; key and payload. Represented as a byte slice
#[async_trait]
impl MessageHandler for MessagePrinter {
    async fn handle(&self, key: &[u8], payload: &[u8]) -> KafkaResult<()> {
        println!("Key: {}", String::from_utf8_lossy(key));
        println!("Payload: {}", String::from_utf8_lossy(payload));

        Ok(())
    }
}

/// This is the main function that processes the Kafka Messages concurrently.
#[tokio::main]
async fn main() -> KafkaResult<()> {
    // Initialise the tracing library for structured logging.

    tracing_subscriber::fmt::init();

    // inbound_config: initiates a new config for the inbound consumer.
    // fulfillment_config: initiates a new config for the fulfillment consumer.

    let inbound_config = InboundConfig::new();
    let fulfillment_config = FulfillmentConfig::new();

    // inbound_consumer: creates a new consumer for the Inbound Pipeline
    // fulfillment_consumer: creates a new consumer for the Fulfillment Pipeline.
    let inbound_consumer = EventConsumer::new(inbound_config, MessagePrinter::new())?;
    let fulfillment_consumer = EventConsumer::new(fulfillment_config, MessagePrinter::new())?;

    // try_join! executes multiple async tasks and waits for all of them to complete, if any ask returns an error, it stops and propagates the error to the caller
    tokio::try_join!(inbound_consumer.start(), fulfillment_consumer.start())?;

    Ok(())
}
