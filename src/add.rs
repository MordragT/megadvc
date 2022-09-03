use crate::{
    lock::{LocalLock, LockError},
    LOCK_PATH, OPTIONS_PATH,
};
use std::{env::current_dir, fs, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AddError {
    #[error("Io Error")]
    Io(#[from] std::io::Error),
    #[error("Repository absent")]
    Absent,
    #[error("Toml Deserialize Error")]
    TomlDe(#[from] toml::de::Error),
    #[error("Toml Serialize Error")]
    TomlSer(#[from] toml::ser::Error),
    #[error("Lock Error")]
    Lock(#[from] LockError),
    #[error("File absent: {0:?}")]
    FileAbsent(PathBuf),
}

type AddResult<T> = Result<T, AddError>;

pub fn add(files: Vec<PathBuf>) -> AddResult<()> {
    let cwd = current_dir()?;

    let options_path = cwd.join(OPTIONS_PATH);
    let lock_path = cwd.join(LOCK_PATH);

    if !options_path.exists() || !lock_path.exists() {
        return Err(AddError::Absent);
    }

    let lock_bytes = fs::read(&lock_path)?;
    let mut lock: LocalLock = toml::from_slice(&lock_bytes)?;

    for file in files {
        if !file.exists() {
            return Err(AddError::FileAbsent(file));
        }
        lock.stage_add(file);
    }

    log::info!("All files staged to add");

    let lock_string = toml::to_string(&lock)?;
    fs::write(lock_path, lock_string)?;

    Ok(())
}
