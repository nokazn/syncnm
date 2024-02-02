use std::path::PathBuf;
use thiserror::Error;

use crate::utils::path::to_absolute_path;

#[derive(Debug, Error)]
pub enum Error {
  #[error("No such file or directory: `{}`", stringify_path(.0))]
  NoEntryError(Paths),
  #[error("Invalid workspace: `{}`", stringify_path(&Paths::One(.0.to_path_buf())))]
  InvalidWorkspaceError(PathBuf),
  #[error("\"name\" and \"version\" are missing in: `{}`", stringify_path(&Paths::One(.0.to_path_buf())))]
  InvalidPackageJsonFieldsForYarnError(PathBuf),
  #[error("\"name\" is missing in: `{}`", stringify_path(&Paths::One(.0.to_path_buf())))]
  InvalidPackageJsonFieldsForBunError(PathBuf),
  #[error("Failed to parse: `{}`", stringify_path(.0))]
  ParseError(Paths),
}

#[derive(Debug)]
pub enum Paths {
  One(PathBuf),
  Multiple(Vec<PathBuf>),
}

fn stringify_path(paths: &Paths) -> String {
  match paths {
    Paths::One(path) => to_absolute_path(path).to_string_lossy().to_string(),
    Paths::Multiple(paths) => paths
      .iter()
      .map(|path| to_absolute_path(path).to_string_lossy().to_string())
      .collect::<Vec<_>>()
      .join(", "),
  }
}
