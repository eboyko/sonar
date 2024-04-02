use crate::error::Error;
use crate::error::Error::{CommunicationError, TerminationError};
use crate::recorder::Recorder;
use crate::settings::Settings;
use log::{debug, error, info, warn};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc};
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};
use ureq::{Agent, AgentBuilder, Response};

const TERMINATION_SIGNALS: [i32; 2] = [SIGINT, SIGTERM];

pub(crate) struct Listener {
    agent: Agent,
    settings: Settings,
    recorder: Recorder,
}

impl Listener {
    pub fn start(&mut self) {
        let terminator = Arc::new(AtomicBool::new(false));

        self.listen_termination_signal(terminator.clone());
        self.listen_stream(terminator.clone());
    }

    fn listen_termination_signal(&self, terminator: Arc<AtomicBool>) {
        let mut signals = Signals::new(TERMINATION_SIGNALS).unwrap();

        thread::spawn(move || {
            for signal in signals.forever() {
                warn!("Signal {} received", signal);

                if TERMINATION_SIGNALS.contains(&signal) {
                    warn!("Termination signal received");
                    return terminator.store(true, Relaxed);
                }
            }
        });
    }

    fn listen_stream(&mut self, terminator: Arc<AtomicBool>) {
        loop {
            match self.connect(terminator.clone()) {
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

    fn connect(&mut self, terminator: Arc<AtomicBool>) -> Result<(), Error> {
        match self.agent.get(&self.settings.url).call() {
            Ok(response) => self.handle_response(response, terminator.clone())?,
            Err(error) => self.handle_error(error, terminator.clone())?,
        }

        Ok(())
    }

    fn handle_error(&self, error: ureq::Error, terminator: Arc<AtomicBool>) -> Result<(), Error> {
        error!("Connection error ({}). Retry in {} seconds.", error, self.settings.timeout.as_secs());

        let time = Instant::now();
        let retry_interval = Duration::from_millis(500);

        while time.elapsed() < self.settings.timeout {
            if terminator.load(Relaxed) {
                return Err(TerminationError);
            }

            sleep(retry_interval);
        }

        Ok(())
    }

    fn handle_response(&mut self, response: Response, terminator: Arc<AtomicBool>) -> Result<(), Error> {
        info!("Successfully connected to {}", self.settings.url);

        let mut reader = response.into_reader();
        let mut buffer = vec![0; 2048];

        loop {
            match reader.read(&mut buffer) {
                Ok(length) => {
                    debug!("{} bytes received", length);
                    self.recorder.write(&buffer[..length]);
                }
                Err(error) => {
                    error!("Reading error ({})", error);
                    self.recorder.write(&buffer[..buffer.len()]);
                    self.recorder.flush();
                    return Err(CommunicationError);
                }
            }

            if terminator.load(Relaxed) {
                return Err(TerminationError);
            }
        }
    }
}

pub(crate) fn build(settings: Settings, recorder: Recorder) -> Listener {
    let agent = AgentBuilder::new()
        .timeout_connect(settings.timeout)
        .timeout_read(settings.timeout)
        .build();

    Listener {
        settings,
        recorder,
        agent,
    }
}
