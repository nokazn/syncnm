use base64ct::{Base64, Encoding};
use sha2::{Digest, Sha256};
use std::{
  fs, io,
  path::{Path, PathBuf},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::utils::path::to_absolute_path;

#[derive(EnumIter, Debug)]
enum LockfileKind {
  PackageLock,
  YarnLock,
  PnpmLock,
  BunLockb,
}

impl LockfileKind {
  pub fn file_names(&self) -> Vec<&str> {
    match self {
      LockfileKind::PackageLock => vec!["package-lock.json"],
      LockfileKind::YarnLock => vec!["yarn.lock"],
      LockfileKind::PnpmLock => vec!["pnpm-lock.yaml", "pnpm-lock.yml"],
      LockfileKind::BunLockb => vec!["bun.lockb"],
    }
  }
}

#[derive(Debug)]
pub struct Lockfile {
  kind: LockfileKind,
  path: PathBuf,
}

impl Lockfile {
  pub fn new<T: AsRef<Path>>(dir_path: T) -> Result<Self, String> {
    match Lockfile::try_to_read_lockfile(&dir_path) {
      Some((kind, path)) => Ok(Lockfile { kind, path }),
      None => Err(format!(
        "No lockfile at `{}`",
        to_absolute_path(&dir_path).to_string_lossy()
      )),
    }
  }

  fn try_to_read_lockfile<T: AsRef<Path>>(dir_path: T) -> Option<(LockfileKind, PathBuf)> {
    for kind in LockfileKind::iter() {
      for lockfile in kind.file_names() {
        let file_path = dir_path.as_ref().join(lockfile);
        if file_path.exists() {
          return Some((kind, file_path));
        }
      }
    }
    None
  }

  pub fn generate_hash(&self) -> Result<String, Option<io::Error>> {
    let mut file = fs::File::open(&self.path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let raw_hash = hasher.finalize();
    let hash = Base64::encode_string(&raw_hash);
    Ok(hash)
  }
}
