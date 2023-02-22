use serde::Deserialize;
use serde_json;
use sqlx::postgres::PgListener;
use std::str::FromStr;

use indexer::chains;

#[derive(Deserialize, Debug)]
enum ChainName {
    Solana,
    Eth,
    Egld,
}
#[derive(Deserialize, Debug)]
struct EventRecord {
    id: String,
    address: String,
    chain_name: ChainName,
}
#[derive(Deserialize, Debug)]
struct WalletChangedEvent {
    operation: String,
    record: EventRecord,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().expect("Failed to read .env file");

    let conn_str = std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required.");
    let pool = sqlx::PgPool::connect(&conn_str).await?;
    let mut listener = PgListener::connect_with(&pool).await?;
    println!("Building PG pool.");

    listener.listen("wallet_changed").await?;

    loop {
        let notification = listener.recv().await?;
        let event_record: WalletChangedEvent =
            serde_json::from_str(notification.payload()).unwrap();
        println!("[from recv]: {:?}", event_record);

        init_module(event_record);
    }
}

fn init_module(event: WalletChangedEvent) {
    let token = chains::solana::TokenAddress {
        id: &uuid::Uuid::from_str(&event.record.id.to_owned()).unwrap(),
        address: event.record.address,
    };

    match event.record.chain_name {
        ChainName::Solana => chains::solana::init_chain_with_token(&token),
        _ => println!("We do not support this chain"),
    }
}
