use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use log::{error, info, warn};
use reqwest::{get, Response};
use tokio::select;
use tokio::time::{sleep, timeout};
use tokio_util::bytes;
use tokio_util::sync::CancellationToken;
use url::Url;

use crate::listener::error::Error as ListenerError;
use crate::listener::error::Error::*;
use crate::storage::recorder::Recorder;

mod error;

const SLEEP_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) struct Listener {
    url: Url,
    timeout: Duration,
    recorder: Arc<Recorder>,
    context: CancellationToken,
    pub(crate) bytes: AtomicU64,
}

impl Listener {
    pub(crate) fn new(
        url: Url,
        timeout: Duration,
        recorder: Arc<Recorder>,
        context: CancellationToken,
    ) -> Self {
        Listener {
            url,
            timeout,
            recorder,
            context,
            bytes: AtomicU64::new(0),
        }
    }

    pub fn bytes_received(&self) -> u64 {
        self.bytes.load(Relaxed)
    }

    pub async fn start(&self) -> Result<(), ListenerError> {
        loop {
            match self.listen().await {
                termination @ Terminated => {
                    warn!("Termination signal received. Shutting down.");
                    self.recorder.flush();
                    return Err(termination);
                }
                error => {
                    error!("{}. Reconnecting in {} seconds.", error, self.timeout.as_secs());
                    self.recorder.flush();
                    self.pause().await?
                }
            }
        }
    }

    async fn pause(&self) -> Result<(), ListenerError> {
        let sleep_initial_time = Instant::now();

        while sleep_initial_time.elapsed() < self.timeout {
            select! {
                _ = self.context.cancelled() => { return Err(Terminated) }
                _ = sleep(SLEEP_INTERVAL) => {}
            }
        }

        Ok(())
    }

    async fn listen(&self) -> ListenerError {
        match get(self.url.as_str()).await {
            Ok(response) => {
                if response.status() == 200 {
                    self.process_response(response).await
                } else {
                    StreamEmpty
                }
            }
            Err(error) => ConnectionFailed(error)
        }
    }

    async fn process_response(&self, response: Response) -> ListenerError {
        let mut stream = response.bytes_stream();
        info!("Listening to {}", self.url);

        loop {
            select! {
                payload = timeout(self.timeout, stream.next()) => {
                    match payload {
                        Ok(Some(Ok(data))) => self.write_data(data),
                        Ok(Some(Err(error))) => return StreamCorrupted(error),
                        Ok(None) => return StreamEmpty,
                        Err(error) => return StreamElapsed(error)
                    }
                }
                _ = self.context.cancelled() => return Terminated
            }
        }
    }

    fn write_data(&self, data: bytes::Bytes) {
        if self.recorder.write(&data).is_err() {
            error!("Unexpected error while writing the data. Terminating shortly.");
            self.context.cancel();
        }

        self.bytes.fetch_add(data.len() as u64, Relaxed);
    }
}

pub(crate) fn build(
    url: Url,
    timeout: Duration,
    recorder: Arc<Recorder>,
    context: CancellationToken,
) -> Arc<Listener> {
    Arc::new(Listener::new(url, timeout, recorder, context))
}
