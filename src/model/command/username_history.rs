use clap::Parser;
use qqbot_sdk::model::User;

use crate::handlers::commands_handler::CommandExecutable;

use crate::model::UsernameHistory;

#[derive(Debug, Parser)]
#[command(about, long_about = None)]
pub struct UsernameHistoryQueryCmd {
    #[arg(short, long)]
    pub user: u64,
    #[arg(short, long, default_value_t = 0)]
    pub skip: usize,
    #[arg(short, long, default_value_t = 10)]
    pub limit: usize
}

impl CommandExecutable for UsernameHistoryQueryCmd {
    type Output = Vec<UsernameHistory>;

    async fn execute_on(
        &self,
        executor: &crate::handlers::commands_handler::CommandsHandlerInner,
    ) -> crate::Result<Self::Output> {
        let db = &executor.surreal;
        let mut history = db
            .query("SELECT * FROM username_history WHERE userid=$userid ORDER BY time DESC LIMIT $limit START $skip")
            .bind(("userid", self.user as i64))
            .bind(("skip", self.skip))
            .bind(("limit", self.limit))
            .await?;
        let historys = history.take::<Vec<UsernameHistory>>(0)?;
        Ok(historys)
    }
}
