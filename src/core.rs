use serde::{Deserialize, Serialize};
use std::{path::Path, result};
use strum_macros::EnumIter;

use crate::{
  cache::Cache,
  errors::Error,
  lockfile::Lockfile,
  package_manager::PackageManager,
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

pub fn run(base_dir: impl AsRef<Path>, cache_dir: Option<impl AsRef<Path>>) -> Result<()> {
  let lockfile = Lockfile::new(&base_dir);
  let lockfile_kind = lockfile.as_ref().map(|l| l.kind).ok();
  let project_root = ProjectRoot::new(&base_dir, lockfile_kind)?;

  let base_dir = base_dir.as_ref().to_path_buf();
  let node_modules_dir = &base_dir.join("node_modules");
  let cache = Cache::new(&base_dir, &node_modules_dir, cache_dir.as_ref());

  if let Ok(lockfile) = &lockfile {
    let _ = generate_cache_key(&lockfile, &project_root).map(|cache_key| {
      #[allow(unused_must_use)]
      let _ = cache
        .and_then(|cache| cache.restore(cache_key.to_string()))
        .map(|result| {
          return result;
        });
    });
  }

  let package_manager: PackageManager = project_root.kind.into();
  package_manager.install(&base_dir)?;

  let lockfile = lockfile.or(Lockfile::new(&base_dir))?;
  let cache_key = generate_cache_key(&lockfile, &project_root)?;
  let cache = Cache::new(&base_dir, &node_modules_dir, cache_dir.as_ref());
  cache.and_then(|cache| cache.save(cache_key.to_string()))?;

  Ok(())
}

fn generate_cache_key(lockfile: &Lockfile, project: &ProjectRoot) -> Result<Hash> {
  let lockfile_hash = lockfile.generate_hash()?;
  let project_hash = project.generate_hash()?;
  Ok(Hash(
    format!("{}-{}", lockfile_hash, project_hash).to_string(),
  ))
}
