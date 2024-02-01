use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(EnumIter, Serialize, Deserialize, Hash, Clone, Copy, Debug)]
pub enum PackageManagerKind {
  PackageLock,
  YarnLock,
  PnpmLock,
  BunLockb,
}

impl PackageManagerKind {
  pub fn file_names(&self) -> Vec<&str> {
    match self {
      PackageManagerKind::PackageLock => vec!["package-lock.json"],
      PackageManagerKind::YarnLock => vec!["yarn.lock"],
      PackageManagerKind::PnpmLock => vec!["pnpm-lock.yaml", "pnpm-lock.yml"],
      PackageManagerKind::BunLockb => vec!["bun.lockb"],
    }
  }
}

