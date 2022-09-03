use crate::{
    lock::{LocalLock, LockError},
    megacmd::{MegaCmd, MegaError},
    options::Options,
    LOCK_PATH, OPTIONS_PATH,
};
use std::{
    fs,
    path::{PathBuf, StripPrefixError},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error("TOML Serialization Error")]
    Toml(#[from] toml::ser::Error),
    #[error("Io Error")]
    Io(#[from] std::io::Error),
    #[error("Strip Prefix Error")]
    StripPrefix(#[from] StripPrefixError),
    #[error("Repository exists")]
    Exists,
    #[error("Remote Repository exists")]
    RemoteExists,
    #[error("Error with megacmd")]
    Mega(#[from] MegaError),
    #[error("Lock Error")]
    Lock(#[from] LockError),
}

type InitResult<T> = Result<T, InitError>;

// TODO force option to override current

pub fn init(local: PathBuf, remote: Option<PathBuf>) -> InitResult<()> {
    let local_absolute = local.canonicalize()?;
    let local_relative = if let Some(parent) = local.parent() {
        local.strip_prefix(parent)?.to_owned()
    } else {
        local
    };

    let options_path = local_absolute.join(OPTIONS_PATH);
    let lock_path = local_absolute.join(LOCK_PATH);

    if options_path.exists() || lock_path.exists() {
        return Err(InitError::Exists);
    }

    log::debug!("LOCAL: repository not initialised");

    let lock = LocalLock::from_path(&local_absolute)?;
    let lock_toml = toml::to_string(&lock)?;
    fs::write(&lock_path, lock_toml)?;

    log::debug!("LOCAL: {LOCK_PATH} written");

    let options = if let Some(remote) = remote {
        Options::new(remote, local_absolute)
    } else {
        Options::new(local_relative, local_absolute)
    };

    let toml = toml::to_string(&options)?;
    fs::write(options_path, toml)?;

    log::debug!("LOCAL: {OPTIONS_PATH} written");

    if MegaCmd::lock_exists(&options)? {
        Err(InitError::RemoteExists)
    } else {
        Ok(())
    }
}
