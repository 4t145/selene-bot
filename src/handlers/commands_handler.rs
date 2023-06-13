use std::{str::FromStr, sync::Arc};

use chrono::{Utc, DateTime};
use qqbot_sdk::{
    bot::{Handler, MessageBuilder},
    model::*,
};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal
};

use crate::{configs::BotConfig, model::{UsernameHistory, DeletedHistoryQuery}};

pub enum Commands {
    QueryUser { username: String },
    Hello,
}

impl FromStr for Commands {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut segs = s.split(' ');
        let Some(cmd) = segs.next() else {
            return Err("empty input".to_string());
        };
        match cmd {
            "/query_user" => {
                let username = segs.next().ok_or("username not found")?.to_owned();

                Ok(Commands::QueryUser { username })
            }
            "/hello" => Ok(Commands::Hello),
            _ => Err(format!("unknown command: {}", cmd)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandsHandler(pub Arc<CommandsHandlerInner>);

impl std::ops::Deref for CommandsHandler {
    type Target = CommandsHandlerInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct CommandsHandlerInner {
    pub surreal: Surreal<Client>,
}
impl CommandsHandler {
    pub async fn init(config: &BotConfig) -> crate::Result<Self> {
        CommandsHandlerInner::init(config)
            .await
            .map(|x| CommandsHandler(Arc::new(x)))
    }
    pub async fn get_user_history(&self, user_id: u64) -> crate::Result<Vec<UsernameHistory>> {
        let db = &self.surreal;
        let mut history = db
            .query("SELECT * FROM username_history WHERE userid=$userid ORDER BY time DESC")
            .bind(("userid", user_id))
            .await?;
        let historys = history.take::<Vec<UsernameHistory>>(0)?;
        Ok(historys)
    }
    pub async fn get_deleted_history(&self, query: DeletedHistoryQuery) -> crate::Result<Vec<(DateTime<Utc>, String)>> {
        let db = &self.surreal;
        let mut history = db
            .query("SELECT  FROM deleted_history WHERE message.author.id=$userid ORDER BY time DESC SKIP $skip LIMIT $limit")
            .bind(("userid", query.userid))
            .bind(("skip", query.skip))
            .bind(("limit", query.limit))
            .await?;
        let historys = history.take::<Vec<(DateTime<Utc>, String)>>(0)?;
        Ok(historys)
    }
    // pub async fn get_userid_by_name(&self, username: &str) -> crate::Result<Option<u64>> {
    //     let db = &self.surreal;
    //     let mut history = db
    //         .query("SELECT id FROM message WHERE author.username=$username ORDER BY time DESC").bind(("username", username))
    //         .await?;
    //     let historys = history.take::<Option<String>>(0)?;
    //     Ok(historys)
    // }
}
impl CommandsHandlerInner {
    pub async fn init(config: &BotConfig) -> crate::Result<Self> {
        let surreal_cfg = &config.surreal;
        let surreal = Surreal::new::<Ws>(format!(
            "{host}:{port}",
            host = &surreal_cfg.host,
            port = &surreal_cfg.port
        ))
        .await?;

        surreal
            .signin(Root {
                username: &surreal_cfg.username,
                password: &surreal_cfg.password,
                // namespace: &config.namespace,
            })
            .await?;

        surreal
            .use_ns(&config.surreal.namespace)
            .use_db("selene_main")
            .await?;

        Ok(CommandsHandlerInner { surreal })
    }
}

impl CommandsHandler {
    async fn handler_command(&self, cmd: MessageContent) -> Option<String> {
        // first element is at me
        let mut segs = cmd.segments.iter().skip(1);
        if let Some(MessageSegment::Text(cmd)) = segs.next() {
            let cmd = cmd.trim_start();
            match cmd.split(' ').next() {
                Some("这个人也帮我查一下") | Some("/query_user") => {
                    let Some(MessageSegment::At ( user_id ) ) = segs.next() else {
                        return Some("expect a user as input".to_string()) 
                    };
                    let history = self.get_user_history(*user_id).await;
                    match history {
                        Ok(history) => {
                            let mut reply = String::new();
                            for h in history {
                                reply.push_str(&format!(
                                    "username: {}, time: {}\n",
                                    h.username, h.time
                                ));
                            }
                            Some(reply)
                        }
                        Err(e) => Some(e.to_string()),
                    }
                }
                Some("/delete_history") => {
                    let Some(MessageSegment::At ( user_id ) ) = segs.next() else {
                        return Some("expect a user as input".to_string()) 
                    };
                    let history = self.get_user_history(*user_id).await;
                    match history {
                        Ok(history) => {
                            let mut reply = String::new();
                            for h in history {
                                reply.push_str(&format!(
                                    "username: {}, time: {}\n",
                                    h.username, h.time
                                ));
                            }
                            Some(reply)
                        }
                        Err(e) => Some(e.to_string()),
                    }
                }
                Some("/Hello") => Some("Hello, I'm Selene".to_string()),

                _ => None,
            }
        } else {
            None
        }
    }
}

impl Handler for CommandsHandler {
    fn handle(
        &self,
        event: qqbot_sdk::websocket::ClientEvent,
        ctx: std::sync::Arc<qqbot_sdk::bot::Bot>,
    ) -> Result<(), qqbot_sdk::bot::BotError> {
        #[allow(clippy::single_match)]
        match event {
            qqbot_sdk::websocket::ClientEvent::AtMessageCreate(msg) => {
                let handler = self.clone();
                tokio::spawn(async move {
                    let segs = MessageContent::from_str(&msg.content)?;
                    let Some(reply) = handler.handler_command(segs).await else {
                        return Ok(());
                    };
                    ctx.send_message(
                        msg.channel_id,
                        &MessageBuilder::default()
                            .content(reply.as_str())
                            .reply_to(msg.as_ref())
                            .build()?,
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                    Ok::<_, String>(())
                });
            }
            _ => {}
        }
        Ok(())
    }
}
