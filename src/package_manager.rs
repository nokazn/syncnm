use std::{path::Path, process::Command};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
  core::Result,
  errors::{to_error, Error},
  utils::path::to_absolute_path,
};

#[derive(Debug, PartialEq)]
pub struct PackageManager {
  pub executable_name: String,
  pub install_sub_command: String,
}

impl From<PackageManagerKind> for PackageManager {
  fn from(value: PackageManagerKind) -> Self {
    match value {
      PackageManagerKind::Npm => PackageManager::new("npm", "install").unwrap(),
      PackageManagerKind::Yarn => PackageManager::new("yarn", "install").unwrap(),
      PackageManagerKind::Pnpm => PackageManager::new("pnpm", "install").unwrap(),
      PackageManagerKind::Bun => PackageManager::new("bun", "install").unwrap(),
    }
  }
}

impl PackageManager {
  pub fn new(
    executable_name: impl Into<String>,
    install_sub_command: impl Into<String>,
  ) -> Result<Self> {
    let executable_name = executable_name.into();
    let install_sub_command = install_sub_command.into();
    Ok(Self {
      executable_name,
      install_sub_command,
    })
  }

  pub fn install(self, base_dir: impl AsRef<Path>) -> Result<()> {
    let base_dir = to_absolute_path(base_dir)?;
    let output = Command::new(&self.executable_name)
      .arg(&self.install_sub_command)
      .current_dir(&base_dir)
      .output()
      .map_err(to_error)?;
    if output.status.success() {
      Ok(())
    } else {
      Err(Error::FailedToInstallDependencies(self, base_dir))
    }
  }
}

#[derive(EnumIter, Serialize, Deserialize, Hash, Clone, Copy, Debug, PartialEq, Default)]
pub enum PackageManagerKind {
  #[default]
  Npm,
  Yarn,
  Pnpm,
  Bun,
}

impl PackageManagerKind {
  pub fn to_lockfile_names(self) -> Vec<&'static str> {
    match self {
      PackageManagerKind::Npm => vec!["package-lock.json"],
      PackageManagerKind::Yarn => vec!["yarn.lock"],
      PackageManagerKind::Pnpm => vec!["pnpm-lock.yaml"],
      PackageManagerKind::Bun => vec!["bun.lockb"],
    }
  }

  pub fn to_corepack_name(self) -> Option<&'static str> {
    match self {
      PackageManagerKind::Npm => Some("npm"),
      PackageManagerKind::Yarn => Some("yarn"),
      PackageManagerKind::Pnpm => Some("pnpm"),
      PackageManagerKind::Bun => None,
    }
  }
}
