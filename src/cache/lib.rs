use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::cache::metadata::Metadata;
use crate::core::APP_NAME;
use crate::errors::Error;
use crate::utils::path::to_dir_key;
use crate::utils::{fs, hash::Hash};

#[derive(Debug, PartialEq, Clone)]
pub struct Cache {
  base_dir: PathBuf,
  target_dir: PathBuf,
  cache_dir: PathBuf,
  metadata: Metadata,
}

impl Cache {
  pub fn new(
    base_dir: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
    cache_dir: Option<impl AsRef<Path>>,
  ) -> Result<Self> {
    let base_dir = fs::exists_dir(base_dir)?;
    let target_dir = fs::exists_dir(target_dir)?;
    let cache_dir = cache_dir
      .map(|c| c.as_ref().to_path_buf())
      .or(dirs::cache_dir().map(|c| c.join(APP_NAME)))
      .ok_or(Error::NotAccessible(PathBuf::from(
        "Cache directory in your environment",
      )))?;
    let cache_dir = fs::exists_dir(&cache_dir).or(fs::make_dir_if_not_exists(&cache_dir))?;
    let metadata = Metadata::new(&cache_dir)?;
    Ok(Self {
      base_dir,
      target_dir,
      cache_dir,
      metadata,
    })
  }

  fn to_cache_path(&self, key: &Hash) -> PathBuf {
    self.cache_dir.join(key.to_string())
  }

  pub fn save(&self, key: Hash) -> Result<Self> {
    let cache = self.to_cache_path(&key);
    fs::create_symlink(&self.target_dir, cache).or::<Error>(Ok(()))?;
    let metadata = Metadata::new(&self.cache_dir)?;
    // TODO: branch and commit
    metadata.update(&self.base_dir, &key, "branch".into(), "commit".into())?;
    Ok(self.clone())
  }

  pub fn revoke_current_cache(&self, base_dir: &PathBuf) -> Result<Self> {
    if let Some(current_cache_key) = self.find_current_cache(base_dir) {
      fs::rename(&self.target_dir, self.to_cache_path(&current_cache_key))?;
    };
    Ok(self.clone())
  }

  pub fn find_current_cache(&self, base_dir: &PathBuf) -> Option<Hash> {
    let dir_key = to_dir_key(base_dir);
    let current_hash_key = Metadata::new(&self.cache_dir)
      .ok()?
      .contents
      .get(&dir_key)?
      .clone()
      .current_hash_key?;
    Some(current_hash_key)
  }

  pub fn restore(&self, base_dir: &PathBuf, key: &Hash) -> Result<Self> {
    let key: String = key.to_string();
    let cache = self.cache_dir.join(&key);

    if cache.is_symlink() {
      Ok(self.clone())
    } else if cache.is_dir() {
      if let Some(current_hash_key) = self.find_current_cache(base_dir) {
        if current_hash_key.to_string() == key {
          return Ok(self.clone());
        }
        // escape the current cache if exists
        fs::rename(&self.target_dir, self.to_cache_path(&current_hash_key))
          .map_err(|error| error.context("Failed to save the old cache"))
          .unwrap_or(());
      }
      // restore the cache
      fs::rename(cache, &self.target_dir).and(Ok(self.clone()))
    } else {
      Err(Error::NotDir(cache).into())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{collections::HashMap, fs, path::PathBuf};

  use crate::{
    test_each,
    utils::{
      fs::exists_dir,
      path::{clean_path_separator, to_absolute_path},
      result::convert_panic_to_result,
    },
  };

  struct CacheNewTestCase {
    input: (PathBuf, PathBuf, Option<PathBuf>),
    expected: Result<Cache>,
  }

  fn test_cache_new_each(case: CacheNewTestCase) {
    let cache_dir = case.input.2.clone().and_then(|c| exists_dir(c).ok());
    let cache = Cache::new(case.input.0, case.input.1, case.input.2);
    let result = convert_panic_to_result(|| {
      if case.expected.is_ok() {
        assert!(cache.is_ok());
        let cache = cache.as_ref().unwrap();
        let expected = &case.expected.unwrap();
        assert_eq!(clean_path_separator(&cache.base_dir), expected.base_dir);
        assert_eq!(clean_path_separator(&cache.target_dir), expected.target_dir);
        assert_eq!(clean_path_separator(&cache.cache_dir), expected.cache_dir);
        assert_eq!(cache.metadata.contents, expected.metadata.contents);
        assert_eq!(
          clean_path_separator(&cache.metadata.file_path),
          expected.metadata.file_path
        );
      } else {
        assert!(cache.is_err());
      }
    });
    if cache_dir.is_some() {
      fs::remove_file(cache.unwrap().metadata.file_path).unwrap();
    } else {
      fs::remove_dir_all(cache.unwrap().cache_dir).unwrap();
    }
    result.map_err(|error| panic!("{:?}", error)).unwrap();
  }

  test_each!(
    test_cache_new,
    test_cache_new_each,
    "1" => CacheNewTestCase {
      input: (PathBuf::from("src"), PathBuf::from("src"), None),
      expected: Ok(Cache {
        base_dir: to_absolute_path("src").unwrap(),
        target_dir: to_absolute_path("src").unwrap(),
        cache_dir: dirs::cache_dir().unwrap().join(APP_NAME),
        metadata: Metadata {
          contents: HashMap::new(),
          file_path: dirs::cache_dir().unwrap().join(APP_NAME).join("metadata.json"),
        },
      }),
    },
    "2" => CacheNewTestCase {
      input: (PathBuf::from("src"), PathBuf::from("src"), Some(PathBuf::from("tests/fixtures/cache/.cache"))),
      expected: Ok(Cache {
        base_dir: to_absolute_path("src").unwrap(),
        target_dir: to_absolute_path("src").unwrap(),
        cache_dir: to_absolute_path("tests/fixtures/cache/.cache").unwrap(),
        metadata: Metadata {
          contents: HashMap::new(),
          file_path: to_absolute_path("tests/fixtures/cache/.cache/metadata.json").unwrap(),
        },
      }),
    },
    "3" => CacheNewTestCase {
      input: (PathBuf::from("src"), PathBuf::from("src"), Some(PathBuf::from("tests/fixtures/cache"))),
      expected: Ok(Cache {
        base_dir: to_absolute_path("src").unwrap(),
        target_dir: to_absolute_path("src").unwrap(),
        cache_dir: to_absolute_path("tests/fixtures/cache").unwrap(),
        metadata: Metadata {
          contents: HashMap::new(),
          file_path: to_absolute_path("tests/fixtures/cache/metadata.json").unwrap(),
        },
      }),
    },
  );
}
