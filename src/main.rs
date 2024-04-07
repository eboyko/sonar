use log::error;

mod launcher;
mod listener;
mod monitor;
mod recorder;
mod settings;

#[tokio::main]
async fn main() {
    env_logger::init();

    if let Err(error) = launcher::start().await {
        error!("{}", error);
    }
}
