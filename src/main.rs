mod settings;
mod recorder;
mod listener;

use crate::settings::Settings;
use crate::recorder::Recorder;
use crate::listener::Listener;

fn main() {
    env_logger::init();

    let settings = Settings::new();
    let recorder = Recorder::new(settings.path);
    let mut listener = Listener::new(settings.url, recorder);

    listener.start();
}
