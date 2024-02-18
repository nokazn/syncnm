use serde::{Deserialize, Serialize};
use std::{path::Path, result};
use strum_macros::EnumIter;

use crate::{
  errors::Error,
  lockfile::Lockfile,
  project::ProjectRoot,
  utils::hash::{Hash, Hashable},
};

pub type Result<T> = result::Result<T, Error>;

#[derive(EnumIter, Serialize, Deserialize, Hash, Clone, Copy, Debug, PartialEq)]
pub enum PackageManagerKind {
  Npm,
  Yarn,
  Pnpm,
  Bun,
}

impl Default for PackageManagerKind {
  fn default() -> Self {
    PackageManagerKind::Npm
  }
}

impl PackageManagerKind {
  pub fn to_lockfile_names(&self) -> Vec<&str> {
    match self {
      PackageManagerKind::Npm => vec!["package-lock.json"],
      PackageManagerKind::Yarn => vec!["yarn.lock"],
      PackageManagerKind::Pnpm => vec!["pnpm-lock.yaml"],
      PackageManagerKind::Bun => vec!["bun.lockb"],
    }
  }

  pub fn to_corepack_name(&self) -> Option<&'static str> {
    match self {
      PackageManagerKind::Npm => Some("npm"),
      PackageManagerKind::Yarn => Some("yarn"),
      PackageManagerKind::Pnpm => Some("pnpm"),
      PackageManagerKind::Bun => None,
    }
  }
}

struct Core;

impl Core {
  pub fn run(base_dir: impl AsRef<Path>) -> Result<()> {
    let lockfile = Lockfile::new(&base_dir);
    let lockfile_kind = lockfile.map(|l| l.kind).ok();
    let project_root = ProjectRoot::new(&base_dir, lockfile_kind)?;
    project_root.generate_hash();
    Ok(())
  }

  fn generate_cache_key(lockfile: Lockfile, project: ProjectRoot) -> Result<Hash> {
    let lockfile_hash = lockfile.generate_hash()?;
    let project_hash = project.generate_hash()?;
    Ok(Hash(
      format!("{}-{}", lockfile_hash, project_hash).to_string(),
    ))
  }
}
