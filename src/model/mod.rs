use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub mod query;
#[derive(Debug, Serialize, Deserialize)]
pub struct UsernameHistory {
    pub userid: i64,
    pub username: String,
    pub time: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DeletedHistoryQuery {
    pub userid: u64,
    pub skip: usize,
    pub limit: usize,
}

