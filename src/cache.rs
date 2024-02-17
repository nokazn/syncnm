use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::Result;
use crate::errors::{Error, Paths};
use crate::utils::fs::{create_symlink_dir, exists_dir, rename_dir, write};
use crate::utils::hash::Hash;

#[derive(Deserialize, Serialize, Clone, PartialEq)]
struct CacheMeta {
  branch: String,
  commit: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
struct MetadataJson {
  current_hash: Option<Hash>,
  caches: HashMap<Hash, CacheMeta>,
}

#[derive(PartialEq, Clone)]
struct Metadata {
  contents: MetadataJson,
  file_path: PathBuf,
}

impl Metadata {
  pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
    const FILE_NAME: &'static str = "metadata.json";
    let file_path = cache_dir.as_ref().join(FILE_NAME);
    let contents = fs::read_to_string(&file_path);
    match contents {
      Ok(contents) => serde_json::from_str::<MetadataJson>(&contents)
        .map(|contents| Self {
          contents,
          file_path: file_path.clone(),
        })
        .map_err(|_| Error::ParseError(Paths::One(file_path))),
      Err(_) => Err(Error::NoEntryError(Paths::One(file_path))),
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
      .map_err(|_| Error::ParseError(Paths::One(self.file_path.clone())))?;
    write(&self.file_path, json)?;
    Ok(Self {
      contents,
      ..self.clone()
    })
  }
}

///
/// -----------------------------------------------------------------------------
///
#[derive(Debug, PartialEq)]
pub struct Cache {
  base_dir: PathBuf,
  cache_dir: PathBuf,
}

impl Cache {
  pub fn new<T: AsRef<Path>>(base_dir: T, cache_dir: Option<T>) -> Result<Self> {
    const DEFAULT_CACHE_DIR: &'static str = ".cache/syncnm";

    let base_dir = exists_dir(base_dir)?;
    let cache_dir = exists_dir(
      cache_dir
        .map(|c| c.as_ref().to_path_buf())
        .unwrap_or(base_dir.join(DEFAULT_CACHE_DIR)),
    )?;

    Ok(Self {
      base_dir,
      cache_dir,
    })
  }

  pub fn save<T: Into<String>>(&self, key: T) -> Result<()> {
    let key = key.into();
    let cache = self.cache_dir.join(&key);
    create_symlink_dir(&self.base_dir, cache)?;
    let metadata = Metadata::new(&self.cache_dir)?;
    metadata.update(Hash(key), "branch".to_string(), "commit".to_string())?;
    Ok(())
  }

  fn find_current_cache(&self) -> Option<PathBuf> {
    let current_hash = Metadata::new(&self.cache_dir).ok()?.contents.current_hash?;
    exists_dir(self.cache_dir.join(current_hash.to_string())).ok()
  }

  pub fn restore<T: Into<String>>(&self, key: T) -> Result<()> {
    let cache = self.cache_dir.join(key.into());
    if cache.is_dir() {
      if let Some(current) = self.find_current_cache() {
        rename_dir(&self.base_dir, current)
          .map_err(|error| error.log_warn(Some("Failed to save the current cache")))
          .unwrap_or(());
      }
      rename_dir(cache, &self.base_dir)
    } else {
      Err(Error::NotDirError(cache))
    }
  }
}
