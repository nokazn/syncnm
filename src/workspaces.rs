use std::{
  fs,
  path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
  core::PackageManagerKind,
  errors::{Error, Paths},
  utils,
};

#[derive(Debug)]
pub struct Workspaces {
  kind: PackageManagerKind,
  pub packages: Vec<PathBuf>,
}

impl Workspaces {
  pub fn new(base_dir: PathBuf, kind: PackageManagerKind, patterns: Option<Vec<String>>) -> Self {
    Self {
      kind: kind,
      packages: match &kind {
        PackageManagerKind::Npm => Workspaces::resolve_npm_workspaces(base_dir, patterns),
        PackageManagerKind::Bun => Workspaces::resolve_bun_workspaces(base_dir, patterns),
        PackageManagerKind::Yarn => Workspaces::resolve_yarn_workspaces(base_dir, patterns),
        PackageManagerKind::Pnpm => Workspaces::resolve_pnpm_workspaces(base_dir),
      },
    }
  }

  fn resolve_npm_workspaces(base_dir: PathBuf, patterns: Option<Vec<String>>) -> Vec<PathBuf> {
    utils::glob::collect(&base_dir, patterns, true)
  }

  /// Evaluate the given patterns individually and return the paths of matched entries in case of yarn.
  fn resolve_yarn_workspaces(base_dir: PathBuf, patterns: Option<Vec<String>>) -> Vec<PathBuf> {
    utils::glob::collect(&base_dir, patterns, false)
  }

  fn resolve_bun_workspaces(base_dir: PathBuf, patterns: Option<Vec<String>>) -> Vec<PathBuf> {
    utils::glob::collect(&base_dir, patterns, false)
  }

  fn resolve_pnpm_workspaces(base_dir: PathBuf) -> Vec<PathBuf> {
    match PnpmWorkspace::new(&base_dir) {
      Ok(p) => utils::glob::collect(&base_dir, p.packages, true),
      Err(_) => vec![],
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct PnpmWorkspace {
  packages: Option<Vec<String>>,
}

impl PnpmWorkspace {
  fn new(base_dir: &PathBuf) -> Result<Self, Error> {
    let file_paths = Self::to_pnpm_workspace(base_dir);
    let contents = Self::read_to_string(&file_paths)?;
    serde_yaml::from_str::<Self>(&contents)
      .map_err(|_| Error::ParseError(Paths::Multiple(file_paths.to_vec())))
  }

  fn to_pnpm_workspace<T: AsRef<Path>>(base_dir: T) -> [PathBuf; 2] {
    const PNPM_WORKSPACE: [&str; 2] = ["pnpm-workspace.yaml", "pnpm-workspace.yml"];
    let base_dir = base_dir.as_ref().to_path_buf();
    PNPM_WORKSPACE.map(|p| base_dir.join(p))
  }

  fn read_to_string(file_paths: &[PathBuf; 2]) -> Result<String, Error> {
    for file_path in file_paths.iter() {
      if let Ok(contents) = fs::read_to_string(&file_path) {
        return Ok(contents);
      }
    }
    Err(Error::NoEntryError(Paths::Multiple(file_paths.to_vec())))
  }
}
