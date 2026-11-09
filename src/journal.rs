//! Cooperating-writer, append-only journal compatible with the JavaScript adapter.
use crate::{Result, canonical, err, identity, require};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

fn io(e: std::io::Error) -> crate::Error {
    err(&format!("journal-io:{:?}", e.kind()))
}
#[derive(Clone, Debug)]
pub struct FileJournal {
    directory: PathBuf,
}
pub struct LockedJournal<'a> {
    journal: &'a FileJournal,
    lock: Option<File>,
}
impl Drop for LockedJournal<'_> {
    fn drop(&mut self) {
        self.lock.take();
        let _ = fs::remove_file(self.journal.directory.join(".writer-lock"));
    }
}
static STAGES: AtomicU64 = AtomicU64::new(0);
impl FileJournal {
    pub fn new(directory: impl AsRef<Path>) -> Self {
        Self {
            directory: directory.as_ref().to_path_buf(),
        }
    }
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    pub fn read(&self) -> Result<Vec<Value>> {
        let entries = match fs::read_dir(&self.directory) {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(io(e)),
        };
        let mut files = vec![];
        for e in entries {
            let e = e.map_err(io)?;
            let name = e
                .file_name()
                .into_string()
                .map_err(|_| err("invalid-journal"))?;
            if name == ".writer-lock" || name.starts_with(".stage-") {
                continue;
            }
            require(e.file_type().map_err(io)?.is_file(), "invalid-journal")?;
            files.push((name, e.path()));
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        let mut records = vec![];
        for (i, (name, path)) in files.iter().enumerate() {
            require(*name == format!("{i:012}.json"), "invalid-journal")?;
            let bytes = fs::read(path).map_err(io)?;
            records.push(canonical::decode(&bytes).map_err(|_| err("invalid-journal"))?);
        }
        identity::verify_records(&json!(records), &json!({}))?;
        Ok(records)
    }
    /// The lock spans read/decision/publication, not just the final file write.
    pub fn transaction<T>(&self, f: impl FnOnce(&mut LockedJournal<'_>) -> Result<T>) -> Result<T> {
        fs::create_dir_all(&self.directory).map_err(io)?;
        let mut lock = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.directory.join(".writer-lock"))
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    err("journal-busy")
                } else {
                    io(e)
                }
            })?;
        let initialized = (|| {
            lock.write_all(canonical::canonical_json(&json!({"kind":"temple3.writer-lock/v0.1","token":format!("rust-{}",std::process::id())}))?.as_bytes()).map_err(io)?;
            lock.sync_all().map_err(io)
        })();
        let mut guard = LockedJournal {
            journal: self,
            lock: Some(lock),
        };
        initialized?;
        f(&mut guard)
    }
    pub fn append(&self, entry: &Value, expected_tip: &Value) -> Result<Value> {
        self.transaction(|j| j.append(entry, expected_tip))
    }
}
impl LockedJournal<'_> {
    pub fn read(&self) -> Result<Vec<Value>> {
        self.journal.read()
    }
    pub fn append(&mut self, entry: &Value, expected_tip: &Value) -> Result<Value> {
        self.append_observed(entry, expected_tip, |_| Ok(()))
    }
    /// Test instrumentation observes real write barriers; it cannot skip writes.
    /// Any observer failure is an uncertain result; reopen before retrying.
    pub fn append_observed(
        &mut self,
        entry: &Value,
        expected_tip: &Value,
        mut observe: impl FnMut(&str) -> Result<()>,
    ) -> Result<Value> {
        require(expected_tip.is_string(), "missing-journal-tip")?;
        let records = self.read()?;
        let tip = identity::verify_records(&json!(records), &json!({"expectedTip":expected_tip}))?;
        let record = identity::record_for(entry, records.len(), &tip)?;
        let stage = self.journal.directory.join(format!(
            ".stage-rust-{}-{}",
            std::process::id(),
            STAGES.fetch_add(1, Ordering::Relaxed)
        ));
        let result = (|| {
            observe("lock-synced")?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&stage)
                .map_err(io)?;
            file.write_all(canonical::canonical_json(&record)?.as_bytes())
                .map_err(io)?;
            observe("stage-written")?;
            file.sync_all().map_err(io)?;
            observe("stage-synced")?;
            drop(file);
            fs::hard_link(
                &stage,
                self.journal
                    .directory
                    .join(format!("{:012}.json", records.len())),
            )
            .map_err(io)?;
            observe("published")?;
            fs::remove_file(&stage).map_err(io)?;
            observe("stage-removed")?;
            #[cfg(unix)]
            File::open(&self.journal.directory)
                .map_err(io)?
                .sync_all()
                .map_err(io)?;
            // Windows directory fsync is not available through this portable adapter.
            Ok(record)
        })();
        match fs::remove_file(stage) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(io(e)),
        }
        result
    }
}
