use clap::Parser;
use qqbot_sdk::model::{GuildId, ChannelId};

use crate::handlers::commands_handler::CommandExecutable;

use crate::model::{UsernameHistory, DeletedHistory};

#[derive(Debug, Parser)]
#[command(about, long_about = None)]
pub struct DeletedHistoryQueryCmd {
    #[arg(short, long)]
    pub user: u64,
    #[arg(short, long, default_value_t = 0)]
    pub skip: usize,
    #[arg(short, long, default_value_t = 10)]
    pub limit: usize,
}

impl CommandExecutable for DeletedHistoryQueryCmd {
    type Output = Vec<DeletedHistory>;

    async fn execute_on(
        &self,
        executor: &crate::handlers::commands_handler::CommandsHandlerInner,
    ) -> crate::Result<Self::Output> {
        let db = &executor.surreal;
        
        let mut history = db
            .query("SELECT message.id AS message_id, message.channel_id AS channel_id FROM deleted_history WHERE message.author.id=$userid LIMIT $limit START $skip")
            .bind(("userid", self.user as i64))
            .bind(("skip", self.skip))
            .bind(("limit", self.limit))
            .await?;
        let historys = history.take::<Vec<DeletedHistory>>(0)?;
        Ok(historys)
    }
}

#[test]
fn test() {
    let args = DeletedHistoryQueryCmd::try_parse_from(["test", "--help"]);
    if let Err(e) = args {
        println!("{}", e);
    }
}
