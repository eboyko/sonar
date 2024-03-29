use std::collections::HashMap;
use std::env;
use url::Url;

// List of arguments that can be passed to the application from the command line
const PERMITTED_ARGUMENTS: [&str; 2] = ["path", "url"];

pub(crate) fn fetch_arguments() -> Result<HashMap<String, String>, &'static str> {
    let mut arguments = HashMap::new();

    let raw_arguments: Vec<String> = env::args().skip(1).collect();
    if raw_arguments.len() % 2 != 0 {
        return Err("Arguments string is corrupted");
    }

    for raw_argument_parts in raw_arguments.chunks(2) {
        match fetch_argument(raw_argument_parts.first(), raw_argument_parts.last()) {
            Ok((key, value)) => arguments.insert(key, value),
            Err(message) => return Err(message)
        };
    }

    Ok(arguments)
}

fn fetch_argument(raw_key: Option<&String>, raw_value: Option<&String>) -> Result<(String, String), &'static str> {
    if let (Some(raw_key), Some(raw_value)) = (raw_key, raw_value) {
        let key = raw_key.strip_prefix("--").ok_or("Non-prefixed argument found")?;

        if !PERMITTED_ARGUMENTS.contains(&key) {
            return Err("Unknown argument");
        }

        if key == "url" && Url::parse(raw_value).is_err() {
            return Err("Invalid URL value format");
        }

        Ok((key.to_string(), raw_value.to_string()))
    } else {
        Err("Argument is corrupted")
    }
}
