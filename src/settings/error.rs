use std::error::Error as StandardError;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum Error {
    CommandLineCorrupted,
    PathOmitted,
    UrlOmitted,
    ArgumentPrefixCorrupted(String),
    ArgumentInvalid(String),
    UrlInvalid(String),
    PortInvalid(String),
}

impl StandardError for Error {}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Error::CommandLineCorrupted => write!(formatter, "Command line is corrupted"),
            Error::PathOmitted => write!(formatter, "Mandatory `path` argument is omitted"),
            Error::UrlOmitted => write!(formatter, "Mandatory `url` argument is omitted"),
            Error::ArgumentPrefixCorrupted(prefix) => write!(formatter, "Argument prefix is corrupted: {}", prefix),
            Error::ArgumentInvalid(argument) => write!(formatter, "Argument is invalid: {}", argument),
            Error::UrlInvalid(url) => write!(formatter, "URL is invalid: {}", url),
            Error::PortInvalid(port) => write!(formatter, "Port is invalid: {}", port),
        }
    }
}
