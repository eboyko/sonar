use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub(crate) enum Error {
    ActiveDiskDetectionFailed,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            Error::ActiveDiskDetectionFailed => write!(formatter, "Could not detect an active disk for the provided path"),
        }
    }
}

impl std::error::Error for Error {}

