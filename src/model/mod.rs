use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub mod command;
pub mod query;

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernameHistory {
    pub userid: i64,
    pub username: String,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeletedHistory {
    pub message_id: String,
    pub channel_id: i64,
    // pub time: DateTime<Utc>,
}
