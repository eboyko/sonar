use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum Error {
    OperationFailed(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Error::OperationFailed(error) => write!(formatter, "I/O operation failed: {}", error),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::OperationFailed(error)
    }
}
