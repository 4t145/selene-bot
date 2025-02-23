use std::collections::HashMap;

use crate::configs::SurrealConfig;
use crate::error::*;
use crate::model::*;
use chrono::{DateTime, Utc};
use qqbot_sdk::bot::Bot;
use qqbot_sdk::event::handler::EventHandler;
use qqbot_sdk::event::model::Event;
use qqbot_sdk::model::*;

use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::{Root};

use surrealdb::Surreal;

#[derive(Debug)]
pub struct SurrealWriter {
    db: Surreal<Client>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRecord {}

impl SurrealWriter {
    pub async fn init(config: &SurrealConfig) -> Result<Self> {
        let db = Surreal::new::<Ws>(format!(
            "{host}:{port}",
            host = &config.host,
            port = &config.port
        ))
        .await?;

        db.signin(Root {
            username: &config.username,
            password: &config.password,
            // namespace: &config.namespace,
        })
        .await?;

        db.use_ns(&config.namespace).use_db("selene_main").await?;

        Ok(SurrealWriter { db })
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageSurreal {
    #[serde(flatten)]
    _inner: MessageBotRecieved,
    reactions: HashMap<String, u16>,
}

impl From<&MessageBotRecieved> for MessageSurreal {
    fn from(val: &MessageBotRecieved) -> Self {
        MessageSurreal {
            _inner: val.clone(),
            reactions: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MessgaeRecord {}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Empty;


#[derive(Debug, Serialize, Deserialize)]
pub struct UserAvatarHistory {
    pub userid: u64,
    pub avatar: String,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct QueryUsernameHistory {
    pub userid: u64,
}

#[derive(Debug, Deserialize)]
pub struct QueryUsernameResult {
    pub username: String,
    pub time: DateTime<Utc>,
}

impl EventHandler for SurrealWriter {
    fn would_handle(&self, event: &Event, bot: &Bot) -> bool {
        match event {
            Event::MessageCreate(_) => true,
            Event::PublicMessageDelete(_) => true,
            _ => false,
        }
    }
    async fn handle(
        &self,
        event: Event,
        _bot: &Bot,
    ) -> std::result::Result<(), qqbot_sdk::Error> {
        #[allow(clippy::single_match)]
        match event {
            Event::MessageCreate(message) => {
                let db = self.db.clone();
                    let _: Option<Empty> = db
                        .create(("message", &message.id.to_string()))
                        .content(message)
                        .await
                        .unwrap_or_default();
                    let userid = message.author.id;
                    let username = &message.author.username;
                    let avatar = &message.author.avatar;
                    // query recent record
                    if let Ok(mut resp) = db.query("SELECT * FROM username_history WHERE userid=$userid ORDER BY time DESC LIMIT 1").bind(("userid", userid)).await {
                        dbg!(&resp);
                        let update = if let Ok(Some(recent_record)) = resp.take::<Option<UsernameHistory>>(0)  {
                            &recent_record.username != username
                        } else {
                            true
                        };
                        dbg!(update);
                        if update {
                            let _: Option<Empty> = db.create("username_history").content(&UsernameHistory {
                                userid: userid as i64,
                                username: username.clone(),
                                time: message.timestamp,
                            }).await.unwrap_or_default();
                            
                        }
                    }
                    if let Ok(mut resp) = db.query("SELECT * FROM useravatar_history WHERE userid=$userid ORDER BY time DESC LIMIT 1").bind(("userid", userid)).await {
                        let update = if let Ok(Some(recent_record)) = resp.take::<Option<UserAvatarHistory>>(0)  {
                            recent_record.avatar != avatar.as_deref().unwrap_or_default()
                        } else {
                            false
                        };
                        if update {
                            let _: Option<Empty> = db.create("useravatar_history").content(&UserAvatarHistory {
                                userid: userid,
                                avatar: avatar.as_deref().unwrap_or_default().into(),
                                time: message.timestamp,
                            }).await.unwrap_or_default();
                        }
                    }
            }
            Event::PublicMessageDelete(delete) => {
                let db = self.db.clone();
                tokio::spawn(async move {
                    let _: Option<Empty> = db
                        .create(("message_delete", &delete.message.id.to_string()))
                        .content(&delete.as_ref().clone())
                        .await
                        .unwrap_or_default();
                    Ok::<(), String>(())
                });
            }
            _ => {}
        }
        Ok(())
    }
}
