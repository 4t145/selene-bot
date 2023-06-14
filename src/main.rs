#![feature(async_fn_in_trait)]
use qqbot_sdk::{bot::*, http::api::*, websocket::*};

use crate::handlers::{
    commands_handler::CommandsHandler,
    surreal_writer::SurrealWriter,
};
mod configs;
mod error;
mod handlers;
use crate::error::*;
use configs::*;
mod model;
mod clients;
mod cmd;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let bot_config = BotConfig::load_from_file(config_path());
    // 启动webapi client
    let auth = Authority::Bot {
        app_id: &bot_config.sdk.app_id,
        token: &bot_config.sdk.token,
    };
    let surreal_writer = SurrealWriter::init(&bot_config.surreal).await?;
    let commands_handler = CommandsHandler::init(&bot_config).await?;
    let bot = BotBuilder::default()
        .auth(auth)
        .intents(
            Intends::PUBLIC_GUILD_MESSAGES
                | Intends::GUILD_MESSAGE_REACTIONS
                | Intends::GUILD_MESSAGES,
        )
        .start()
        .await
        .unwrap();
    log::info!("bot: {:?}", bot.about_me().await?);
    bot.fetch_my_guilds().await?;
    log::info!("guilds count: {:?}", bot.cache().get_guilds_count().await);
    bot.register_handler("surreal::writter", surreal_writer)
        .await;
    bot.register_handler("commands_handler", commands_handler)
        .await;

    // wait for ctrl-c
    // bot.await;
    // wait for ctrl-c
    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
}
