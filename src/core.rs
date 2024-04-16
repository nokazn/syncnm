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
    path::to_dir_key,
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
    let cache = Cache::new(&base_dir, &node_modules_dir, cache_dir.as_ref());
    let cache_hash_key = generate_cache_key(&base_dir, lockfile, &project_root);
    match (cache.as_ref(), cache_hash_key) {
      (Ok(cache), Ok(cache_hash_key)) => {
        match cache.restore(&base_dir, &cache_hash_key) {
          Ok(_) => return Ok(()),
          _ => {}
        };
      }
      _ => {}
    }
    if let Ok(cache) = &cache {
      // save the current cache before update node_modules and a lockfile
      cache.revoke_current_cache(&base_dir)?;
    }
  }

  let package_manager: PackageManager = project_root.kind.into();
  package_manager.execute_install(&base_dir)?;

  // a lockfile may updated after executing install
  let lockfile = Lockfile::new(&base_dir)?;
  let cache_key = generate_cache_key(&base_dir, &lockfile, &project_root)?;
  // reevaluate the cache because cache directory may change
  let cache = Cache::new(&base_dir, &node_modules_dir, cache_dir.as_ref());
  cache.and_then(|cache| cache.save(cache_key))?;
  Ok(())
}

fn generate_cache_key(
  base_dir: &PathBuf,
  lockfile: &Lockfile,
  project: &ProjectRoot,
) -> Result<Hash> {
  let lockfile_hash = lockfile.generate_hash()?;
  let project_hash = project.generate_hash()?;
  let base_dir = to_dir_key(base_dir);
  Ok(Hash(
    format!(
      "{}-{}-{}",
      &lockfile_hash.to_string(),
      &project_hash.to_string(),
      &base_dir.to_string()
    )
    .to_string(),
  ))
}

fn to_node_modules_dir(base_dir: impl AsRef<Path>) -> PathBuf {
  base_dir.as_ref().to_path_buf().join("node_modules")
}
