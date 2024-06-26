use std::{
  fs,
  path::{Path, PathBuf},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{errors::Error, utils};

use super::package_manager::PackageManagerKind;

#[derive(Debug)]
pub struct Workspaces {
  pub packages: Vec<PathBuf>,
}

impl Workspaces {
  pub fn new(base_dir: PathBuf, kind: PackageManagerKind, patterns: Option<Vec<String>>) -> Self {
    Self {
      packages: match &kind {
        PackageManagerKind::Npm => Workspaces::resolve_npm_workspaces(base_dir, patterns),
        PackageManagerKind::Bun => Workspaces::resolve_bun_workspaces(base_dir, patterns),
        PackageManagerKind::Yarn => Workspaces::resolve_yarn_workspaces(base_dir, patterns),
        PackageManagerKind::Pnpm => Workspaces::resolve_pnpm_workspaces(base_dir),
      },
    }
  }

  /// Support full glob syntax including negate patterns.
  /// - [workspaces | npm Docs](https://docs.npmjs.com/cli/v7/using-npm/workspaces)
  fn resolve_npm_workspaces(base_dir: PathBuf, patterns: Option<Vec<String>>) -> Vec<PathBuf> {
    utils::glob::collect(&base_dir, patterns, true)
  }

  /// Evaluate the given patterns individually and return the paths of matched entries in case of yarn.
  /// - [Workspaces | yarn](https://classic.yarnpkg.com/en/docs/workspaces)
  fn resolve_yarn_workspaces(base_dir: PathBuf, patterns: Option<Vec<String>>) -> Vec<PathBuf> {
    utils::glob::collect(&base_dir, patterns, false)
  }

  /// Full glob syntax is not supported yet.
  /// - https://bun.sh/docs/install/workspaces
  /// - https://github.com/oven-sh/bun/issues/1918
  fn resolve_bun_workspaces(base_dir: PathBuf, patterns: Option<Vec<String>>) -> Vec<PathBuf> {
    utils::glob::collect(&base_dir, patterns, false)
  }

  /// Support full glob syntax including negate patterns.
  /// - [Workspace | pnpm](https://pnpm.io/workspaces)
  /// - [pnpm-workspace.yaml | pnpm](https://pnpm.io/pnpm-workspace_yaml)
  fn resolve_pnpm_workspaces(base_dir: PathBuf) -> Vec<PathBuf> {
    match PnpmWorkspace::new(&base_dir) {
      Ok(p) => utils::glob::collect(&base_dir, p.packages, true),
      Err(_) => vec![],
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct PnpmWorkspace {
  packages: Option<Vec<String>>,
}

impl PnpmWorkspace {
  fn new(base_dir: &PathBuf) -> Result<Self> {
    let file_paths = Self::to_pnpm_workspace(base_dir);
    let contents = Self::read_to_string(&file_paths)?;
    serde_yaml::from_str::<Self>(&contents)
      .map_err(|error| Error::Parse(file_paths.to_vec(), error.to_string()).into())
  }

  fn to_pnpm_workspace(base_dir: impl AsRef<Path>) -> [PathBuf; 2] {
    const PNPM_WORKSPACE: [&str; 2] = ["pnpm-workspace.yaml", "pnpm-workspace.yml"];
    let base_dir = base_dir.as_ref().to_path_buf();
    PNPM_WORKSPACE.map(|p| base_dir.join(p))
  }

  fn read_to_string(file_paths: &[PathBuf; 2]) -> Result<String> {
    for file_path in file_paths.iter() {
      if let Ok(contents) = fs::read_to_string(file_path) {
        return Ok(contents);
      }
    }
    Err(Error::NoEntry(file_paths.to_vec()).into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{test_each_serial, utils::path::clean_path_separator};

  struct NewTestCase {
    input: (PathBuf, PackageManagerKind, Option<Vec<String>>),
    expected: Workspaces,
  }

  fn test_new_each(case: NewTestCase) {
    let base_dir = clean_path_separator(case.input.0);
    let patterns = case.input.2.map(|p| {
      p.iter()
        .map(|p| clean_path_separator(p).to_string_lossy().to_string())
        .collect::<Vec<_>>()
    });
    let workspaces = Workspaces::new(base_dir.clone(), case.input.1, patterns);
    assert_eq!(
      workspaces
        .packages
        .iter()
        .map(|p| p.canonicalize().unwrap())
        .collect::<Vec<_>>(),
      case
        .expected
        .packages
        .iter()
        .map(|path| base_dir
          .join(clean_path_separator(path))
          .canonicalize()
          .unwrap())
        .collect::<Vec<_>>()
    );
  }

  test_each_serial!(
    test_new,
    test_new_each,
    "npm" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/npm"),
        PackageManagerKind::Npm,
        Some(vec![
          String::from("packages/*"),
          String::from("!packages/c"),
        ]),
      ),
      expected: Workspaces {
        packages: vec![
          PathBuf::from("./packages/a"),
          PathBuf::from("./packages/b"),
        ],
      },
    },
    "yarn" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/yarn"),
        PackageManagerKind::Yarn,
        Some(vec![
          String::from("packages/*"),
          String::from("!packages/c"),
        ]),
      ),
      expected: Workspaces {
        packages: vec![
          PathBuf::from("./packages/a"),
          PathBuf::from("./packages/b"),
          PathBuf::from("./packages/c"),
        ],
      },
    },
    "pnpm" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/pnpm"),
        PackageManagerKind::Pnpm,
        None,
      ),
      expected: Workspaces {
        packages: vec![
          PathBuf::from("./packages/a"),
          PathBuf::from("./packages/b"),
        ],
      },
    },
    "bun" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/bun"),
        PackageManagerKind::Bun,
        Some(vec![
          String::from("packages/*"),
          String::from("!packages/c"),
        ]),
      ),
      expected: Workspaces {
        packages: vec![
          PathBuf::from("./packages/a"),
          PathBuf::from("./packages/b"),
          PathBuf::from("./packages/c"),
        ],
      },
    },
  );
}
