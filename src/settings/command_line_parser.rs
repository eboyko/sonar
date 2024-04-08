use std::collections::HashMap;
use std::env;

use crate::settings::error::Error;
use crate::settings::error::Error::{ArgumentInvalid, ArgumentPrefixCorrupted, CommandLineCorrupted, PortInvalid, UrlInvalid};
use url::Url;

// List of arguments that can be passed to the application from the command line
const PERMITTED_ARGUMENTS: [&str; 3] = ["path", "url", "port"];

pub(crate) fn fetch_arguments() -> Result<HashMap<String, String>, Error> {
    let mut arguments = HashMap::new();

    let raw_arguments: Vec<String> = env::args().skip(1).collect();
    if raw_arguments.len() % 2 != 0 {
        return Err(CommandLineCorrupted);
    }

    for raw_argument_parts in raw_arguments.chunks(2) {
        match fetch_argument(raw_argument_parts.first(), raw_argument_parts.last()) {
            Ok((key, value)) => arguments.insert(key, value),
            Err(error) => return Err(error),
        };
    }

    Ok(arguments)
}

fn fetch_argument(
    raw_key: Option<&String>,
    raw_value: Option<&String>,
) -> Result<(String, String), Error> {
    if let (Some(raw_key), Some(raw_value)) = (raw_key, raw_value) {
        let key = raw_key
            .strip_prefix("--")
            .ok_or(ArgumentPrefixCorrupted(raw_key.to_string()))?;

        if !PERMITTED_ARGUMENTS.contains(&key) {
            return Err(ArgumentInvalid(key.to_string()));
        }

        match key {
            "url" => {
                if Url::parse(raw_value).is_err() {
                    return Err(UrlInvalid(raw_value.to_string()));
                }
            }
            "port" => {
                if raw_value.parse::<u16>().is_err() {
                    return Err(PortInvalid(raw_value.to_string()));
                }
            }
            _ => {}
        }

        Ok((key.to_string(), raw_value.to_string()))
    } else {
        Err(CommandLineCorrupted)
    }
}
