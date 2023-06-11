use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernameHistory {
    pub userid: i64,
    pub username: String,
    pub time: DateTime<Utc>,
}
