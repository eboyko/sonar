use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use log::{debug, error, info};

use chrono::Utc;

pub(crate) struct Recorder {
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
