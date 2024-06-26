use std::{fmt::Debug, path::PathBuf};

use itertools::Itertools;
use thiserror::Error;

use crate::{project::PackageManager, utils::path::to_absolute_path};

#[derive(Debug, Error, PartialEq)]
pub enum Error {
  #[error(
    "Cannot access to a file or a directory: {}",
    stringify_path(vec![.0.to_path_buf()])
  )]
  NotAccessible(PathBuf),

  #[error(
    "No such a file or a directory: {}",
    stringify_path(.0.to_vec())
  )]
  NoEntry(Vec<PathBuf>),

  #[error(
    "Not a directory: {}",
    stringify_path(vec![.0.to_path_buf()])
  )]
  NotDir(PathBuf),

  #[error(
    "No lockfile at {}",
    stringify_path(vec![.0.to_path_buf()])
  )]
  NoLockfile(PathBuf),

  #[error(
    "Multiple lockfiles at {}: {}",
    stringify_path(vec![.0.to_path_buf()]),
    stringify_path(.1.to_vec())
  )]
  MultipleLockfiles(PathBuf, Vec<PathBuf>),

  #[error(
    "Invalid workspace: {}",
    stringify_path(vec![.0.to_path_buf()])
  )]
  InvalidWorkspace(PathBuf),

  #[error(
    "\"name\" or \"version\" are missing in {}",
    stringify_path(vec![.0.to_path_buf()])
  )]
  InvalidPackageJsonFieldsForYarn(PathBuf),

  #[error(
    "\"private\" should be set to true in {}",
    stringify_path(vec![.0.to_path_buf()])
  )]
  InvalidPackageJsonPrivateForYarn(PathBuf),

  #[error(
    "\"name\" is missing in {}",
    stringify_path(vec![.0.to_path_buf()])
  )]
  InvalidPackageJsonFieldsForBun(PathBuf),

  #[error(
    "Failed to parse {}: {}",
    stringify_path(.0.to_vec()),
    .1
  )]
  Parse(Vec<PathBuf>, String),

  #[error(
    "Invalid glob pattern: {:?}",
    .0
  )]
  InvalidGlobPattern(&'static str),

  #[error(
    "Failed to install dependencies by \"{}\" at {:?}: {}",
    stringify_install_command(.0),
    .1,
    .2
  )]
  FailedToInstallDependencies(PackageManager, PathBuf, String),

  #[error(
    "Error: {:?}",
    .0
  )]
  Any(String),
}

impl Error {
  pub fn log_debug<E: Debug>(self, error: E) -> Self {
    log::debug!("{}: {:?}", &self.to_string(), error);
    self
  }

  pub fn log_warn(self, prefix: Option<&str>) -> Self {
    if let Some(prefix) = prefix {
      log::warn!("{}: {}", prefix, &self.to_string());
    } else {
      log::warn!("{}", &self.to_string());
    }
    self
  }

  pub fn log_error(self, prefix: Option<&str>) -> Self {
    if let Some(prefix) = prefix {
      log::error!("{}: {}", prefix, &self.to_string());
    } else {
      log::error!("{}", &self.to_string());
    }
    // TODO: terminate the process
    self
  }
}

/// convert to stringified absolute path
fn stringify_path(paths: Vec<PathBuf>) -> String {
  paths
    .iter()
    .map(|path| {
      to_absolute_path(path)
        .unwrap_or(path.clone())
        .to_string_lossy()
        .to_string()
    })
    .collect_vec()
    .join(", ")
}

fn stringify_install_command(package_manager: &PackageManager) -> String {
  format!(
    "{} {}",
    package_manager.executable_name, package_manager.install_sub_command
  )
}

pub fn to_error<E: Debug>(error: E) -> anyhow::Error {
  Error::Any(format!("{:?}", error)).into()
}
