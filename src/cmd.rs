use clap::{Parser, Args, Subcommand, ValueEnum};
use crate::model::command::{*, self};
/// Doc comment
// #[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
// #[group(required = true)]
// pub struct Cli {
//     #[command(subcommand)]
//     pub command: Command,
// }

/// Doc comment
#[derive(Parser, Debug)]
#[command(author="4t145", version, about="Selene🌙 Bot🤖 Is👐 Watching👀 You🫵", long_about = None)]
pub enum Command {
    #[command(name="deleted", about="查询撤回消息", long_about = None)]
    DeletedHistoryQuery(DeletedHistoryQueryCmd),
    #[command(name="username", about="查询历史昵称", long_about = None)]
    UsernameHistoryQuery(UsernameHistoryQueryCmd),
}

#[test]
fn test_cli() {
    let cmd = [
        "@<this—bot>",
        "--help"
    ];
    match Command::try_parse_from(cmd) {
        Ok(cmd) => println!("{:?}", cmd),
        Err(info) => println!("{info}"),
    }
}