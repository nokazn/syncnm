use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{
  cache::Cache,
  project::{Lockfile, PackageManager, ProjectRoot},
  utils::{
    hash::{Hash, Hashable},
    path::to_dir_key,
  },
};

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
    if let (Ok(cache), Ok(cache_hash_key)) = (cache.as_ref(), cache_hash_key) {
      if cache.restore(&base_dir, &cache_hash_key).is_ok() {
        return Ok(());
      }
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

#[cfg(test)]
mod tests {
  use regex::Regex;
  use serial_test::serial;

  use super::*;

  #[serial]
  #[test]
  fn test_generate_cache_key() -> Result<()> {
    let base_dir = PathBuf::from("tests/fixtures/core");
    let lockfile = Lockfile::new(&base_dir)?;
    let project = ProjectRoot::new(&base_dir, Some(lockfile.kind))?;
    let result = generate_cache_key(&base_dir, &lockfile, &project)?;
    let r = Regex::new(
      r"^l3cuczxmteircrzf6dw52asj6vt6opt2-ilchfsie572gsieon7up5cbljysxda5p-[a-z_]+tests_fixtures_core$",
    );
    assert!(r.unwrap().is_match(&result.to_string()));
    Ok(())
  }
}
