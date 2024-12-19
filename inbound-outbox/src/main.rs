use std::time::Duration;

use chrono::{DateTime, Utc};
use common_kafka::{config::InboundConfig, EventProducer};
use inbound_outbox::Outbox;
use mongodb::{bson::doc, Client};
use serde::{Deserialize, Serialize};
use tokio::task;
use tokio::time::interval;
use tokio_stream::StreamExt;

#[derive(Debug, Deserialize)]
struct OutboxEntry {
    #[serde(rename = "_id")]
    id: mongodb::bson::oid::ObjectId,
    user_id: u64,
    payload: String,
    status: String,
}
#[derive(Debug, Deserialize, Serialize)]
struct InboundEntry {
    #[serde(rename = "_id")]
    id: mongodb::bson::oid::ObjectId,
    user_id: u64,
    shipment_id: u64,
    product_id: u64,
    quantity: u32,
    created_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    let db = client.database("warehouse");
    let collection = db.collection::<OutboxEntry>("inbound_outbox");
    let outbox = Outbox::new("mongodb://localhost:27017", "warehouse", "inbound_outbox").await?;

    let kafka_producer = EventProducer::new(InboundConfig::new())?;

    // Spawn a task for periodically fetching pending entries
    let outbox_task = {
        let collection = collection.clone();
        let outbox = outbox.clone();
        let kafka_producer = kafka_producer.clone();

        task::spawn(async move {
            let mut fetch_interval = interval(Duration::from_secs(10)); // Adjust as needed

            loop {
                fetch_interval.tick().await;

                match outbox.fetch_pending_entries().await {
                    Ok(pending_entries) => {
                        for entry in pending_entries {
                            println!("Processing pending entry: {:?}", entry);

                            let result = kafka_producer
                                .send_event(&entry.user_id.to_string(), &entry.payload)
                                .await;

                            match result {
                                Ok(_) => {
                                    if let Some(id) = entry.id {
                                        if let Err(err) = collection
                                            .update_one(
                                                doc! { "_id": id },
                                                doc! { "$set": { "status": "sent" } },
                                            )
                                            .await
                                        {
                                            eprintln!("Failed to update entry: {:?}", err);
                                        } else {
                                            println!("Marked as sent: {:?}", id);
                                        }
                                    }
                                }
                                Err(err) => {
                                    eprintln!("Failed to send pending Kafka message: {:?}", err);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to fetch pending entries: {:?}", err);
                    }
                }
            }
        })
    };

    // Main loop for processing change stream events
    let mut change_stream = collection.watch().await?;

    while let Some(change) = change_stream.next().await {
        if let Ok(change_event) = change {
            if let Some(full_doc) = change_event.full_document {
                if full_doc.status == "pending" {
                    let result = kafka_producer
                        .send_event(&full_doc.user_id.to_string(), &full_doc.payload)
                        .await;

                    match result {
                        Ok(_) => {
                            collection
                                .update_one(
                                    doc! { "_id": full_doc.id },
                                    doc! { "$set": { "status": "sent" } },
                                )
                                .await?;
                            println!("Successfully sent and marked as sent: {:?}", full_doc.id);
                        }
                        Err(err) => {
                            eprintln!("Failed to send Kafka message: {:?}", err);
                        }
                    }
                }
            }
        }
    }

    // Wait for the outbox task to complete (though it never will in this design)
    outbox_task.await?;

    Ok(())
}
