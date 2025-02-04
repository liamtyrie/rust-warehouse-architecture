use std::time::Duration;

use chrono::{DateTime, Utc};
use common_kafka::{config::InboundConfig, EventProducer};
use inbound_outbox::Outbox;
use mongodb::{bson::doc, Client};
use serde::{Deserialize, Serialize};
use tokio::task;
use tokio::time::{interval, sleep};
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

async fn send_with_retry(
    producer: &EventProducer,
    key: &str,
    payload: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        match producer.send_event(key, payload).await {
            Ok(_) => return Ok(()),
            Err(err) => {
                if retries >= 3 {
                    return Err(err.into());
                }
                retries += 1;
                sleep(Duration::from_secs(2u64.pow(retries))).await;
            }
        }
    }
}

async fn process_pending_entries(
    outbox: Outbox,
    collection: mongodb::Collection<OutboxEntry>,
    kafka_producer: EventProducer,
) {
    let mut fetch_interval = interval(Duration::from_secs(10));

    loop {
        fetch_interval.tick().await;

        match outbox.fetch_pending_entries().await {
            Ok(pending_entries) => {
                for entry in pending_entries {
                    println!("Processing pending entry: {:?}", entry);

                    let filter = doc! { "_id": entry.id, "status": "pending"};
                    let update = doc! { "$set": { "status": "processing"}};

                    if let Ok(Some(claimed_entry)) =
                        collection.find_one_and_update(filter, update).await
                    {
                        if let Err(err) = send_with_retry(
                            &kafka_producer,
                            &claimed_entry.user_id.to_string(),
                            &claimed_entry.payload,
                        )
                        .await
                        {
                            eprintln!("Failed to send Kafka Message: {:?}", err);
                            continue;
                        }

                        if let Err(err) = collection
                            .update_one(
                                doc! { "_id": claimed_entry.id},
                                doc! {"$set": { "status": "sent"}},
                            )
                            .await
                        {
                            eprintln!("Failed to update entry: {:?}", err);
                        } else {
                            println!("Marked as sent: {:?}", claimed_entry.id)
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to fetch pending entries: {:?}", err);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    let db = client.database("warehouse");
    let collection = db.collection::<OutboxEntry>("inbound_outbox");
    let outbox = Outbox::new("mongodb://localhost:27017", "warehouse", "inbound_outbox").await?;

    let kafka_producer = EventProducer::new(InboundConfig::new())?;

    // Spawn a task for periodically fetching pending entries
    let outbox_task = task::spawn(process_pending_entries(
        outbox.clone(),
        collection.clone(),
        kafka_producer.clone(),
    ));

    let mut change_stream = collection.watch().await?;

    while let Some(change) = change_stream.next().await {
        if let Ok(change_event) = change {
            if let Some(full_doc) = change_event.full_document {
                if full_doc.status == "pending" {
                    let filter = doc! { "_id": full_doc.id, "status": "pending"};
                    let update = doc! {"$set": {"status": "processing"}};

                    if let Ok(Some(claimed_entry)) =
                        collection.find_one_and_update(filter, update).await
                    {
                        if let Err(err) = send_with_retry(
                            &kafka_producer,
                            &claimed_entry.user_id.to_string(),
                            &claimed_entry.payload,
                        )
                        .await
                        {
                            eprintln!("Failed to send Kafka Message: {:?}", err);
                            continue;
                        }

                        if let Err(err) = collection
                            .update_one(
                                doc! { "_id": claimed_entry.id},
                                doc! { "$set": { "status": "sent"}},
                            )
                            .await
                        {
                            eprintln!("Failed to update entry {:?}", err);
                        } else {
                            println!(
                                "Successfully sent and marked as sent: {:?}",
                                claimed_entry.id
                            );
                        }
                    }
                }
            }
        }
    }

    outbox_task.await?;

    Ok(())
}
