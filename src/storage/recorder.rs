use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize};
use std::sync::atomic::Ordering::Acquire;

use chrono::Utc;
use log::{error, info};

use crate::storage::disk_inspector;
use crate::storage::disk_inspector::DiskInspector;
use crate::storage::error::Error;
use crate::storage::error::Error::OperationFailed;

pub struct Recorder {
    path: PathBuf,
    file: Mutex<Option<File>>,
    bytes_written: AtomicUsize,
    disk_inspector: DiskInspector,
}

impl Recorder {
    pub fn new(path: PathBuf, disk_inspector: DiskInspector) -> Self {
        Recorder {
            path: fs::canonicalize(path).unwrap(),
            file: Mutex::new(None),
            bytes_written: AtomicUsize::new(0),
            disk_inspector,
        }
    }

    pub fn bytes_written(&self) -> usize {
        self.bytes_written.load(Acquire)
    }

    pub fn bytes_available(&self) -> usize {
        self.disk_inspector.bytes_available()
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
            Ok(_) => { self.bytes_written.fetch_add(data.len(), Acquire); }
            Err(error) => error!("{}", error)
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

    fn create_file(&self) -> Result<File, Error> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let filepath = self.path.join(format!("{}.mp3", timestamp));

        match OpenOptions::new().create(true).append(true).open(&filepath) {
            Ok(file) => {
                info!("New file `{}` created", &filepath.display());
                Ok(file)
            }
            Err(error) => Err(OperationFailed(error))
        }
    }
}

pub(crate) fn build(path: PathBuf) -> Result<Arc<Recorder>, Error> {
    let disk_inspector = disk_inspector::build(&path)?;
    Ok(Arc::new(Recorder::new(path, disk_inspector)))
}
