use std::fs::{File, OpenOptions, TryLockError};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

const LOCK_TIMEOUT: Duration = Duration::from_secs(30);
const RETRY_INTERVAL: Duration = Duration::from_millis(25);

#[derive(Debug, Clone, Copy)]
pub enum LockMode {
    Shared,
    Exclusive,
}

pub struct RepositoryLock {
    _file: File,
}

impl RepositoryLock {
    pub fn project(project_root: &Path, mode: LockMode) -> Result<Self> {
        Self::acquire(&project_root.join(".base/.lock"), mode)
    }

    pub fn global(base_home: &Path, mode: LockMode) -> Result<Self> {
        Self::acquire(&base_home.join(".lock"), mode)
    }

    fn acquire(path: &Path, mode: LockMode) -> Result<Self> {
        let parent = path.parent().context("lock path has no parent")?;
        std::fs::create_dir_all(parent)
            .with_context(|| format!("cannot create lock directory {}", parent.display()))?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .with_context(|| format!("cannot open repository lock {}", path.display()))?;
        let started = Instant::now();
        loop {
            let result = match mode {
                LockMode::Shared => file.try_lock_shared(),
                LockMode::Exclusive => file.try_lock(),
            };
            match result {
                Ok(()) => return Ok(Self { _file: file }),
                Err(TryLockError::WouldBlock) if started.elapsed() < LOCK_TIMEOUT => {
                    thread::sleep(RETRY_INTERVAL);
                }
                Err(TryLockError::WouldBlock) => bail!(
                    "timed out after {} seconds waiting for Base repository lock {}",
                    LOCK_TIMEOUT.as_secs(),
                    path.display()
                ),
                Err(TryLockError::Error(error)) => {
                    return Err(error).with_context(|| format!("cannot lock {}", path.display()));
                }
            }
        }
    }
}
