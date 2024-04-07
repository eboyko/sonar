use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum Error {
    PortBindingFailed(std::io::Error),
    Terminated,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Error::PortBindingFailed(error) => write!(formatter, "Failed to bind: {}", error),
            Error::Terminated => write!(formatter, "Terminated")
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::PortBindingFailed(error)
    }
}
