use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicI8;
use std::sync::atomic::Ordering::Relaxed;
use std::time::{SystemTime, UNIX_EPOCH};

use sysinfo::Disks;

use crate::storage::error::Error;
use crate::storage::error::Error::ActiveDiskDetectionFailed;

pub(crate) struct DiskInspector {
    disks: RwLock<Disks>,
    active_disk_index: usize,
    last_refresh_timestamp: AtomicI8,
}

impl DiskInspector {
    const REFRESH_PERIOD_DURATION: i8 = 60;

    pub(crate) fn new(disks: Disks, target_device_index: usize) -> Self {
        Self {
            disks: RwLock::from(disks),
            active_disk_index: target_device_index,
            last_refresh_timestamp: AtomicI8::new(timestamp()),
        }
    }

    pub(crate) fn available_bytes(&self) -> u64 {
        self.ensure_refreshed();

        let disks = self.disks.read().unwrap();
        disks[self.active_disk_index].available_space()
    }

    fn ensure_refreshed(&self) {
        if self.refresh_required() {
            self.refresh();
        }
    }

    fn refresh_required(&self) -> bool {
        self.last_refresh_timestamp.load(Relaxed) + Self::REFRESH_PERIOD_DURATION < timestamp()
    }

    fn refresh(&self) {
        self.disks.write().unwrap().refresh();
        self.last_refresh_timestamp.store(timestamp(), Relaxed);
    }
}

pub(crate) fn build(absolute_path: &PathBuf) -> Result<Arc<DiskInspector>, Error> {
    let disks = Disks::new_with_refreshed_list();

    let active_disk_index = disks
        .iter()
        .position(|device| absolute_path.starts_with(device.mount_point()));

    match active_disk_index {
        Some(target_device_index) => Ok(Arc::new(DiskInspector::new(disks, target_device_index))),
        None => Err(ActiveDiskDetectionFailed),
    }
}

fn timestamp() -> i8 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i8
}
