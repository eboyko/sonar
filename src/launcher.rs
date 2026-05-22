use log::{info, warn};
use std::error::Error;
use std::sync::Arc;

use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::settings::Settings;
use crate::storage::recorder;
use crate::{listener, monitor};

pub(crate) struct Launcher {
    threads: TaskTracker,
    context: CancellationToken,
}

impl Launcher {
    pub async fn listen_signals(&self) {
        let mut interrupt = signal(SignalKind::interrupt()).unwrap();
        let mut terminate = signal(SignalKind::terminate()).unwrap();
        info!("Listening for termination signals.");

        select! {
            _ = interrupt.recv() => {},
            _ = terminate.recv() => {},
        }

        warn!("Termination signal received. Shutting down.");
        self.context.cancel();
        self.threads.close();
    }
}

pub(crate) async fn start(settings: Settings) -> Result<(), Box<dyn Error>> {
    let launcher = Arc::new(Launcher {
        threads: TaskTracker::new(),
        context: CancellationToken::new(),
    });

    let recorder = recorder::build(settings.records_path)?;

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
