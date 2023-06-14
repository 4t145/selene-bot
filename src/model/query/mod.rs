use crate::handlers::commands_handler::CommandsHandlerInner;
mod deleted_history;
pub use deleted_history::*;


pub trait QueryExecutable {
    type Output;
    async fn query(&self, executor: &CommandsHandlerInner) -> crate::Result<Self::Output>;
}