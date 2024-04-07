use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum Error {
    CommandLineCorrupted,
    ArgumentPrefixCorrupted(String),
    ArgumentInvalid(String),
    UrlFormatInvalid(String),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Error::CommandLineCorrupted => write!(formatter, "Command line is corrupted"),
            Error::ArgumentPrefixCorrupted(prefix) => write!(formatter, "Argument prefix is corrupted: {}", prefix),
            Error::ArgumentInvalid(argument) => write!(formatter, "Argument is invalid: {}", argument),
            Error::UrlFormatInvalid(url) => write!(formatter, "URL format is invalid: {}", url),
        }
    }
}
