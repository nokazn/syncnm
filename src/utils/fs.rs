use std::{
  fs,
  path::{Path, PathBuf},
};

use crate::{
  core::Result,
  errors::{Error, Paths},
};

pub fn exists_dir<T: AsRef<Path>>(dir: T) -> Result<PathBuf> {
  let dir = dir
    .as_ref()
    .to_path_buf()
    .canonicalize()
    .map_err(|_| Error::NoEntryError(Paths::One(dir.as_ref().to_path_buf())))?;
  if dir.is_dir() {
    Ok(dir)
  } else {
    Err(Error::NotDirError(dir))
  }
}

fn make_dir_if_not_exists(dir: impl AsRef<Path>) -> Result<()> {
  if !dir.as_ref().exists() {
    fs::create_dir_all(dir).map_err(|error| Error::Any(error.to_string()))?;
  }
  Ok(())
}

pub fn rename_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  make_dir_if_not_exists(&to)?;
  fs::rename(&from, &to).map_err(|error| Error::Any(error.to_string()))
}

#[cfg(unix)]
pub fn create_symlink_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  make_dir_if_not_exists(&to)?;
  std::os::unix::fs::symlink(&from, &to).map_err(|error| Error::Any(error.to_string()))
}

#[cfg(windows)]
pub fn create_symlink_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  make_dir_if_not_exists(&to)?;
  std::os::windows::fs::symlink_dir(&from, &to).unwrap();
}

pub fn write(file_path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
  fs::write(file_path, contents).map_err(|error| Error::Any(error.to_string()))
}
