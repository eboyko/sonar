use std::error::Error as StandardError;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum Error {
    PortBindingFailed(std::io::Error),
    Terminated,
}

impl StandardError for Error {}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Error::PortBindingFailed(error) => write!(formatter, "Failed to start the server ({})", error),
            Error::Terminated => write!(formatter, "Terminated"),
        }
    }
}