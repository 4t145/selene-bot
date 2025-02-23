use qqbot_sdk::bot;
use rig::agent::Agent;

use crate::handlers::{commands_handler::CommandsHandler, surreal_writer::SurrealWriter};
mod configs;
mod error;
mod handlers;
use crate::error::*;
use configs::*;
mod clients;
mod cmd;
mod model;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let bot_config = crate::BotConfig::load_from_file(config_path());
    // 启动webapi client

    let surreal_writer = SurrealWriter::init(&bot_config.surreal).await?;
    let commands_handler = CommandsHandler::init(&bot_config).await?;
    let bot = bot::Bot::new(bot::BotConfig {
        app_id: bot_config.sdk.app_id.clone(),
        secret: bot_config.sdk.secret.clone(),
        base_url: qqbot_sdk::consts::SANDBOX_DOMAIN.to_string(),
    });
    log::info!("bot: {:?}", bot.about_me().await?);
    bot.fetch_my_guilds().await?;
    log::info!("guilds count: {:?}", bot.cache().get_guilds_count().await);
    bot.event_service()
        .spawn_handler("surreal::writter", surreal_writer)
        .await;
    bot.event_service()
        .spawn_handler("commands_handler", commands_handler)
        .await;
    use rig::{
        agent::{Agent, AgentBuilder},
        providers::deepseek::Client,
    };
    let model = Client::new("test").completion_model("r1");
    let agent = AgentBuilder::new(model).build();
    // wait for ctrl-c
    // bot.await;
    // wait for ctrl-c
    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
}

use rig::tool::Tool;