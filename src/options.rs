use console::style;
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Options {
    pub remote: Remote,
    pub local: Local,
}

impl Options {
    pub fn new(remote_path: PathBuf, local_path: PathBuf) -> Self {
        Self {
            remote: Remote { path: remote_path },
            local: Local {
                path: local_path,
                ignore: Vec::new(),
            },
        }
    }

    pub fn remote_path(&self) -> &Path {
        self.remote.path.as_path()
    }

    pub fn local_path(&self) -> &Path {
        self.local.path.as_path()
    }
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Options:\n")?;
        write!(f, "Remote path: {:?}\n", style(&self.remote.path).yellow())?;
        write!(f, "Local path: {:?}", style(&self.local.path).yellow())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Remote {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Local {
    pub path: PathBuf,
    pub ignore: Vec<PathBuf>,
}
