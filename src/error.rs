use std::fmt::Display;

use qqbot_sdk::bot::BotError;

#[derive(Debug)]
pub enum Error {
    Interal(String),
    External(Box<dyn std::error::Error>),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Interal(message) => write!(f, "{}", message),
            Error::External(external) => write!(f, "{}", external),
        }
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(val: String) -> Self {
        Error::Interal(val)
    }
}

impl From<surrealdb::Error> for Error {
    fn from(val: surrealdb::Error) -> Self {
        Error::External(Box::new(val))
    }
}

impl From<BotError> for Error {
    fn from(val: BotError) -> Self {
        Error::External(Box::new(val))
    }
}
