use std::{
  fs,
  path::{Path, PathBuf},
};

use crate::{
  core::Result,
  errors::{to_error, Error, Paths},
};

pub fn exists_dir(dir: impl AsRef<Path>) -> Result<PathBuf> {
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

pub fn make_dir_if_not_exists(dir: impl AsRef<Path>) -> Result<()> {
  if !dir.as_ref().exists() {
    fs::create_dir_all(dir).map_err(to_error)?;
  }
  Ok(())
}

pub fn rename_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let to = to.as_ref().to_path_buf();
  let parent = &to
    .parent()
    .ok_or(Error::NoEntryError(Paths::One(to.clone())))?;
  make_dir_if_not_exists(parent)?;
  fs::rename(&from, &to).map_err(to_error)
}

#[cfg(unix)]
pub fn create_symlink_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let to = to.as_ref().to_path_buf();
  let parent = &to
    .parent()
    .ok_or(Error::NoEntryError(Paths::One(to.clone())))?;
  make_dir_if_not_exists(parent)?;
  std::os::unix::fs::symlink(&from, &to).map_err(to_error)
}

#[cfg(windows)]
pub fn create_symlink_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let to = to.as_ref().to_path_buf();
  let parent = &to
    .parent()
    .ok_or(Error::NoEntryError(Paths::One(to.clone())))?;
  make_dir_if_not_exists(parent)?;
  std::os::windows::fs::symlink_dir(&from, &to).unwrap();
}

pub fn read_to_string(file_path: impl AsRef<Path>) -> Result<String> {
  fs::read_to_string(file_path).map_err(to_error)
}

pub fn write(file_path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
  fs::write(&file_path, &contents).map_err(to_error)
}
