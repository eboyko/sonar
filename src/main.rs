mod command_line_parser;
mod settings;
mod recorder;
mod listener;

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

    let recorder = match recorder::build(settings.path) {
        Ok(recorder) => recorder,
        Err(message) => {
            error!("{}", message);
            std::process::exit(1);
        }
    };

    let mut listener = listener::build(settings.url, recorder);
    listener.start();
}
