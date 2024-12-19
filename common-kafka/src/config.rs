pub trait KafkaConfigTrait {
    fn brokers(&self) -> &str;
    fn topic(&self) -> &str;
    fn group_id(&self) -> &str;
    fn timeout_ms(&self) -> u64;
    fn max_retries(&self) -> u32;
}

#[derive(Debug, Clone)]
pub struct InboundConfig {
    pub brokers: String,
    pub topic: String,
    pub group_id: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

impl KafkaConfigTrait for InboundConfig {
    fn brokers(&self) -> &str {
        &self.brokers
    }
    fn topic(&self) -> &str {
        &self.topic
    }
    fn group_id(&self) -> &str {
        &self.group_id
    }
    fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

#[derive(Debug, Clone)]
pub struct FulfillmentConfig {
    pub brokers: String,
    pub topic: String,
    pub group_id: String,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

impl KafkaConfigTrait for FulfillmentConfig {
    fn brokers(&self) -> &str {
        &self.brokers
    }
    fn topic(&self) -> &str {
        &self.topic
    }
    fn group_id(&self) -> &str {
        &self.group_id
    }
    fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl InboundConfig {
    pub fn new() -> Self {
        Self {
            brokers: "localhost:9092".to_string(),
            topic: "INBOUND".to_string(),
            group_id: "INBOUND_GROUP".to_string(),
            timeout_ms: 5000,
            max_retries: 5,
        }
    }
}

impl FulfillmentConfig {
    pub fn new() -> Self {
        Self {
            brokers: "localhost:9092".to_string(),
            topic: "FULFILLMENT".to_string(),
            group_id: "FULFILLMENT_GROUP".to_string(),
            timeout_ms: 5000,
            max_retries: 5,
        }
    }
}
