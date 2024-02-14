use std::{fmt::Debug, path::PathBuf};
use thiserror::Error;

use crate::utils::path::to_absolute_path;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
  #[error("Cannot access to a file or a directory: `{}`", stringify_path(&Paths::One(.0.to_path_buf())))]
  NotAccessibleError(PathBuf),

  #[error("No such a file or a directory: `{}`", stringify_path(.0))]
  NoEntryError(Paths),

  #[error("No lockfile at: `{}`", stringify_path(&Paths::One(.0.to_path_buf())))]
  NoLockfileError(PathBuf),

  #[error("Invalid workspace: `{}`", stringify_path(&Paths::One(.0.to_path_buf())))]
  InvalidWorkspaceError(PathBuf),

  #[error("\"name\" or \"version\" are missing in: `{}`", stringify_path(&Paths::One(.0.to_path_buf())))]
  InvalidPackageJsonFieldsForYarnError(PathBuf),

  #[error("\"private\" should be set to `true`: `{}`", stringify_path(&Paths::One(.0.to_path_buf())))]
  InvalidPackageJsonPrivateForYarnError(PathBuf),

  #[error("\"name\" is missing in: `{}`", stringify_path(&Paths::One(.0.to_path_buf())))]
  InvalidPackageJsonFieldsForBunError(PathBuf),

  #[error("Failed to parse: `{}`", stringify_path(.0))]
  ParseError(Paths),

  #[error("Invalid glob pattern: {:?}", .0)]
  InvalidGlobPatternError(&'static str),
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

#[derive(Debug, PartialEq)]
pub enum Paths {
  One(PathBuf),
  Multiple(Vec<PathBuf>),
}

/// convert to stringified absolute path
fn stringify_path(paths: &Paths) -> String {
  match paths {
    Paths::One(path) => to_absolute_path(path)
      .unwrap_or(path.clone())
      .to_string_lossy()
      .to_string(),
    Paths::Multiple(paths) => paths
      .iter()
      .map(|path| {
        to_absolute_path(path)
          .unwrap_or(path.clone())
          .to_string_lossy()
          .to_string()
      })
      .collect::<Vec<_>>()
      .join(", "),
  }
}
