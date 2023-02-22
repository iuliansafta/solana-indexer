use crate::chains::types::BalanceParams;
use sqlx::postgres::PgPool;
use std::env;

pub async fn insert_balance(pool: &PgPool, params: &BalanceParams<'_>) -> anyhow::Result<()> {
    println!("params to insert {:?}", params);

    let _rec = sqlx::query!(
        r#"
        INSERT INTO balances(address_id, pre_balance, post_balance, fee, transaction_hash, transfer_type, block_time)
        VALUES($1,$2,$3,$4,$5,$6,$7)
        "#,
        params.id,
        params.pre_balance,
        params.post_balance,
        params.fee,
        params.transaction_has,
        params.transfer_type,
        params.block_time
    )
    .fetch_one(pool)
    .await?;

    Ok(())
}

// pub fn update_token_address(client: &mut Client, id: i64) {
//     match client.execute("UPDATE wallets SET fetched_on = now() WHERE id=$1", &[&id]) {
//         Ok(_) => {}
//         Err(err) => println!("ERROR update token address: {:?}", err),
//     }
// }
