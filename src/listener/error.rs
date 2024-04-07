use std::fmt::{Display, Formatter, Result};
use tokio::time::error::Elapsed;

#[derive(Debug)]
pub(crate) enum Error {
    Terminated,
    ConnectionFailed(reqwest::Error),
    StreamCorrupted(reqwest::Error),
    StreamEmpty,
    StreamElapsed(Elapsed),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Error::Terminated => write!(formatter, "Terminated"),
            Error::ConnectionFailed(message) => write!(formatter, "Connection failed: {}", message),
            Error::StreamCorrupted(message) => write!(formatter, "Stream is corrupted ({})", message),
            Error::StreamEmpty => write!(formatter, "Stream is empty"),
            Error::StreamElapsed(message) => write!(formatter, "Stream reading timeout ({})", message)
        }
    }
}
