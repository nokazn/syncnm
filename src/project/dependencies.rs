use std::hash::Hash;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{
  lib::{is_valid_base_dir, Dependencies},
  package_json::{to_package_json_path, PackageJson},
  package_manager::PackageManagerKind,
};

use crate::errors::Error;

#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq, Default)]
pub struct PackageDependencies {
  pub dependencies: Dependencies,
  pub dev_dependencies: Dependencies,
  pub peer_dependencies: Dependencies,
  pub overrides: Dependencies,
  pub optional_dependencies: Dependencies,
}

impl PackageDependencies {
  pub fn new(raw: PackageJson) -> Self {
    Self {
      dependencies: raw.dependencies.unwrap_or_default(),
      dev_dependencies: raw.devDependencies.unwrap_or_default(),
      peer_dependencies: raw.peerDependencies.unwrap_or_default(),
      optional_dependencies: raw.optionalDependencies.unwrap_or_default(),
      overrides: raw.overrides.unwrap_or_default(),
    }
  }
}

#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq, Default)]
pub struct WorkspacePackage {
  pub original: PackageJson,
  pub base_dir: PathBuf,
  pub kind: PackageManagerKind,
  pub dependencies: PackageDependencies,
}

impl WorkspacePackage {
  pub fn new(base_dir: impl AsRef<Path>, kind: PackageManagerKind) -> Result<Self> {
    let base_dir = base_dir.as_ref().to_path_buf();
    if !is_valid_base_dir(&base_dir) {
      return Err(Error::InvalidWorkspace(base_dir).into());
    }
    let original = PackageJson::new(&base_dir)?;
    Ok(Self {
      original: original.clone(),
      base_dir,
      kind,
      dependencies: PackageDependencies::new(original),
    })
  }

  pub fn validate_package_json_fields(self, base_dir: impl AsRef<Path>) -> Result<Self> {
    let package_json_path = to_package_json_path(&base_dir);
    match self.kind {
      PackageManagerKind::Yarn
        if self.original.name.is_none() || self.original.version.is_none() =>
      {
        Err(Error::InvalidPackageJsonFieldsForYarn(package_json_path).into())
      }
      PackageManagerKind::Bun if self.original.name.is_none() => {
        Err(Error::InvalidPackageJsonFieldsForBun(package_json_path).into())
      }
      _ => Ok(self),
    }
  }

  pub fn get_package_name(&self) -> (String, String) {
    let name = self.original.name.clone().unwrap_or(
      self
        .base_dir
        .file_name()
        .unwrap_or(self.base_dir.as_os_str())
        .to_string_lossy()
        .to_string(),
    );
    let fallback = self.base_dir.to_string_lossy().to_string();
    (name, fallback)
  }
}
