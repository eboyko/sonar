mod command_line_parser;
mod error;
mod listener;
mod recorder;
mod settings;

use log::error;

fn main() {
    env_logger::init();

    let settings = match settings::build() {
        Ok(settings) => settings,
        Err(message) => {
            error!("{}", message);
            std::process::exit(1);
        }
    };

    let recorder = match recorder::build(settings.path.clone()) {
        Ok(recorder) => recorder,
        Err(message) => {
            error!("{}", message);
            std::process::exit(1);
        }
    };

    let mut listener = listener::build(settings, recorder);
    listener.start();
}
