use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{Result, APP_NAME};
use crate::errors::{to_error, Error, Paths};
use crate::utils::{fs, hash::Hash};

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug, Default)]
struct CacheMeta {
  branch: String,
  commit: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug, Default)]
struct MetadataJson {
  current_hash: Option<Hash>,
  caches: HashMap<Hash, CacheMeta>,
}

#[derive(PartialEq, Clone, Debug, Default)]
struct Metadata {
  contents: MetadataJson,
  file_path: PathBuf,
}

impl Metadata {
  pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
    const FILE_NAME: &str = "metadata.json";
    let file_path = cache_dir.as_ref().join(FILE_NAME);
    let contents = fs::read_to_string(&file_path);
    match contents {
      Ok(contents) => serde_json::from_str::<MetadataJson>(&contents)
        .map(|contents| Self {
          contents,
          file_path: file_path.clone(),
        })
        .map_err(|error| Error::Parse(Paths::One(file_path), error.to_string())),
      Err(_) => {
        let v = Self {
          file_path: file_path.clone(),
          ..Self::default()
        };
        let contents = serde_json::to_string(&v.contents).map_err(to_error)?;
        fs::write(&file_path, contents)?;
        Ok(v)
      }
    }
  }

  fn update(&self, hash: Hash, branch: String, commit: String) -> Result<Self> {
    let contents = MetadataJson {
      current_hash: Some(hash.clone()),
      caches: {
        if self.contents.caches.get(&hash).is_none() {
          let mut caches = self.contents.caches.clone();
          caches.insert(hash.clone(), CacheMeta { branch, commit });
          caches
        } else {
          self.contents.caches.clone()
        }
      },
    };
    let json = serde_json::to_string(&contents)
      .map_err(|error| Error::Parse(Paths::One(self.file_path.clone()), error.to_string()))?;
    fs::write(&self.file_path, json)?;
    Ok(Self {
      contents,
      ..self.clone()
    })
  }
}

///
/// -----------------------------------------------------------------------------
///
#[derive(Debug, PartialEq, Clone)]
pub struct Cache {
  base_dir: PathBuf,
  target_dir: PathBuf,
  cache_dir: PathBuf,
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
    let cache_dir = fs::exists_dir(&cache_dir)
      .or_else(|_| fs::make_dir_if_not_exists(&cache_dir).map(|_| cache_dir))?;

    Ok(Self {
      base_dir,
      target_dir,
      cache_dir,
    })
  }

  pub fn save(&self, key: impl Into<String>) -> Result<Self> {
    let key = key.into();
    let cache = self.cache_dir.join(&key);
    fs::create_symlink(&self.target_dir, cache).or::<Error>(Ok(()))?;
    let metadata = Metadata::new(&self.cache_dir)?;
    // TODO: branch and commit
    metadata.update(Hash(key), "branch".to_string(), "commit".to_string())?;
    Ok(self.clone())
  }

  fn find_current_cache(&self) -> Option<(PathBuf, Hash)> {
    let current_hash = Metadata::new(&self.cache_dir).ok()?.contents.current_hash?;
    let current_path = fs::exists_dir(self.cache_dir.join(current_hash.to_string())).ok()?;
    Some((current_path, current_hash))
  }

  pub fn restore(&self, key: impl Into<String>) -> Result<Self> {
    let key: String = key.into();
    let cache = self.cache_dir.join(&key);
    if cache.is_dir() {
      if let Some((current_path, current_hash)) = self.find_current_cache() {
        if current_hash.as_str() == key {
          return Ok(self.clone());
        }
        fs::rename(&self.target_dir, current_path)
          .map_err(|error| error.log_warn(Some("Failed to save the current cache")))
          .unwrap_or(());
      }
      fs::rename(cache, &self.target_dir).and(Ok(self.clone()))
    } else {
      Err(Error::NotDir(cache))
    }
  }
}

mod tests {
  use super::*;
  use std::path::PathBuf;

  use crate::test_each;

  struct CacheNewTestCase {
    input: (PathBuf, PathBuf, Option<PathBuf>),
    expected: (PathBuf, PathBuf, Option<PathBuf>),
  }

  test_each!(
    test_cache_new,
    |case: CacheNewTestCase| {
      let cache = Cache::new(case.input.0, case.input.1, case.input.2);
      assert_eq!(cache.is_ok(), case.expected.is_ok());
    },
    "1" => CacheNewTestCase {
      input: (PathBuf::from("src"), PathBuf::from("src"), None),
      expected: Ok(()),
    },
  );
}
