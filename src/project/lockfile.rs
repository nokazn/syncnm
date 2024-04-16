use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use anyhow::Result;
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
  errors::{to_error, Error},
  utils::{hash::Hashable, option::both_and_then},
};

use super::package_manager::PackageManagerKind;

#[derive(Debug, PartialEq)]
pub struct Lockfile {
  pub kind: PackageManagerKind,
  path: PathBuf,
}

impl Hashable for Lockfile {
  fn to_hash_target(&self) -> Result<Vec<u8>> {
    fs::read(&self.path).map_err(to_error)
  }
}

impl Lockfile {
  pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
    let base_dir = base_dir.as_ref().to_path_buf();
    Lockfile::try_to_read_lockfile(base_dir).map(|(kind, path)| Self { kind, path })
  }

  fn try_to_read_lockfile(base_dir: PathBuf) -> Result<(PackageManagerKind, PathBuf)> {
    let kinds = PackageManagerKind::iter()
      .filter_map(|kind| {
        kind.to_lockfile_names().iter().find_map(|lockfile| {
          let path = base_dir.join(*lockfile);
          path.exists().then_some((kind, path))
        })
      })
      .collect::<HashMap<_, _>>();

    match kinds.len() {
      0..=1 => kinds
        .iter()
        .next()
        .map(|(kind, path)| (*kind, path.clone()))
        .ok_or(Error::NoLockfile(base_dir).into()),
      2 =>
      // priority to Bun if lockfiles for Bun and Yarn coexist
      // Bun has a option to generate yarn.lock (v1) as said in https://bun.sh/docs/install/lockfile
      {
        both_and_then(
          kinds.get_key_value(&PackageManagerKind::Bun),
          kinds.contains_key(&PackageManagerKind::Yarn).then_some(()),
        )
        .map(|((kind, path), _)| (*kind, path.clone()))
        .ok_or(
          Error::MultipleLockfiles(base_dir, kinds.into_values().sorted().collect_vec()).into(),
        )
      }
      _ => {
        Err(Error::MultipleLockfiles(base_dir, kinds.into_values().sorted().collect_vec()).into())
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{test_each, utils::hash::Hash};

  struct NewTestCase {
    input: &'static str,
    expected: (PackageManagerKind, PathBuf),
  }

  fn test_new_each(case: NewTestCase) {
    let lockfile = Lockfile::new(case.input).unwrap();
    assert_eq!(lockfile.kind, case.expected.0);
    assert_eq!(lockfile.path, case.expected.1);
  }

  test_each!(
    test_new,
    test_new_each,
    "npm" => NewTestCase {
      input: "./tests/fixtures/lockfile/npm",
      expected: (
        PackageManagerKind::Npm,
        PathBuf::from("./tests/fixtures/lockfile/npm/package-lock.json")
      ),
    },
    "yarn" => NewTestCase {
      input: "./tests/fixtures/lockfile/yarn",
      expected: (
        PackageManagerKind::Yarn,
        PathBuf::from("./tests/fixtures/lockfile/yarn/yarn.lock")
      ),
    },
    "pnpm" => NewTestCase {
      input: "./tests/fixtures/lockfile/pnpm",
      expected: (
        PackageManagerKind::Pnpm,
        PathBuf::from("./tests/fixtures/lockfile/pnpm/pnpm-lock.yaml")
      ),
    },
    "bun" => NewTestCase {
      input: "./tests/fixtures/lockfile/bun",
      expected: (
        PackageManagerKind::Bun,
        PathBuf::from("./tests/fixtures/lockfile/bun/bun.lockb")
      ),
    },
    "bun_yarn" => NewTestCase {
      input: "./tests/fixtures/lockfile/bun_yarn",
      expected: (
        PackageManagerKind::Bun,
        PathBuf::from("./tests/fixtures/lockfile/bun_yarn/bun.lockb")
      ),
    },
  );

  #[test]
  fn test_new_nope() -> Result<()> {
    let lockfile = Lockfile::new("tests/fixtures/lockfile/nope");
    assert_eq!(
      lockfile.unwrap_err().downcast::<Error>()?,
      Error::NoLockfile(PathBuf::from("tests/fixtures/lockfile/nope"))
    );
    Ok(())
  }

  #[test]
  fn test_new_multiple() {
    let lockfile = Lockfile::new("tests/fixtures/lockfile/multiple");
    assert_eq!(
      lockfile.unwrap_err().downcast::<Error>().unwrap(),
      Error::MultipleLockfiles(
        PathBuf::from("tests/fixtures/lockfile/multiple"),
        [
          "tests/fixtures/lockfile/multiple/bun.lockb",
          "tests/fixtures/lockfile/multiple/package-lock.json",
          "tests/fixtures/lockfile/multiple/pnpm-lock.yaml",
          "tests/fixtures/lockfile/multiple/yarn.lock",
        ]
        .iter()
        .map(PathBuf::from)
        .collect_vec()
      )
    );
  }

  struct GenerateHashTestCase {
    input: &'static str,
    expected: &'static str,
  }

  fn test_generate_hash_each(case: GenerateHashTestCase) {
    let lockfile = Lockfile::new(case.input).unwrap();
    let hash = lockfile.generate_hash().unwrap();
    assert_eq!(hash, Hash(case.expected.to_string()));
  }

  test_each!(
    test_generate_hash,
    test_generate_hash_each,
    "npm" => GenerateHashTestCase {
      input: "./tests/fixtures/lockfile/npm",
      expected: "xdfolu2ud2lvkfhrtup3p33glgtxqg4i",
    },
    "yarn" => GenerateHashTestCase {
      input: "./tests/fixtures/lockfile/yarn",
      expected: "ozwlyhfk4d5zs2zpb4vj2jvhbm3lhg7o",
    },
    "pnpm" => GenerateHashTestCase {
      input: "./tests/fixtures/lockfile/pnpm",
      expected: "4yompzffg476hx3s6eyc2v3ztcbgwfej",
    },
    "bun" => GenerateHashTestCase {
      input: "./tests/fixtures/lockfile/bun",
      expected: "qubt74wrmoca7vuruv3xnfaaclzvmvwm",
    },
  );
}
