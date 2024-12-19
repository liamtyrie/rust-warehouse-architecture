use chrono::{DateTime, Utc};
use common_kafka::EventProducer;
use futures_util::TryStreamExt;
use mongodb::{bson::doc, options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct OutboxEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub user_id: u64,
    pub payload: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
#[derive(Clone)]
pub struct Outbox {
    collection: Collection<OutboxEntry>,
}

impl Outbox {
    pub async fn new(
        uri: &str,
        db_name: &str,
        collection_name: &str,
    ) -> mongodb::error::Result<Self> {
        let options = ClientOptions::parse(uri).await?;
        let client = Client::with_options(options)?;
        let collection = client
            .database(db_name)
            .collection::<OutboxEntry>(collection_name);
        Ok(Self { collection })
    }

    pub async fn add_entry(
        &self,
        user_id: u64,
        payload: String,
    ) -> mongodb::error::Result<mongodb::bson::oid::ObjectId> {
        let entry = OutboxEntry {
            id: None,
            user_id,
            payload,
            status: "pending".to_string(),
            created_at: Utc::now(),
        };

        let result = self.collection.insert_one(entry).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn mark_sent(&self, id: mongodb::bson::oid::ObjectId) -> mongodb::error::Result<()> {
        self.collection
            .update_one(doc! { "_id": id}, doc! { "$set": { "status": "sent "}})
            .await?;
        Ok(())
    }
}

impl Outbox {
    pub async fn fetch_pending_entries(&self) -> mongodb::error::Result<Vec<OutboxEntry>> {
        let cursor = self.collection.find(doc! { "status": "pending"}).await?;

        cursor.try_collect().await
    }
}

pub async fn retry_pending_entries(outbox: Arc<Outbox>, kafka_producer: Arc<EventProducer>) {
    let mut retry_interval = interval(Duration::from_secs(60));

    loop {
        retry_interval.tick().await;

        let pending_entries = match outbox.fetch_pending_entries().await {
            Ok(entries) => entries,
            Err(err) => {
                eprintln!("Failed to fetch pending entries: {:?}", err);
                continue;
            }
        };

        if pending_entries.is_empty() {
            println!("No pending entries to retry");
            continue;
        }

        for entry in pending_entries {
            println!("Retrying entry with ID: {:?}", entry.id);

            match kafka_producer
                .send_event(entry.user_id.to_string(), &entry.payload)
                .await
            {
                Ok(_) => {
                    if let Some(entry_id) = entry.id {
                        if let Err(err) = outbox.mark_sent(entry_id).await {
                            eprintln!(
                                "Failed to update status for entry ID {:?}: {:?}",
                                entry_id, err
                            );
                        } else {
                            println!("Successfully marked entry as sent: {:?}", entry_id);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Failed to send entry to Kafka: {:?}", err);
                }
            }
        }
    }
}
