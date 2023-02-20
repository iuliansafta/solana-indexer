use solana_client::rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use std::str::FromStr;

pub fn rpc_client() -> RpcClient {
    RpcClient::new(std::env::var("SOLANA_RPC_ENDPOINT").unwrap().to_string())
}

pub fn get_signatures_for_address(
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

pub fn get_transaction_by_signature(
    connection: &RpcClient,
    signature: String,
) -> EncodedConfirmedTransactionWithStatusMeta {
    let signature = Signature::from_str(&signature).unwrap();

    let transaction_with_meta = connection
        .get_transaction(&signature, UiTransactionEncoding::Binary)
        .unwrap();

    transaction_with_meta
}

#[derive(Debug)]
pub struct Balance {
    pub balance_before: u64,
    pub balance_after: u64,
    pub transaction_type: i16,
}

pub fn get_balance_at_signature(
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
