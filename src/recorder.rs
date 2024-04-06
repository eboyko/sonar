use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Acquire;

use chrono::Utc;
use log::error;

use crate::error::Error;
use crate::error::Error::FileCreation;

pub struct Recorder {
    path: String,
    file: Mutex<Option<File>>,
    bytes: AtomicUsize,
}

impl Recorder {
    pub fn new(path: String) -> Self {
        Recorder {
            path,
            file: Mutex::new(None),
            bytes: AtomicUsize::new(0),
        }
    }

    pub fn get_bytes(&self) -> usize {
        self.bytes.load(Acquire)
    }

    pub fn write(&self, data: &[u8]) {
        let mut file = self.file.lock().unwrap();

        if file.as_mut().is_none() {
            *file = Some(self.create_file().unwrap());
        }

        match file.as_mut().unwrap().write_all(data) {
            Ok(_) => { self.bytes.fetch_add(data.len(), Acquire); }
            Err(error) => error!("Could not write the data: {}", error)
        }
    }

    pub fn flush(&self) {
        if let Ok(mut lock) = self.file.lock() {
            if let Some(file) = lock.as_mut() {
                file.flush().unwrap();
                *lock = None;
            }
        } else {
            error!("Unable to get the lock");
        }
    }

    fn create_file(&self) -> Result<File, Error> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let filepath = format!("{}/{}.mp3", self.path, timestamp);

        match OpenOptions::new().create(true).append(true).open(filepath) {
            Ok(file) => Ok(file),
            Err(_) => Err(FileCreation),
        }
    }
}

pub(crate) fn build(path: String) -> Result<Arc<Recorder>, Error> {
    fs::create_dir_all(&path)?;
    Ok(Arc::new(Recorder::new(path)))
}

#[cfg(test)]
mod tests {
    const TEMPORARY_STORAGE_PATH: &str = "tmp/storage";

    #[test]
    fn build() {
        super::build(TEMPORARY_STORAGE_PATH.to_string()).unwrap();
        assert!(std::path::Path::new(TEMPORARY_STORAGE_PATH).exists());
        std::fs::remove_dir_all(TEMPORARY_STORAGE_PATH).unwrap();
    }
}
