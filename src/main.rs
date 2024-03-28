use std::{env, fs};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::thread::sleep;
use chrono::Utc;
use log::{debug, error, info, warn};
use ureq;

struct Settings {
    path: String,
    url: String,
}

impl Settings {
    pub fn new() -> Self {
        let mut path = "storage".to_string();
        let mut url = "https://nashe1.hostingradio.ru/ultra-256".to_string();

        let arguments: Vec<String> = env::args().skip(1).collect();
        for pair in arguments.chunks(2) {
            if pair.len() == 2 {
                match pair[0].as_str() {
                    "--path" => { path = pair[1].to_string() }
                    "--url" => { url = pair[1].to_string() }
                    _ => {
                        error!("Unknown argument `{}` ignored", pair[0].strip_prefix("--").unwrap())
                    }
                }
            } else {
                error!("Argument `{}` has no value", pair[0].strip_prefix("--").unwrap())
            }
        }

        Self { path, url }
    }
}

struct Recorder {
    path: String,
    file: Option<File>,
}

impl Recorder {
    pub fn new(path: String) -> Self {
        match fs::create_dir_all(&path) {
            Ok(_) => { info!("Directory `{}` created", &path) }
            Err(error) => { error!("Error creating directory `{}` ({})", &path, error) }
        }
        Self { path, file: None }
    }

    pub fn write(&mut self, data: &[u8]) {
        match self.file {
            Some(ref mut file) => {
                match file.write(&data) {
                    Ok(_) => { debug!("{} bytes written", &data.len()) }
                    Err(error) => error!("Error writing data ({})", error)
                }
            }
            None => {
                self.create_file();
                self.write(data);
            }
        }
    }

    pub fn flush(&mut self) {
        if let Some(file) = &mut self.file {
            match file.flush() {
                Ok(_) => { self.file = None }
                Err(error) => error!("Error flushing file: {}", error)
            }
        }
    }

    fn create_file(&mut self) {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let filepath = format!("{}/{}.mp3", self.path, timestamp);
        let file = OpenOptions::new().create(true).write(true).append(true).open(filepath).unwrap();

        self.file = Some(file)
    }
}

struct Listener {
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

fn main() {
    env_logger::init();

    let settings = Settings::new();
    let recorder = Recorder::new(settings.path);
    let mut listener = Listener::new(settings.url, recorder);

    listener.start();
}
