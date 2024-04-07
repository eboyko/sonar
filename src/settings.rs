use std::sync::Arc;
use std::time::Duration;
use crate::settings::command_line_parser::fetch_arguments;
use crate::settings::error::Error;

mod command_line_parser;
mod error;

pub struct Settings {
    pub path: String,
    pub url: String,
    pub timeout: Duration,
}

pub(crate) fn build() -> Result<Arc<Settings>, Error> {
    let preferences = match fetch_arguments() {
        Ok(arguments) => arguments,
        Err(error) => return Err(error),
    };

    let path = preferences
        .get("path")
        .ok_or("Mandatory `path` argument missing")?;
    let url = preferences
        .get("url")
        .ok_or("Mandatory `url` argument missing")?;

    Ok(Arc::new(Settings {
        path: path.to_string(),
        url: url.to_string(),
        timeout: Duration::from_secs(5),
    }))
}
