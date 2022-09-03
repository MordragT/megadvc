use crate::{
    lock::{Hash, LocalLock, Lock, LockError},
    options::Options,
    LOCK_PATH, OPTIONS_PATH,
};
use console::style;
use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    fmt, fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StatusError {
    #[error("Io Error")]
    Io(#[from] std::io::Error),
    #[error("Repository absent")]
    Absent,
    #[error("Toml Deserialize Error")]
    TomlDe(#[from] toml::de::Error),
    #[error("Lock Error")]
    Lock(#[from] LockError),
}

type StatusResult<T> = Result<T, StatusError>;

#[derive(Debug)]
pub struct LockStatus<'a> {
    files: HashSet<(&'a Path)>,
    staged: HashSet<(&'a PathBuf)>,
    to_remove: HashSet<(&'a PathBuf)>,
    moved: HashSet<(&'a Path, &'a Path)>,
    added: HashSet<(&'a Path)>,
    deleted: HashSet<(&'a Path)>,
}

impl<'a> fmt::Display for LockStatus<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.staged.len() > 0 {
            write!(f, "{}\n", style("Staged files:").bright())?;
            for p in &self.staged {
                write!(f, "{:?}\n", style(p).green())?;
            }
        }

        if self.to_remove.len() > 0 {
            write!(f, "\n{}\n", style("Files to remove:").bright())?;
            for p in &self.to_remove {
                write!(f, "{:?}\n", style(p).red())?;
            }
        }

        if self.moved.len() > 0 {
            write!(f, "\n{}\n", style("Moved files:").bright())?;
            for p in &self.moved {
                write!(f, "{:?}\n", style(p).cyan())?;
            }
        }

        if self.added.len() > 0 {
            write!(f, "\n{}\n", style("Added files:").bright())?;
            for p in &self.added {
                write!(f, "{:?}\n", style(p).green())?;
            }
        }

        if self.deleted.len() > 0 {
            write!(f, "\n{}\n", style("Deleted files:").bright())?;
            for p in &self.deleted {
                write!(f, "{:?}\n", style(p).red())?;
            }
        }

        Ok(())
    }
}

pub fn print_lock_status(old: LocalLock, lock: LocalLock) {
    let files = lock.files().collect();
    let staged = lock.staged().collect();
    let to_remove = lock.to_remove().collect();
    let moved = lock.moved(&old);
    let added = lock.added(&old);
    let deleted = lock.deleted(&old);

    let status = LockStatus {
        files,
        staged,
        to_remove,
        moved,
        added,
        deleted,
    };

    println!("{status}");
}

pub fn status<'a>() -> StatusResult<()> {
    let cwd = current_dir()?;

    let options_path = cwd.join(OPTIONS_PATH);
    let lock_path = cwd.join(LOCK_PATH);

    if !options_path.exists() || !lock_path.exists() {
        return Err(StatusError::Absent);
    }

    let options_bytes = fs::read(&options_path)?;
    let options: Options = toml::from_slice(&options_bytes)?;

    let lock_bytes = fs::read(&lock_path)?;
    let mut lock: LocalLock = toml::from_slice(&lock_bytes)?;
    let old = lock.update()?;

    println!("{options}\n");
    print_lock_status(old, lock);

    Ok(())
}
