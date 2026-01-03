use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use sysinfo::Disks;

use crate::storage::error::Error;
use crate::storage::error::Error::ActiveDiskDetectionFailed;

pub(crate) struct DiskInspector {
    disks: RwLock<Disks>,
    mount_point: PathBuf,
    last_refresh_timestamp: AtomicUsize,
}

impl DiskInspector {
    const REFRESH_PERIOD_DURATION: usize = 60;

    pub(crate) fn new(disks: Disks, mount_point: PathBuf) -> Self {
        Self {
            disks: RwLock::from(disks),
            mount_point,
            last_refresh_timestamp: AtomicUsize::new(timestamp()),
        }
    }

    pub(crate) fn bytes_available(&self) -> u64 {
        self.ensure_refreshed();

        let disks = self.disks.read().unwrap();
        disks
            .iter()
            .find(|disk| disk.mount_point() == self.mount_point)
            .map(|disk| disk.available_space())
            .unwrap_or(0)
    }

    fn ensure_refreshed(&self) {
        if self.refresh_required() {
            self.refresh();
        }
    }

    fn refresh_required(&self) -> bool {
        let current_ts = timestamp();
        let last_refresh = self.last_refresh_timestamp.load(Relaxed);
        current_ts > last_refresh + Self::REFRESH_PERIOD_DURATION
    }

    fn refresh(&self) {
        let mut disks = self.disks.write().unwrap();
        disks.refresh_list();
        self.last_refresh_timestamp.store(timestamp(), Relaxed);
    }
}

pub(crate) fn build(absolute_path: &Path) -> Result<DiskInspector, Error> {
    let disks = Disks::new_with_refreshed_list();

    let mount_point = disks
        .iter()
        .find(|device| absolute_path.starts_with(device.mount_point()))
        .map(|device| device.mount_point().to_path_buf());

    match mount_point {
        Some(path) => Ok(DiskInspector::new(disks, path)),
        None => Err(ActiveDiskDetectionFailed),
    }
}

fn timestamp() -> usize {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize
}
