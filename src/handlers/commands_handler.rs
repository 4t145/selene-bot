use std::{str::FromStr, sync::Arc};

use chrono::{DateTime, Utc};
use clap::Parser;
use qqbot_sdk::{
    bot::message::MessageBuilder,
    event::{handler::EventHandler, model::Event},
    model::*,
};
use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
};

use crate::{configs::BotConfig, model::*};

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

pub trait CommandExecutable {
    type Output;
    async fn execute_on(&self, executor: &CommandsHandlerInner) -> crate::Result<Self::Output>;
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
    pub async fn get_deleted_history(
        &self,
        query: command::DeletedHistoryQueryCmd,
    ) -> crate::Result<Vec<(DateTime<Utc>, String)>> {
        let db = &self.surreal;
        let mut history = db
            .query("SELECT  FROM deleted_history WHERE message.author.id=$userid ORDER BY time DESC SKIP $skip LIMIT $limit")
            .bind(("userid", query.user))
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
    async fn handler_command(&self, cmd: MessageContent) -> String {
        // first element is at me
        let mut segs = Vec::new();
        let mut iter = cmd.segments.iter();
        let Some(MessageSegment::At(_me)) = iter.next() else {
            return String::from("parse error");
        };
        segs.push("@<this—bot>".to_owned());
        for seg in iter {
            match seg {
                MessageSegment::Text(s) => {
                    segs.extend(s.split_whitespace().map(String::from));
                }
                MessageSegment::At(user) => segs.push(user.to_string()),
                MessageSegment::AtAll => segs.push("all".to_string()),
                MessageSegment::Channel(id) => segs.push(id.to_string()),
                _ => {}
            }
        }
        use crate::cmd::*;
        match crate::cmd::Command::try_parse_from(segs) {
            Ok(command) => match command {
                Command::DeletedHistoryQuery(q) => match q.execute_on(self).await {
                    Ok(hs) => {
                        let mut reply = MessageContent::new();
                        reply.text(format!(
                            "{message_id_title:20}|{time_title}\n",
                            message_id_title = "message_id",
                            time_title = "channel_id"
                        ));
                        for DeletedHistory {
                            message_id,
                            channel_id,
                        } in hs
                        {
                            reply.text(format!("{message_id:20}|{channel_id}\n"));
                        }
                        reply.to_string()
                    }
                    Err(e) => e.to_string(),
                },
                Command::UsernameHistoryQuery(q) => match q.execute_on(self).await {
                    Ok(hs) => {
                        let mut reply = MessageContent::new();
                        reply.text(format!(
                            "{username_title:20}|{time_title}\n",
                            username_title = "username",
                            time_title = "record time"
                        ));
                        for UsernameHistory { username, time, .. } in hs {
                            reply.text(format!("{username:20}|{time}\n"));
                        }
                        reply.to_string()
                    }
                    Err(e) => e.to_string(),
                },
            },
            Err(e) => e.to_string(),
        }
    }
}

impl EventHandler for CommandsHandler {
    fn would_handle(&self, event: &Event, bot: &qqbot_sdk::bot::Bot<()>) -> bool {
        if let Event::AtMessageCreate(msg) = event {
            msg.content.starts_with("@<this—bot>")
        } else {
            false
        }
    }
    async fn handle(
        &self,
        event: Event,
        bot: &qqbot_sdk::bot::Bot,
    ) -> Result<(), qqbot_sdk::Error> {
        #[allow(clippy::single_match)]
        match event {
            Event::AtMessageCreate(msg) => {
                let handler = self.clone();

                let segs = MessageContent::from_str(&msg.content)
                    .map_err(|_| qqbot_sdk::Error::unexpected("parse message content error"))?;
                let reply = handler.handler_command(segs).await;
                bot.send_message(
                    msg.channel_id,
                    &MessageBuilder::default()
                        .content(reply.as_str())
                        .reply_to(msg.as_ref())
                        .build(),
                )
                .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
