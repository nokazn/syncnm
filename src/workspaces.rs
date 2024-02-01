use glob::glob;
use log;
use std::path::PathBuf;

use crate::core::PackageManagerKind;

#[derive(Debug)]
pub struct Workspaces {
  kind: PackageManagerKind,
  pub packages: Vec<PathBuf>,
}

impl Workspaces {
  pub fn new(base_dir: PathBuf, kind: PackageManagerKind, patterns: Option<Vec<String>>) -> Self {
    Workspaces {
      kind: kind,
      packages: match &kind {
        PackageManagerKind::PackageLock => Workspaces::resolve_npm_workspaces(base_dir, patterns),
        PackageManagerKind::BunLockb => Workspaces::resolve_bun_workspaces(base_dir, patterns),
        PackageManagerKind::YarnLock => Workspaces::resolve_yarn_workspaces(base_dir, patterns),
        PackageManagerKind::PnpmLock => Workspaces::resolve_pnpm_workspaces(base_dir),
      },
    }
  }

  fn resolve_npm_workspaces(base_dir: PathBuf, patterns: Option<Vec<String>>) -> Vec<PathBuf> {
    todo!("TODO: resolve npm workspaces");
  }

  /// Evaluate the given patterns individually and return the paths of matched entries in case of yarn.
  fn resolve_yarn_workspaces(base_dir: PathBuf, patterns: Option<Vec<String>>) -> Vec<PathBuf> {
    patterns
      .unwrap_or_default()
      .iter()
      .filter_map(|pattern| {
        let file_pattern = base_dir.join(pattern);
        match glob(&file_pattern.to_string_lossy().as_ref()) {
          Ok(entries) => {
            let entries = entries.filter_map(|entry| {
              if let Err(error) = &entry {
                log::warn!("Cannot access to a file or directory: {:?}", error.path());
              }
              entry.ok()
            });
            Some(entries)
          }
          Err(error) => {
            log::warn!("Invalid glob pattern: {:?}", error);
            None
          }
        }
      })
      .flatten()
      .collect::<Vec<_>>()
  }

  fn resolve_bun_workspaces(base_dir: PathBuf, patterns: Option<Vec<String>>) -> Vec<PathBuf> {
    todo!("TODO: resolve bun workspaces")
  }

  fn resolve_pnpm_workspaces(base_dir: PathBuf) -> Vec<PathBuf> {
    todo!("TODO: resolve pnpm workspaces")
  }
}
