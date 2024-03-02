use std::{
  fs,
  path::{Path, PathBuf},
};

use crate::{
  core::Result,
  errors::{to_error, Error, Paths},
};

pub fn exists_dir(dir: impl AsRef<Path>) -> Result<PathBuf> {
  let dir = dir.as_ref().to_path_buf();
  if dir.is_symlink() {
    if dir.read_link().map(|p| p.is_dir()).unwrap_or_default() {
      Ok(dir)
    } else {
      Err(Error::NotDir(dir))
    }
  } else if dir.is_dir() {
    dir.canonicalize().map_err(|_| Error::NotAccessible(dir))
  } else {
    Err(Error::NotDir(dir))
  }
}

pub fn make_dir_if_not_exists(dir: impl AsRef<Path>) -> Result<()> {
  fs::create_dir_all(dir).map_err(to_error)
}

pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let to = to.as_ref().to_path_buf();
  let parent = &to.parent().ok_or(Error::NoEntry(Paths::One(to.clone())))?;
  make_dir_if_not_exists(parent)?;
  fs::remove_dir_all(&to).unwrap_or_default();
  fs::rename(&from, &to).map_err(to_error)
}

#[cfg(unix)]
pub fn create_symlink_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let to = to.as_ref().to_path_buf();
  let parent = &to.parent().ok_or(Error::NoEntry(Paths::One(to.clone())))?;
  make_dir_if_not_exists(parent)?;
  std::os::unix::fs::symlink(&from, &to).map_err(to_error)
}

#[cfg(windows)]
pub fn create_symlink_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let to = to.as_ref().to_path_buf();
  let parent = &to.parent().ok_or(Error::NoEntry(Paths::One(to.clone())))?;
  make_dir_if_not_exists(parent)?;
  std::os::windows::fs::symlink_dir(&from, &to).unwrap();
}

pub fn read_to_string(file_path: impl AsRef<Path>) -> Result<String> {
  fs::read_to_string(file_path).map_err(to_error)
}

pub fn write(file_path: impl AsRef<Path>, contents: impl AsRef<[u8]>) -> Result<()> {
  fs::write(&file_path, &contents).map_err(to_error)
}

#[cfg(test)]
mod tests {
  use std::env::temp_dir;

  use crate::test_each_serial;
  use crate::utils::path::{to_absolute_path, try_to_run_in_base_dir};
  use crate::utils::result::convert_panic_to_result;

  use super::*;

  struct ExistsDirTestCase {
    dir: &'static str,
    expected: Result<PathBuf>,
  }

  test_each_serial!(
    test_exists_dir,
    (|case: &ExistsDirTestCase| {
      let result = exists_dir(case.dir);
      assert_eq!(result, case.expected);
    }),
    "dir" => &ExistsDirTestCase {
      dir: "tests/fixtures/utils/fs/exists_dir/dir1",
      expected: to_absolute_path("tests/fixtures/utils/fs/exists_dir/dir1"),
    },
    "symlink_dir" => &ExistsDirTestCase {
      dir: "tests/fixtures/utils/fs/exists_dir/dir2",
      expected: Ok(PathBuf::from("tests/fixtures/utils/fs/exists_dir/dir2")),
    },
    "symlink_file" => &ExistsDirTestCase {
      dir: "tests/fixtures/utils/fs/exists_dir/file1",
      expected: Err(Error::NotDir(PathBuf::from("tests/fixtures/utils/fs/exists_dir/file1"))),
    },
    "none" => &ExistsDirTestCase {
      dir: "tests/fixtures/utils/fs/exists_dir/none",
      expected: Err(Error::NotDir(PathBuf::from("tests/fixtures/utils/fs/exists_dir/none"))),
    },
  );

  #[test]
  #[serial_test::serial]
  fn test_rename() {
    let base_dir = temp_dir().canonicalize().unwrap().join("fs/test_rename");
    make_dir_if_not_exists(&base_dir).unwrap();
    let after_all = |_| fs::remove_dir_all(&base_dir).unwrap();

    let result = try_to_run_in_base_dir(&base_dir, || {
      let base_dir1 = base_dir.join("dir1/dir2");
      make_dir_if_not_exists(&base_dir1)?;
      let from = base_dir1.join("file1");
      fs::File::create(&from).map_err(to_error)?;
      convert_panic_to_result(|| assert!(from.exists()))?;

      let to = base_dir1.join("file2");
      rename(&from, &to)?;
      convert_panic_to_result(|| {
        assert!(!from.exists());
        assert!(to.exists());
      })?;

      Ok(())
    })
    .map_err(after_all);
    assert_eq!(result, Ok(()));
  }
}
