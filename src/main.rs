use log::error;

mod launcher;
mod listener;
mod monitor;
mod recorder;
mod settings;

#[tokio::main]
async fn main() {
    let settings = settings::parse();

    env_logger::Builder::new().filter_level(settings.log_level).init();

    if let Err(error) = launcher::start(settings).await {
        error!("{}", error);
    }
}
