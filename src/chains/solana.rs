use crate::chains::types::BalanceParams;
use crate::db::insert_balance;
use chrono::NaiveDateTime;
use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use sqlx::PgPool;
use std::str::FromStr;

#[derive(Debug)]
pub struct TokenAddress<'a> {
    pub id: &'a uuid::Uuid,
    pub address: String,
}

#[derive(Debug)]
struct Balance {
    pub balance_before: u64,
    pub balance_after: u64,
    pub transaction_type: i16,
}

pub async fn init_chain_with_token(
    token: &TokenAddress<'_>,
    pool: &PgPool,
) -> anyhow::Result<(), anyhow::Error> {
    let connection = RpcClient::new(std::env::var("SOLANA_RPC_ENDPOINT").unwrap().to_string());

    // println!("Init chain connection with pub key {:?}", token);
    parse_transactions_by_token(&connection, &token, &pool).await?;

    Ok(())
}

async fn parse_transactions_by_token(
    connection: &RpcClient,
    token: &TokenAddress<'_>,
    pool: &PgPool,
) -> anyhow::Result<(), anyhow::Error> {
    let token_address = Pubkey::from_str(&token.address).unwrap();

    let signs_by_token = get_signatures_for_address(&connection, token_address, 1000);

    for signature in signs_by_token {
        let encoded_transaction = get_transaction_by_signature(&connection, signature.clone());
        let transaction_meta = encoded_transaction.transaction.meta.unwrap();

        let balance =
            get_balance_at_signature(&connection, token_address, signature.clone()).unwrap();

        let trans_block_time = encoded_transaction.block_time.unwrap() as i64;
        let block_time: NaiveDateTime =
            NaiveDateTime::from_timestamp_opt(trans_block_time, 0).expect("Date conversion error");

        let params: BalanceParams = BalanceParams {
            id: token.id,
            block_time,
            post_balance: balance.balance_after as i64,
            pre_balance: balance.balance_before as i64,
            fee: transaction_meta.fee as i64,
            transfer_type: balance.transaction_type,
            transaction_has: signature,
        };

        insert_balance(pool, &params).await?
    }

    Ok(())
}

fn get_signatures_for_address(
    connection: &RpcClient,
    address: Pubkey,
    max_length: usize,
) -> Vec<String> {
    let mut signatures = Vec::new();

    let _signatures = connection.get_signatures_for_address(&address).unwrap();

    for sig in _signatures {
        signatures.push(sig.signature);
    }

    loop {
        if signatures.len() > 1000 && signatures.len() < max_length {
            let sign = Signature::from_str(signatures.last().unwrap()).ok();
            let config = GetConfirmedSignaturesForAddress2Config {
                before: sign,
                until: None,
                limit: Some(1000),
                commitment: Some(CommitmentConfig::confirmed()),
            };
            let _signatures = connection
                .get_signatures_for_address_with_config(&address, config)
                .unwrap();
            for sig in _signatures {
                signatures.push(sig.signature);
            }
        }
        break;
    }

    signatures
}

fn get_transaction_by_signature(
    connection: &RpcClient,
    signature: String,
) -> EncodedConfirmedTransactionWithStatusMeta {
    let signature = Signature::from_str(&signature).unwrap();

    let transaction_with_meta = connection
        .get_transaction(&signature, UiTransactionEncoding::Binary)
        .unwrap();

    transaction_with_meta
}

fn get_balance_at_signature(
    connection: &RpcClient,
    address: Pubkey,
    signature: String,
) -> Result<Balance, std::io::Error> {
    let encoded_transaction = get_transaction_by_signature(connection, signature);
    let meta_transaction = encoded_transaction.transaction.meta.unwrap();
    let decoded_transactions = encoded_transaction
        .transaction
        .transaction
        .decode()
        .unwrap();

    for (index, account_key) in decoded_transactions
        .message
        .static_account_keys()
        .iter()
        .enumerate()
    {
        if account_key.to_string() == address.to_string() {
            let balance_after = meta_transaction.post_balances[index];
            let balance_before = meta_transaction.pre_balances[index];
            let transaction_type = if balance_before < balance_after { 1 } else { 2 };

            return Ok(Balance {
                balance_after,
                balance_before,
                transaction_type,
            });
        }
    }

    Ok(Balance {
        balance_after: 0,
        balance_before: 0,
        transaction_type: 0,
    })
}
