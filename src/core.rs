use serde::{Deserialize, Serialize};
use std::result;
use strum_macros::EnumIter;

use crate::errors::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(EnumIter, Serialize, Deserialize, Hash, Clone, Copy, Debug, PartialEq)]
pub enum PackageManagerKind {
  Npm,
  Yarn,
  Pnpm,
  Bun,
}

impl Default for PackageManagerKind {
  fn default() -> Self {
    PackageManagerKind::Npm
  }
}

impl PackageManagerKind {
  pub fn lockfile_names(&self) -> Vec<&str> {
    match self {
      PackageManagerKind::Npm => vec!["package-lock.json"],
      PackageManagerKind::Yarn => vec!["yarn.lock"],
      PackageManagerKind::Pnpm => vec!["pnpm-lock.yaml"],
      PackageManagerKind::Bun => vec!["bun.lockb"],
    }
  }

  pub fn name(&self) -> Option<&'static str> {
    match self {
      PackageManagerKind::Npm => Some("npm"),
      PackageManagerKind::Yarn => Some("yarn"),
      PackageManagerKind::Pnpm => Some("pnpm"),
      PackageManagerKind::Bun => None,
    }
  }
}
