use base64ct::{Base64, Encoding};
use sha2::{Digest, Sha256};
use std::{
  fs, io,
  path::{Path, PathBuf},
};
use strum::IntoEnumIterator;

use crate::{
  core::{PackageManagerKind, Result},
  errors::Error,
};

#[derive(Debug, PartialEq)]
pub struct Lockfile {
  pub kind: PackageManagerKind,
  path: PathBuf,
}

impl Lockfile {
  pub fn new(dir_path: impl AsRef<Path>) -> Result<Self> {
    match Lockfile::try_to_read_lockfile(&dir_path) {
      Some((kind, path)) => Ok(Self { kind, path }),
      None => Err(Error::NoLockfileError(dir_path.as_ref().to_path_buf())),
    }
  }

  fn try_to_read_lockfile(dir_path: impl AsRef<Path>) -> Option<(PackageManagerKind, PathBuf)> {
    for kind in PackageManagerKind::iter() {
      for lockfile in kind.to_lockfile_names() {
        let file_path = dir_path.as_ref().join(lockfile);
        if file_path.exists() {
          return Some((kind, file_path));
        }
      }
    }
    None
  }

  pub fn generate_hash(&self) -> Result<String> {
    let mut file = fs::File::open(&self.path).map_err(|error| Error::Any(error.to_string()))?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).map_err(|error| Error::Any(error.to_string()))?;
    let raw_hash = hasher.finalize();
    let hash = Base64::encode_string(&raw_hash);
    Ok(hash)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{core::PackageManagerKind, test_each, utils::path::to_absolute_path};

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
    assert_eq!(hash, case.expected);
  }

  test_each!(
    test_generate_hash,
    test_generate_hash_each,
    "npm" => GenerateHashTestCase {
      input: "./tests/fixtures/lockfile/npm",
      expected: "uMrl01Qel1UU8Z0ft+9mWad4G4g0Vjwye3+gVGi7FNM=",
    },
    "yarn" => GenerateHashTestCase {
      input: "./tests/fixtures/lockfile/yarn",
      expected: "dmy8HKrg+5lrLw8qnSanCzazm+5zgxA6la9Z2zh7GJ0=",
    },
    "pnpm" => GenerateHashTestCase {
      input: "./tests/fixtures/lockfile/pnpm",
      expected: "5hzH5KU3P+PfcvEwLVd5mIJrFInY5SfHCCeoPCspqUs=",
    },
    "bun" => GenerateHashTestCase {
      input: "./tests/fixtures/lockfile/bun",
      expected: "hQM/8tFjhA/WkaV3dpQAEvNWVsyou1GpqyhIxwfKiTA=",
    },
  );
}
