use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum Error {
    InitializationError(String),
    TerminationError,
    CommunicationError,
    IoError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Error::InitializationError(message) => write!(formatter, "Initialization error: {}", message),
            Error::TerminationError => write!(formatter, "Termination error"),
            Error::CommunicationError => write!(formatter, "Communication error"),
            Error::IoError(error) => write!(formatter, "I/O error: {}", error),
        }
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::CommunicationError
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::CommunicationError
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}
