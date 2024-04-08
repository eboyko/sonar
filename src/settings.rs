use std::sync::Arc;
use std::time::Duration;

use crate::settings::command_line_parser::fetch_arguments;
use crate::settings::error::{Error, Error::PathOmitted, Error::UrlOmitted};

mod command_line_parser;
mod error;

pub struct Settings {
    pub path: String,
    pub url: String,
    pub port: u16,
    pub timeout: Duration,
}

pub(crate) fn build() -> Result<Arc<Settings>, Error> {
    let preferences = match fetch_arguments() {
        Ok(arguments) => arguments,
        Err(error) => return Err(error),
    };

    let path = preferences.get("path").ok_or(PathOmitted)?;
    let url = preferences.get("url").ok_or(UrlOmitted)?;
    let port = preferences.get("port").unwrap_or(&"3000".to_string()).parse::<u16>().unwrap();

    Ok(Arc::new(Settings {
        path: path.to_string(),
        url: url.to_string(),
        port,
        timeout: Duration::from_secs(5),
    }))
}
