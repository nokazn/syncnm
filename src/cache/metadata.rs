use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::errors::{to_error, Error};
use crate::utils::path::{to_dir_key, DirKey};
use crate::utils::{fs, hash::Hash};

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug, Default)]
pub struct CacheMeta {
  branch: String,
  commit: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug, Default)]
pub struct MetadataContents {
  pub current_hash_key: Option<Hash>,
  pub caches: HashMap<Hash, CacheMeta>,
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct Metadata {
  pub contents: HashMap<DirKey, MetadataContents>,
  pub file_path: PathBuf,
}

impl Metadata {
  pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
    const FILE_NAME: &str = "metadata.json";
    let file_path = cache_dir.as_ref().join(FILE_NAME);
    let contents = fs::read_to_string(&file_path);
    match contents {
      Ok(contents) => serde_json::from_str::<HashMap<DirKey, MetadataContents>>(&contents)
        .map(|contents| Self {
          contents,
          file_path: file_path.clone(),
        })
        .map_err(|error| Error::Parse(vec![file_path], error.to_string()).into()),
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

  pub fn update(
    &self,
    base_dir: &PathBuf,
    hash: &Hash,
    branch: String,
    commit: String,
  ) -> Result<Self> {
    let dir_key = to_dir_key(base_dir);
    let contents_value = {
      let mut caches = self
        .contents
        .get(&dir_key)
        .map(|c| c.caches.clone())
        .unwrap_or_default();
      caches.insert(hash.clone(), CacheMeta { branch, commit });
      MetadataContents {
        current_hash_key: Some(hash.clone()),
        caches,
      }
    };
    let mut contents = self.contents.clone();
    contents.insert(dir_key, contents_value);
    let json = serde_json::to_string(&contents)
      .map_err(|error| Error::Parse(vec![self.file_path.clone()], error.to_string()))?;
    fs::write(&self.file_path, json)?;
    Ok(Self {
      contents,
      ..self.clone()
    })
  }
}
