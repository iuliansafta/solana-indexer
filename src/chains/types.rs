use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug)]
pub struct BalanceParams<'a> {
    pub id: &'a Uuid,
    pub block_time: NaiveDateTime,
    pub post_balance: i64,
    pub pre_balance: i64,
    pub fee: i64,
    pub transfer_type: i16,
    pub transaction_has: String,
}
