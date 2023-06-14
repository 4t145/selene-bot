use chrono::{DateTime, Utc};
use qqbot_sdk::model::{ChannelId, MessageId};

pub struct DeletedHistoryQuery {
    pub userid: i64,
    pub limit: i64,
    pub skip: i64,
    pub channel: Option<i64>,
    pub guild: Option<i64>,
}


pub struct DeletedHistoryQueryOutput {
    pub id: MessageId,
    pub channel_id: i64,
    pub guild_id: i64,
    pub time: DateTime<Utc>
}