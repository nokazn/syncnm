use std::{
  fs,
  path::{Path, PathBuf},
};

use strum::IntoEnumIterator;

use crate::{
  core::Result,
  errors::{to_error, Error},
  package_manager::PackageManagerKind,
  utils::hash::Hashable,
};

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
    match Lockfile::try_to_read_lockfile(&base_dir) {
      Some((kind, path)) => Ok(Self { kind, path }),
      None => Err(Error::NoLockfile(base_dir)),
    }
  }

  fn try_to_read_lockfile(base_dir: impl AsRef<Path>) -> Option<(PackageManagerKind, PathBuf)> {
    for kind in PackageManagerKind::iter() {
      for lockfile in kind.to_lockfile_names() {
        let path = base_dir.as_ref().join(lockfile);
        if path.exists() {
          return Some((kind, path));
        }
      }
    }
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    test_each,
    utils::{hash::Hash, path::to_absolute_path},
  };

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
  );

  #[test]
  fn test_new_nope() {
    let lockfile = Lockfile::new("tests/fixtures/lockfile/nope");
    assert_eq!(
      lockfile.unwrap_err().to_string(),
      format!(
        "No lockfile at: `{}`",
        to_absolute_path("tests/fixtures/lockfile/nope")
          .unwrap()
          .to_string_lossy()
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
