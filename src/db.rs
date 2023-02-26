use crate::chains::types::BalanceParams;
use sqlx::postgres::PgPool;

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
    ).execute(pool).await?;

    // let _rec2 = sqlx::query!(
    //     r#"
    //     UPDATE wallets SET fetched_on = now() WHERE id=$1
    //     "#,
    //     params.id
    // )
    // .execute(pool)
    // .await?;

    Ok(())
}
