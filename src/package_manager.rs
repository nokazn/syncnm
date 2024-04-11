use std::{
  io::{BufRead, BufReader},
  path::Path,
  process::{Command, Stdio},
  thread, vec,
};

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
    let mut child = Command::new(&self.executable_name)
      .arg(&self.install_sub_command)
      .current_dir(&base_dir)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|error| to_error(error.to_string()))?;

    let stdout_reader = BufReader::new(
      child
        .stdout
        .take()
        .ok_or(to_error("stdout is not available".to_string()))?,
    );
    let stderr_reader = BufReader::new(
      child
        .stdout
        .take()
        .ok_or(to_error("stderr is not available".to_string()))?,
    );

    thread::spawn(move || {
      for line in stdout_reader.lines() {
        let line = line.map_err(|error| {
          Error::FailedToInstallDependencies(self.clone(), base_dir.clone(), error.to_string())
        });
        // let line = line.map_err(|error| to_error(error.to_string()));
        if let Ok(line) = line {
          println!("{}", line);
        }
      }
    });
    thread::spawn(move || {
      for line in stderr_reader.lines() {
        let line = line.map_err(|error| {
          Error::FailedToInstallDependencies(self.clone(), base_dir.clone(), error.to_string())
        });
        // let line = line.map_err(|error| to_error(error.to_string()));
        if let Ok(line) = line {
          println!("{}", line);
        }
      }
    });

    child
      .wait()
      .map_err(|error| to_error(error.to_string()))
      .and_then(|status| {
        if status.success() {
          Ok(())
        } else {
          Err(Error::Any("".into()))
          // Err(to_error(format!(
          //   "Command exited with {} status",
          //   status
          //     .code()
          //     .map(|code| code.to_string())
          //     .unwrap_or("non-zero".into())
          // )))
        }
      });
    Ok(())
  }
}

#[derive(EnumIter, Serialize, Deserialize, Hash, Clone, Copy, Debug, Eq, PartialEq, Default)]
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
