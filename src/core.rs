use std::{
  path::{Path, PathBuf},
  result,
};

use crate::{
  cache::Cache,
  errors::Error,
  lockfile::Lockfile,
  package_manager::PackageManager,
  project::ProjectRoot,
  utils::{
    hash::{Hash, Hashable},
    result::both_and_then,
  },
};

pub type Result<T> = result::Result<T, Error>;

pub const APP_NAME: &str = "syncnm";

pub fn run(base_dir: impl AsRef<Path>, cache_dir: Option<impl AsRef<Path>>) -> Result<()> {
  let base_dir = base_dir.as_ref().to_path_buf();
  let node_modules_dir = to_node_modules_dir(&base_dir);

  let lockfile = Lockfile::new(&base_dir);
  let lockfile_kind = lockfile.as_ref().map(|l| l.kind).ok();
  let project_root = ProjectRoot::new(&base_dir, lockfile_kind)?;

  if let Ok(lockfile) = &lockfile {
    if both_and_then(
      Cache::new(&base_dir, &node_modules_dir, cache_dir.as_ref()),
      generate_cache_key(lockfile, &project_root),
    )
    .and_then(|(cache, cache_key)| cache.restore(cache_key.to_string()))
    .is_ok()
    {
      return Ok(());
    }
  }

  let package_manager: PackageManager = project_root.kind.into();
  package_manager.execute_install(&base_dir)?;

  let lockfile = lockfile.or(Lockfile::new(&base_dir))?;
  let cache_key = generate_cache_key(&lockfile, &project_root)?;
  // reevaluate the cache because cache directory may change
  let cache = Cache::new(&base_dir, &node_modules_dir, cache_dir.as_ref());
  cache.and_then(|cache| cache.save(cache_key.to_string()))?;
  Ok(())
}

fn generate_cache_key(lockfile: &Lockfile, project: &ProjectRoot) -> Result<Hash> {
  let lockfile_hash = lockfile.generate_hash()?;
  let project_hash = project.generate_hash()?;
  Ok(Hash(
    format!(
      "{}-{}",
      &lockfile_hash.to_string()[..16].to_lowercase(),
      &project_hash.to_string()[..16].to_lowercase()
    )
    .to_string(),
  ))
}

fn to_node_modules_dir(base_dir: impl AsRef<Path>) -> PathBuf {
  base_dir.as_ref().to_path_buf().join("node_modules")
}
