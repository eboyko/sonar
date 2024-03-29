use log::error;
use std::process::exit;
use crate::command_line_parser::fetch_arguments;

pub(crate) struct Settings {
    pub(crate) path: String,
    pub(crate) url: String,
}

pub(crate) fn build() -> Settings {
    let preferences = match fetch_arguments() {
        Ok(arguments) => arguments,
        Err(message) => {
            error!("{}", message);
            exit(1);
        }
    };

    let path = preferences.get("path").unwrap_or_else(|| {
        error!("Path is not provided");
        exit(1);
    });

    let url = preferences.get("url").unwrap_or_else(|| {
        error!("URL is not provided");
        exit(1);
    });

    Settings {
        path: path.to_string(),
        url: url.to_string(),
    }
}

