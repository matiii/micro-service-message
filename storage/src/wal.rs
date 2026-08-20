use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use bytes::Bytes;
use crc32fast::Hasher;

pub struct Wal {
    file: File,
    local_path: String,
    snapshot_path: String,
}

impl Wal {
    pub fn new(base_path: &str, snapshot_path: &str, namespace: &str) -> anyhow::Result<Self> {
        let local_wal = format!("{}/{}/wal", base_path, namespace);
        let snapshot_wal = format!("{}/{}/wal", snapshot_path, namespace);
        let snapshot_directory = format!("{}/{}/snapshot", snapshot_path, namespace);

        fs::create_dir_all(&local_wal)?;
        fs::create_dir_all(&snapshot_wal)?;
        fs::create_dir_all(&snapshot_directory)?;

        let start = SystemTime::now();
        let since_epoch = start
            .duration_since(UNIX_EPOCH)?;
        let local_path = format!("{}/wal-{}.msm", local_wal, since_epoch.as_secs());
        let snapshot_path = format!("{}/wal-{}.msm", snapshot_wal, since_epoch.as_secs());
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open::<&str>(local_path.as_ref())?;

        Ok(Self {
            file,
            local_path,
            snapshot_path,
        })
    }

    pub fn append(&mut self, data: Bytes) -> anyhow::Result<()> {
        let mut hasher = Hasher::new();
        hasher.update(&data);
        let crc = hasher.finalize();

        self.file.write_all(&(data.len() as u32).to_le_bytes())?;
        self.file.write_all(&crc.to_le_bytes())?;
        self.file.write_all(&data)?;

        self.file.sync_all()?;

        Ok(())
    }

    pub fn snapshot(&self) -> anyhow::Result<()> {
        fs::copy(&self.local_path, &self.snapshot_path)?;

        Ok(())
    }

    pub fn local_path(&self) -> String {
        self.local_path.clone()
    }
}

pub fn clean_wal(wal: Wal) -> anyhow::Result<()> {
    let wal_path = wal.local_path();
    drop(wal);

    fs::remove_file(wal_path)?;

    Ok(())
}