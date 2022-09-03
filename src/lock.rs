use serde_derive::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum LockError {
    #[error("Io Error")]
    Io(#[from] io::Error),
    #[error("Changes staged")]
    StagedChanges,
}

type LockResult<T> = Result<T, LockError>;

pub type Hash = [u8; 32];

pub trait Lock {
    fn files_map(&self) -> &HashMap<Hash, PathBuf>;
    fn generation(&self) -> usize;

    fn moved<'a>(&'a self, other: &'a impl Lock) -> HashSet<(&'a Path, &'a Path)> {
        let files = self.files_map();
        let other_files = other.files_map();
        let keys = files.keys().collect::<HashSet<&Hash>>();
        let other_keys = other_files.keys().collect::<HashSet<&Hash>>();

        keys.intersection(&other_keys)
            .filter_map(|hash| {
                let path = files.get(*hash).expect("Must be present");
                let other_path = other_files.get(*hash).expect("Must be present");

                if path != other_path {
                    Some((other_path.as_path(), path.as_path()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn deleted<'a>(&'a self, other: &'a impl Lock) -> HashSet<&'a Path> {
        let files = self.files_map();
        let other_files = other.files_map();
        let keys = files.keys().collect::<HashSet<&Hash>>();
        let other_keys = other_files.keys().collect::<HashSet<&Hash>>();

        other_keys
            .difference(&keys)
            .map(|hash| other_files.get(*hash).expect("Must be present").as_path())
            .collect()
    }

    fn added(&self, other: &impl Lock) -> HashSet<&Path> {
        let files = self.files_map();
        let other_files = other.files_map();
        let keys = files.keys().collect::<HashSet<&Hash>>();
        let other_keys = other_files.keys().collect::<HashSet<&Hash>>();

        keys.difference(&other_keys)
            .map(|hash| files.get(*hash).expect("Must be present").as_path())
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteLock {
    generation: usize,
    files: HashMap<Hash, PathBuf>,
}

impl Lock for RemoteLock {
    fn generation(&self) -> usize {
        self.generation
    }

    fn files_map(&self) -> &HashMap<Hash, PathBuf> {
        &self.files
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalLock {
    path: PathBuf,
    generation: usize,
    remove: HashSet<PathBuf>,
    add: HashSet<PathBuf>,
    files: HashMap<Hash, PathBuf>,
}

// TODO Hashing fast enough ?
fn hash_path(path: &Path) -> io::Result<Hash> {
    let bytes = fs::read(path)?;

    let mut hasher = blake3::Hasher::default();
    hasher.update(&bytes);
    Ok(hasher.finalize().into())
}

fn hashed_files(path: &Path) -> io::Result<HashMap<Hash, PathBuf>> {
    let mut files = HashMap::new();

    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.metadata()?.is_file() {
            let path = entry.path();
            let hash = hash_path(path)?;
            files.insert(hash, path.to_owned());
        }
    }

    Ok(files)
}

impl LocalLock {
    pub fn from_path<P: AsRef<Path>>(path: P) -> LockResult<Self> {
        let path = path.as_ref().to_owned();
        let files = hashed_files(&path)?;

        Ok(Self {
            path,
            files,
            generation: 0,
            add: HashSet::new(),
            remove: HashSet::new(),
        })
    }

    pub fn update(&mut self) -> LockResult<Self> {
        // if self.add.len() != 0 || self.remove.len() != 0 {
        //     return Err(LockError::StagedChanges);
        // }

        let files = hashed_files(&self.path)?;
        let new_lock = Self {
            path: self.path.clone(),
            files,
            generation: self.generation + 1,
            add: self.add.clone(),
            remove: self.remove.clone(),
        };
        let old = std::mem::replace(self, new_lock);

        Ok(old)
    }

    pub fn files(&self) -> impl Iterator<Item = &Path> {
        self.files.iter().map(|(_, v)| v.as_path())
    }

    pub fn stage_remove(&mut self, path: PathBuf) -> bool {
        self.remove.insert(path)
    }

    pub fn stage_add(&mut self, path: PathBuf) -> bool {
        self.add.insert(path)
    }

    pub fn staged(&self) -> impl Iterator<Item = &PathBuf> {
        self.add.difference(&self.remove)
    }

    pub fn to_remove(&self) -> impl Iterator<Item = &PathBuf> {
        self.remove.difference(&self.add)
    }
}

impl Lock for LocalLock {
    fn generation(&self) -> usize {
        self.generation
    }
    fn files_map(&self) -> &HashMap<Hash, PathBuf> {
        &self.files
    }
}
