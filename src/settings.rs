use std::env;
use log::error;

pub(crate) struct Settings {
    pub(crate) path: String,
    pub(crate) url: String,
}

impl Settings {
    pub fn new() -> Self {
        let mut path = "storage".to_string();
        let mut url = "https://nashe1.hostingradio.ru/ultra-256".to_string();

        let arguments: Vec<String> = env::args().skip(1).collect();
        for pair in arguments.chunks(2) {
            if pair.len() == 2 {
                match pair[0].as_str() {
                    "--path" => { path = pair[1].to_string() }
                    "--url" => { url = pair[1].to_string() }
                    _ => {
                        error!("Unknown argument `{}` ignored", pair[0].strip_prefix("--").unwrap())
                    }
                }
            } else {
                error!("Argument `{}` has no value", pair[0].strip_prefix("--").unwrap())
            }
        }

        Self { path, url }
    }
}
