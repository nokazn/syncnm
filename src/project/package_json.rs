use std::{
  fs,
  hash::Hash,
  path::{Path, PathBuf},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::errors::Error;
use crate::project::lib::Dependencies;

pub fn to_package_json_path(base_dir: impl AsRef<Path>) -> PathBuf {
  const PACKAGE_JSON: &str = "package.json";
  base_dir.as_ref().join(PACKAGE_JSON)
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Hash, Debug, Clone, PartialEq, Default)]
pub struct PackageJson {
  pub name: Option<String>,
  pub version: Option<String>,
  pub private: Option<bool>,
  pub packageManager: Option<String>,
  pub dependencies: Option<Dependencies>,
  pub devDependencies: Option<Dependencies>,
  pub peerDependencies: Option<Dependencies>,
  pub overrides: Option<Dependencies>,
  pub optionalDependencies: Option<Dependencies>,
  pub workspaces: Option<Vec<String>>,
}

impl PackageJson {
  pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
    let file_path = to_package_json_path(base_dir);
    let contents = fs::read_to_string(&file_path);
    match contents {
      Ok(contents) => serde_json::from_str::<Self>(&contents)
        .map_err(|error| Error::Parse(vec![file_path], error.to_string()).into()),
      Err(_) => Err(Error::NoEntry(vec![file_path]).into()),
    }
  }
}
