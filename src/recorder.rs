use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, RwLock};

use chrono::Utc;
use log::{debug, error};

use crate::error::Error;
use crate::settings::Settings;

pub(crate) struct Recorder {
    pub(crate) bytes_counter: usize,
    file: Option<File>,
    settings: Arc<Settings>,
}

impl Recorder {
    pub fn write(&mut self, data: &[u8]) {
        match self.file {
            Some(ref mut file) => match file.write_all(data) {
                Ok(_) => self.bytes_counter += data.len(),
                Err(error) => error!("Could not write the data: {}", error),
            },
            None => {
                self.create_file();
                self.write(data);
            }
        }
    }

    pub fn flush(&mut self) {
        if let Some(file) = &mut self.file {
            match file.flush() {
                Ok(_) => self.file = None,
                Err(error) => error!("Could not flush the file: {}", error),
            }
        }
    }

    fn create_file(&mut self) {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let filepath = format!("{}/{}.mp3", self.settings.path, timestamp);

        match OpenOptions::new().create(true).append(true).open(filepath) {
            Ok(file) => self.file = Some(file),
            Err(error) => error!("Could not create the file: {}", error),
        };
    }
}

pub(crate) fn build(settings: Arc<Settings>) -> Result<Arc<RwLock<Recorder>>, Error> {
    fs::create_dir_all(&settings.path)?;
    Ok(Arc::new(RwLock::new(Recorder {
        settings,
        file: None,
        bytes_counter: 0,
    })))
}
