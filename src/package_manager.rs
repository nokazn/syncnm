use std::{path::Path, process::Command, vec};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{core::Result, errors::Error};

#[derive(Debug, PartialEq, Clone)]
pub struct PackageManager {
  pub executable_name: String,
  pub install_sub_command: String,
  pub lockfile_names: Vec<&'static str>,
  pub corepack_name: Option<&'static str>,
}

impl From<PackageManagerKind> for PackageManager {
  fn from(value: PackageManagerKind) -> Self {
    match value {
      PackageManagerKind::Npm => {
        PackageManager::new("npm", "install", vec!["package-lock.json"], Some("npm"))
      }
      PackageManagerKind::Yarn => {
        PackageManager::new("yarn", "install", vec!["yarn.lock"], Some("yarn"))
      }
      PackageManagerKind::Pnpm => {
        PackageManager::new("pnpm", "install", vec!["pnpm-lock.yaml"], Some("pnpm"))
      }
      PackageManagerKind::Bun => PackageManager::new("bun", "install", vec!["bun.lockb"], None),
    }
  }
}

impl PackageManager {
  pub fn new(
    executable_name: impl Into<String>,
    install_sub_command: impl Into<String>,
    lockfile_names: Vec<&'static str>,
    corepack_name: Option<&'static str>,
  ) -> Self {
    let executable_name = executable_name.into();
    let install_sub_command = install_sub_command.into();
    Self {
      executable_name,
      install_sub_command,
      lockfile_names,
      corepack_name,
    }
  }

  pub fn execute_install(self, base_dir: impl AsRef<Path>) -> Result<()> {
    let base_dir = base_dir.as_ref().to_path_buf();
    let to_error =
      |message: String| Error::FailedToInstallDependencies(self.clone(), base_dir.clone(), message);
    let output = Command::new(&self.executable_name)
      .arg(&self.install_sub_command)
      .current_dir(&base_dir)
      .output()
      .map_err(|error| to_error(error.to_string()))?;

    // TODO: stream
    let text = String::from_utf8_lossy(&output.stdout);
    println!("{}", text);

    if output.status.success() {
      Ok(())
    } else {
      Err(to_error(
        String::from_utf8_lossy(&output.stderr).to_string(),
      ))
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
    let package_manager: PackageManager = self.into();
    package_manager.lockfile_names
  }

  pub fn to_corepack_name(self) -> Option<&'static str> {
    let package_manager: PackageManager = self.into();
    package_manager.corepack_name
  }
}
