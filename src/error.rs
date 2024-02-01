use std::path::{Path, PathBuf};

use crate::utils::path::to_absolute_path;

pub trait IoError {
  fn new<T: AsRef<Path>>(file_path: T) -> Self;
}

#[derive(Debug)]
pub struct InvalidPackageJsonForWorkspacesError {
  pub file_path: PathBuf,
  pub message: String,
}

impl IoError for InvalidPackageJsonForWorkspacesError {
  fn new<T: AsRef<Path>>(file_path: T) -> Self {
    InvalidPackageJsonForWorkspacesError {
      file_path: file_path.as_ref().to_path_buf(),
      message: format!(
        "name and version are required in `{}`",
        to_absolute_path(&file_path).to_string_lossy()
      ),
    }
  }
}

#[derive(Debug)]
pub struct NoEntryError {
  pub file_path: PathBuf,
  pub message: String,
}

impl IoError for NoEntryError {
  fn new<T: AsRef<Path>>(file_path: T) -> Self {
    NoEntryError {
      file_path: file_path.as_ref().to_path_buf(),
      message: format!(
        "No such file or directory: `{}`",
        to_absolute_path(&file_path).to_string_lossy()
      ),
    }
  }
}

#[derive(Debug)]
pub struct ParseJsonError {
  pub file_path: PathBuf,
  pub message: String,
}

impl IoError for ParseJsonError {
  fn new<T: AsRef<Path>>(file_path: T) -> Self {
    ParseJsonError {
      file_path: file_path.as_ref().to_path_buf(),
      message: format!(
        "Failed to parse `{}`",
        to_absolute_path(&file_path).to_string_lossy()
      ),
    }
  }
}

#[derive(Debug)]
pub enum Error {
  NoEntryError(NoEntryError),
  InvalidPackageJsonForWorkspacesError(InvalidPackageJsonForWorkspacesError),
  ParseJsonError(ParseJsonError),
}
