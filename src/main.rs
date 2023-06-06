use qqbot_sdk::{bot::*, http::api::*, websocket::*};

use crate::handlers::influx_writer::{MessageWriter, InfluxDbClient};
mod handlers;
mod configs;
use configs::*;

#[tokio::main]
async fn main() -> Result<(), BotError> {
    let bot_config = BotConfig::load_from_file(config_path());
    // 启动webapi client
    let auth = Authority::Bot {
        app_id: &bot_config.sdk.app_id,
        token: &bot_config.sdk.token,
    };
    let message_writer = MessageWriter {
        client: InfluxDbClient::new(&bot_config.influxdb.url, &bot_config.influxdb.database),
    };
    let bot = BotBuilder::default()
        .auth(auth)
        .intents(Intends::PUBLIC_GUILD_MESSAGES | Intends::GUILD_MESSAGE_REACTIONS | Intends::GUILD_MESSAGES )
        .start()
        .await
        .unwrap();
    log::info!("bot: {:?}", bot.about_me().await?);
    bot.fetch_my_guilds().await?;
    log::info!("guilds count: {:?}", bot.cache().get_guilds_count().await);
    bot.register_handler("influx::write::message", message_writer).await;
    // wait for ctrl-c
    // bot.await;
    // wait for ctrl-c
    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
}
