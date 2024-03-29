use std::thread::sleep;
use log::{debug, error, warn};

use crate::recorder::Recorder;

pub(crate) struct Listener {
    url: String,
    recorder: Recorder,
}

impl Listener {
    pub fn new(url: String, recorder: Recorder) -> Self {
        Self { url, recorder }
    }

    pub fn start(&mut self) {
        let default_timeout = std::time::Duration::from_secs(5);

        let transceiver = ureq::AgentBuilder::new()
            .timeout_connect(default_timeout)
            .timeout_read(default_timeout)
            .build();

        loop {
            let connection = transceiver.get(&self.url).call();
            if let Ok(response) = connection {
                let mut reader = response.into_reader();
                let mut buffer = vec![0; 2048];

                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => {
                            warn!("End of stream reached");
                            self.recorder.flush();
                            break;
                        }
                        Ok(length) => {
                            debug!("Received {} bytes", length);
                            self.recorder.write(&buffer[..length])
                        }
                        Err(error) => {
                            error!("Error ({})", error);
                            self.recorder.write(&buffer);
                            self.recorder.flush();
                            break;
                        }
                    }
                }
            }

            sleep(default_timeout);
        }
    }
}
