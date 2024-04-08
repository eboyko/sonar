use std::error::Error;
use std::sync::Arc;
use log::{error, info, warn};

use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::{listener, monitor, recorder, settings};

pub(crate) struct Launcher {
    threads: TaskTracker,
    context: CancellationToken,
}

impl Launcher {
    pub async fn listen_signals(&self) {
        let context = self.context.clone();
        let mut signals = Signals::new([SIGINT, SIGTERM]).unwrap();
        info!("Listening for termination signals.");
        
        if signals.forever().next().is_some() {
            warn!("Termination signal received. Shutting down.");
            context.cancel();
            self.threads.close();
        }
    }
}

pub(crate) async fn start() -> Result<(), Box<dyn Error>> {
    let launcher = Arc::new(Launcher {
        threads: TaskTracker::new(),
        context: CancellationToken::new(),
    });

    let settings = match settings::build() {
        Ok(settings) => settings,
        Err(error) => return Err(Box::new(error)),
    };

    let recorder = recorder::build(settings.path.clone())?;

    let listener = listener::build(
        settings.url.clone(),
        settings.timeout,
        Arc::clone(&recorder),
        launcher.context.clone(),
    );

    let monitor = monitor::build(
        settings.port,
        Arc::clone(&listener),
        Arc::clone(&recorder),
        launcher.context.clone(),
    );

    let launcher_reference = Arc::clone(&launcher);
    launcher.threads.spawn(async move { launcher_reference.listen_signals().await });
    launcher.threads.spawn(async move { listener.start().await });
    launcher.threads.spawn(async move { monitor.start().await });

    launcher.threads.wait().await;

    Ok(())
}

