use std::fmt::Error;
use std::sync::Arc;

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
        if signals.forever().next().is_some() {
            context.cancel();
            self.threads.close();
        }
    }
}

pub(crate) async fn start() -> Result<(), Error> {
    let launcher = Arc::new(Launcher {
        threads: TaskTracker::new(),
        context: CancellationToken::new(),
    });

    let settings = settings::build()?;

    let recorder = recorder::build(settings.path.clone())?;

    let listener = listener::build(
        settings.url.clone(),
        settings.timeout,
        Arc::clone(&recorder),
        launcher.context.clone(),
    );

    let monitor = monitor::build(
        Arc::clone(&listener),
        Arc::clone(&recorder),
        launcher.context.clone(),
    ).await.unwrap();

    let launcher_reference = Arc::clone(&launcher);
    launcher.threads.spawn(async move { listener.start().await });
    launcher.threads.spawn(async move { monitor.start().await });
    launcher.threads.spawn(async move { launcher_reference.listen_signals().await; });

    launcher.threads.wait().await;

    Ok(())
}
