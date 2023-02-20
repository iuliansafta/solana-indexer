use chrono::NaiveDateTime;
use postgres::{Client, Error, NoTls};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use indexer::rpc_client::{
    get_balance_at_signature, get_signatures_for_address, get_transaction_by_signature, rpc_client,
};

struct Address {
    id: i64,
    address: String,
}

#[derive(Debug)]
struct BalanceParams {
    id: i64,
    block_time: NaiveDateTime,
    post_balance: i64,
    pre_balance: i64,
    fee: i64,
    transfer_type: i16,
    transaction_has: String,
    // address: String,
}

fn main() -> Result<(), Error> {
    dotenv::dotenv().expect("Failed to read .env file");

    let sys_time = SystemTime::now();
    println!(
        "Querying addresses: {}",
        sys_time.duration_since(UNIX_EPOCH).unwrap().as_millis()
    );

    let mut addresses_to_query: Vec<Address> = Vec::new();
    let mut client = Client::connect(&std::env::var("DB_STRING").unwrap(), NoTls)?;

    match client.query(
        "SELECT * FROM solana_addresses WHERE inspected_on is null",
        &[],
    ) {
        Ok(rows) => {
            for row in rows {
                let adr = Address {
                    id: row.get(0),
                    address: row.get(1),
                };
                addresses_to_query.push(adr);
            }
        }
        Err(_) => {}
    }

    for token in addresses_to_query {
        println!("Addresses to query {:?}", token.address);
        parse_transactions_by_token(token, &mut client);
    }

    Ok(())
}

fn parse_transactions_by_token(token: Address, client: &mut Client) {
    let connection = rpc_client();
    let token_address = Pubkey::from_str(&token.address).unwrap();

    let signs_by_token = get_signatures_for_address(&connection, token_address, 1000);

    for signature in signs_by_token {
        let encoded_transaction = get_transaction_by_signature(&connection, signature.clone());
        let transaction_meta = encoded_transaction.transaction.meta.unwrap();
        println!("fee: {:?}", transaction_meta.fee);

        let balance =
            get_balance_at_signature(&connection, token_address, signature.clone()).unwrap();

        let trans_block_time = encoded_transaction.block_time.unwrap() as i64;
        println!("block time: {:?}", trans_block_time);

        let block_time: NaiveDateTime =
            NaiveDateTime::from_timestamp_opt(trans_block_time, 0).expect("cicic");

        let params: BalanceParams = BalanceParams {
            id: token.id,
            block_time: block_time,
            post_balance: balance.balance_after as i64,
            pre_balance: balance.balance_before as i64,
            fee: transaction_meta.fee as i64,
            transfer_type: balance.transaction_type,
            transaction_has: signature,
        };

        insert_balance(&mut *client, &params);

        println!("Inserted params {:?}", params);
    }
}

fn insert_balance(client: &mut Client, params: &BalanceParams) {
    println!("params to insert {:?}", params);

    match client
        .execute(
            "INSERT INTO balances(address_id, pre_balance, post_balance, fee, transaction_has, transfer_type, block_time) VALUES($1,$2,$3,$4,$5,$6,$7)",
            &[&params.id, &params.pre_balance, &params.post_balance, &params.fee, &params.transaction_has, &params.transfer_type, &params.block_time],
        )
        {
            Ok(_) => {
                update_token_address(client, params.id)
            }
            Err(err) => println!("ERROR inserting balance: {:?}", err)
        }
}

fn update_token_address(client: &mut Client, id: i64) {
    match client.execute(
        "UPDATE solana_addresses SET inspected_on = now() WHERE id=$1",
        &[&id],
    ) {
        Ok(_) => {}
        Err(err) => println!("ERROR update token address: {:?}", err),
    }
}
