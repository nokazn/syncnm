use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Serialize, Deserialize, Hash, Clone, Copy, Debug)]
pub enum PackageManagerKind {
  Npm,
  Yarn,
  Pnpm,
  Bun,
}

impl PackageManagerKind {
  pub fn file_names(&self) -> Vec<&str> {
    match self {
      PackageManagerKind::Npm => vec!["package-lock.json"],
      PackageManagerKind::Yarn => vec!["yarn.lock"],
      PackageManagerKind::Pnpm => vec!["pnpm-lock.yaml", "pnpm-lock.yml"],
      PackageManagerKind::Bun => vec!["bun.lockb"],
    }
  }
}
