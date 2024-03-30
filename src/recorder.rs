use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use log::{debug, error};

use chrono::Utc;

pub(crate) struct Recorder {
    path: String,
    file: Option<File>,
}

impl Recorder {
    pub fn write(&mut self, data: &[u8]) {
        match self.file {
            Some(ref mut file) => {
                match file.write_all(data) {
                    Ok(_) => debug!("{} bytes written", data.len()),
                    Err(error) => error!("Could not write the data: {}", error)
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
                Ok(_) => self.file = None,
                Err(error) => error!("Could not flush the file: {}", error)
            }
        }
    }

    fn create_file(&mut self) {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let filepath = format!("{}/{}.mp3", self.path, timestamp);

        match OpenOptions::new().create(true).append(true).open(filepath) {
            Ok(file) => self.file = Some(file),
            Err(error) => error!("Could not create the file: {}", error)
        };
    }
}

pub(crate) fn build(path: String) -> Result<Recorder, String> {
    if let Err(error) = fs::create_dir_all(&path) {
        return Err(format!("Could not create path `{}`: {}", path, error));
    }

    Ok(Recorder { path, file: None })
}
