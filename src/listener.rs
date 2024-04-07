use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Acquire;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::Response;
use tokio::select;
use tokio::time::{sleep, timeout};
use tokio_util::bytes;
use tokio_util::sync::CancellationToken;

use crate::listener::error::Error;
use crate::recorder::Recorder;

mod error;

const SLEEP_INTERVAL: Duration = Duration::from_millis(500);

pub(crate) struct Listener {
    url: String,
    timeout: Duration,
    recorder: Arc<Recorder>,
    context: CancellationToken,
    pub(crate) bytes: AtomicUsize,
}

impl Listener {
    pub(crate) fn new(
        url: String,
        timeout: Duration,
        recorder: Arc<Recorder>,
        context: CancellationToken,
    ) -> Self {
        Listener {
            url,
            timeout,
            recorder,
            context,
            bytes: AtomicUsize::new(0),
        }
    }

    pub fn get_bytes(&self) -> usize {
        self.bytes.load(Acquire)
    }

    pub async fn start(&self) -> Result<(), Error> {
        loop {
            match self.listen().await {
                termination @ Error::Terminated => {
                    self.recorder.flush();
                    return Err(termination);
                }
                _ => self.pause().await?
            }
        }
    }

    async fn pause(&self) -> Result<(), Error> {
        let sleep_initial_time = Instant::now();

        while sleep_initial_time.elapsed() < self.timeout {
            select! {
                _ = self.context.cancelled() => { return Err(Error::Terminated) }
                _ = sleep(SLEEP_INTERVAL) => {}
            }
        }

        Ok(())
    }

    async fn listen(&self) -> Error {
        match reqwest::get(&self.url).await {
            Ok(response) => self.process_response(response).await,
            Err(error) => Error::ConnectionFailed(error),
        }
    }

    async fn process_response(&self, response: Response) -> Error {
        let mut stream = response.bytes_stream();

        loop {
            select! {
                payload = timeout(self.timeout, stream.next()) => {
                    match payload {
                        Ok(Some(Ok(data))) => self.write_data(data),
                        Ok(Some(Err(error))) => return Error::StreamCorrupted(error),
                        Ok(None) => return Error::StreamEmpty,
                        Err(error) => return Error::StreamElapsed(error),
                    }
                },
                _ = self.context.cancelled() => return Error::Terminated
            }
        }
    }

    fn write_data(&self, data: bytes::Bytes) {
        self.recorder.write(&data);
        self.bytes.fetch_add(data.len(), Acquire);
    }
}

pub(crate) fn build(
    url: String,
    timeout: Duration,
    recorder: Arc<Recorder>,
    context: CancellationToken,
) -> Arc<Listener> {
    Arc::new(Listener::new(url, timeout, recorder, context))
}
