use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use log::{debug, error, info, warn};
use tokio::select;
use tokio_util::sync::CancellationToken;
use ureq::{Agent, AgentBuilder, Response};

use crate::error::Error;
use crate::error::Error::{CommunicationError, TerminationError};
use crate::recorder::Recorder;
use crate::settings::Settings;

pub(crate) struct Listener {
    agent: Agent,
    context: CancellationToken,
    recorder: Arc<RwLock<Recorder>>,
    settings: Arc<Settings>,
}

impl Listener {
    pub async fn listen_stream(&mut self) {
        loop {
            match self.connect().await {
                Err(CommunicationError) => {
                    error!("Communication error occurred");
                    continue;
                }
                Err(TerminationError) => {
                    warn!("Shutting down");
                    break;
                }
                _ => continue,
            }
        }
    }

    async fn connect(&mut self) -> Result<(), Error> {
        match self.agent.get(&self.settings.url).call() {
            Ok(response) => self.handle_response(response).await?,
            Err(error) => self.handle_error(error).await?,
        }

        Ok(())
    }

    async fn handle_error(&self, error: ureq::Error) -> Result<(), Error> {
        error!(
            "Connection error ({}). Retry in {} seconds.",
            error,
            self.settings.timeout.as_secs()
        );

        let handle_time = Instant::now();
        let retry_interval = Duration::from_millis(500);

        while handle_time.elapsed() < self.settings.timeout {
            select! {
                context = self.context.cancelled() => { return Err(TerminationError) }
                nothing = tokio::time::sleep(retry_interval) => {}
            }
        }

        Ok(())
    }

    async fn handle_response(&mut self, response: Response) -> Result<(), Error> {
        info!("Successfully connected to {}", self.settings.url);

        let mut reader = response.into_reader();
        let mut buffer = vec![0; 2048];

        loop {
            select! {
                payload = async { reader.read(&mut buffer) } => {
                    let mut recorder = self.recorder.write().unwrap();

                    match payload {
                        Ok(length) => {
                            debug!("{} bytes received", length);
                            recorder.write(&buffer[..length]);
                        }
                        Err(error) => {
                            error!("Reading error ({})", error);

                            recorder.write(&buffer[..buffer.len()]);
                            recorder.flush();

                            return Err(CommunicationError);
                        }
                    }
                }
                nothing = self.context.cancelled() => { return Err(TerminationError) }
            }
        }
    }
}

pub(crate) fn build(
    settings: Arc<Settings>,
    recorder: Arc<RwLock<Recorder>>,
    context: CancellationToken,
) -> Listener {
    let agent = AgentBuilder::new()
        .timeout_connect(settings.timeout)
        .timeout_read(settings.timeout)
        .build();

    Listener {
        agent,
        settings,
        recorder,
        context,
    }
}
