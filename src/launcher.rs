use std::sync::Arc;

use log::{debug, warn};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::{listener, monitor, recorder, settings};
use crate::error::Error;

pub(crate) struct Launcher {
    threads: TaskTracker,
    context: CancellationToken,
}

impl Launcher {
    pub async fn listen_signals(&self) {
        let context = self.context.clone();

        self.threads.spawn(async move {
            let mut signals = Signals::new([SIGINT, SIGTERM]).unwrap();
            debug!("Termination signal monitoring initiated");

            for signal in signals.forever() {
                warn!("Termination signal {} received", signal);
                context.cancel();
                return;
            }
        });
    }

    pub async fn stop(&self) {
        self.threads.close();
        self.threads.wait().await;
    }
}

pub(crate) async fn start() -> Result<(), Error> {
    let launcher = Launcher {
        threads: TaskTracker::new(),
        context: CancellationToken::new(),
    };

    let settings = settings::build()?;
    let recorder = recorder::build(Arc::clone(&settings))?;
    let mut monitor = monitor::build(Arc::clone(&recorder), launcher.context.clone()).await?;
    let mut listener = listener::build(Arc::clone(&settings), Arc::clone(&recorder), launcher.context.clone());

    launcher.listen_signals().await;
    launcher.threads.spawn(async move { monitor.start().await });
    launcher.threads.spawn(async move { listener.listen_stream().await });

    launcher.stop().await;

    Ok(())
}
