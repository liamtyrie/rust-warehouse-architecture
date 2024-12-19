use csv::ReaderBuilder;
use inbound_outbox::Outbox;
use model::{CsvRow, RowWithUser};
use rand::seq::SliceRandom;
use rand::Rng;
use std::time::Duration;
use tokio::time::sleep;
pub mod model;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let outbox = Outbox::new("mongodb://localhost:27017", "warehouse", "inbound_outbox").await?;
    let pending_entries = outbox.fetch_pending_entries().await?;

    for entry in pending_entries {
        println!("Pending Entry: {:?}", entry);
    }

    let user_ids = vec![
        16349, 17899, 17455, 20778, 13447, 20865, 17745, 18752, 16334, 16577, 17889, 13887, 12227,
    ];

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("inbound-stock.csv")?;

    let csv_rows: Vec<CsvRow> = reader.deserialize().collect::<Result<_, _>>()?;

    if csv_rows.is_empty() {
        println!("No data found in the CSV file");
        return Ok(());
    }

    for user_id in user_ids.iter() {
        println!("Processing for userId: {}", user_id);

        let mut rng = rand::thread_rng();
        let row_count: usize = rng.gen_range(5..=10);
        let selected_rows = csv_rows.choose_multiple(&mut rng, row_count);

        for row in selected_rows {
            let row_with_user = RowWithUser {
                user_id: *user_id,
                shipment_id: row.shipment_id,
                product_id: row.product_id,
                quantity: row.quantity,
            };

            let payload = serde_json::to_string(&row_with_user)?;
            let outbox_id = outbox.add_entry(*user_id, payload.clone()).await?;
            println!("Outbox entry created with ID: {}", outbox_id);

            let delay = rng.gen_range(3000..=9000);
            sleep(Duration::from_millis(delay)).await;
        }

        println!("Finished processing for userId: {}\n", user_id);
    }

    Ok(())
}
