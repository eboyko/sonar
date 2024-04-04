use log::error;

use crate::error::Error;
use crate::launcher::Launcher;

mod command_line_parser;
mod error;
mod listener;
mod recorder;
mod settings;
mod monitor;
mod launcher;

#[tokio::main]
async fn main() {
    env_logger::init();

    if let Err(error) = launcher::start().await {
        error!("{}", error);
        return;
    }
}
