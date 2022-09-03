use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf, StripPrefixError},
    process::{Command, Stdio},
    str::FromStr,
};
use thiserror::Error;

use crate::{options::Options, LOCK_PATH};

#[derive(Debug, Error)]
pub enum MegaError {
    #[error("Io Error")]
    Io(#[from] io::Error),
    #[error["Strip Prefix Error"]]
    StripPrefix(#[from] StripPrefixError),
}

type MegaResult<T> = Result<T, MegaError>;

pub struct MegaCmd;

impl MegaCmd {
    pub fn push<P: AsRef<Path>>(file: P, options: &Options) -> MegaResult<()> {
        let file = file.as_ref().canonicalize()?;
        let remote = options.remote_path();
        let local = options.local_path();
        let relative = file.strip_prefix(local)?;

        let remote_dest = remote.join(relative);

        MegaCmd::put(file, remote_dest)
    }

    pub fn extend<'a>(
        files: impl IntoIterator<Item = &'a Path>,
        options: &Options,
    ) -> MegaResult<()> {
        for file in files {
            MegaCmd::push(file, options)?;
        }

        Ok(())
    }

    pub fn lock_exists(options: &Options) -> MegaResult<bool> {
        let remote = options.remote_path();
        let files = MegaCmd::ls(remote)?;

        let lock_path = PathBuf::from_str(LOCK_PATH).unwrap();

        Ok(files.contains(&lock_path))
    }

    fn put<L: AsRef<Path>, R: AsRef<Path>>(local_file: L, remote_path: R) -> MegaResult<()> {
        let _child = Command::new("mega-put")
            .arg("-c")
            .arg(local_file.as_ref())
            .arg(remote_path.as_ref())
            .spawn()?;

        Ok(())
    }

    fn rm<P: AsRef<Path>>(remote_path: P) -> MegaResult<()> {
        let _child = Command::new("mega-rm")
            .arg("-r")
            .arg(remote_path.as_ref())
            .spawn()?;

        // if !output.status.success() {
        //     io::stdout().write_all(&output.stdout)?;
        //     io::stderr().write_all(&output.stderr)?;
        // }

        Ok(())
    }

    fn ls<P: AsRef<Path>>(remote_path: P) -> MegaResult<Vec<PathBuf>> {
        let mut child = Command::new("mega-ls")
            .stdout(Stdio::piped())
            .arg(remote_path.as_ref())
            .spawn()?;
        let mut stdout = child
            .stdout
            .take()
            .expect("Could not get stdout from mega-ls");
        let mut output = String::new();
        stdout.read_to_string(&mut output)?;

        let paths = output
            .lines()
            .map(|p| PathBuf::from_str(p).expect("Could not convert &str to PathBuf"))
            .collect::<Vec<PathBuf>>();

        Ok(paths)
    }

    fn mv<S: AsRef<Path>, D: AsRef<Path>>(remote_src: S, remote_dest: D) -> MegaResult<()> {
        let _child = Command::new("mega-mv")
            .arg(remote_src.as_ref())
            .arg(remote_dest.as_ref())
            .spawn()?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::MegaCmd;

    #[test]
    fn test_ls() {
        MegaCmd::ls("Sync").unwrap();
    }
}
