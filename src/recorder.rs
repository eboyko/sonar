use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Error, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Acquire;

use chrono::Utc;
use log::error;

use crate::recorder::error::Error as RecorderError;
use crate::recorder::error::Error::OperationFailed;

mod tests;
mod error;

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
        let mut current_file = self.file.lock().unwrap();

        if current_file.as_mut().is_none() {
            match self.create_file() {
                Ok(new_file) => *current_file = Some(new_file),
                Err(error) => error!("{}", error)
            }
        }

        match current_file.as_mut().unwrap().write_all(data) {
            Ok(_) => { self.bytes.fetch_add(data.len(), Acquire); }
            Err(error) => error!("{}", error),
        }
    }

    pub fn flush(&self) {
        if let Ok(mut lock) = self.file.lock() {
            if let Some(file) = lock.as_mut() {
                file.flush().unwrap();
                *lock = None;
            }
        }
    }

    fn create_file(&self) -> Result<File, RecorderError> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let filepath = format!("{}/{}.mp3", self.path, timestamp);

        match OpenOptions::new().create(true).append(true).open(filepath) {
            Ok(file) => Ok(file),
            Err(error) => Err(OperationFailed(error)),
        }
    }
}

pub(crate) fn build(path: String) -> Result<Arc<Recorder>, Error> {
    fs::create_dir_all(&path)?;
    Ok(Arc::new(Recorder::new(path)))
}
