use log::{info, warn};
use std::error::Error;
use std::sync::Arc;

use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::settings::Settings;
use crate::{listener, monitor};
use crate::storage::recorder;

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

pub(crate) async fn start(settings: Settings) -> Result<(), Box<dyn Error>> {
    let launcher = Arc::new(Launcher {
        threads: TaskTracker::new(),
        context: CancellationToken::new(),
    });

    let recorder = recorder::build(&settings.records_path)?;

    let listener = listener::build(
        settings.stream_url,
        settings.stream_timeout,
        Arc::clone(&recorder),
        launcher.context.clone(),
    );

    let monitor = monitor::build(
        settings.monitor_port_number,
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
