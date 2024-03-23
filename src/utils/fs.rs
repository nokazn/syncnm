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
    let original = dir.read_link();
    match &original {
      Ok(original) if original.is_dir() => original
        .canonicalize()
        .map_err(|_| Error::NotAccessible(dir)),
      Ok(_) => Err(Error::NotDir(dir)),
      Err(_) => Err(Error::NotAccessible(dir)),
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
pub fn create_symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let to = to.as_ref().to_path_buf();
  let parent = &to.parent().ok_or(Error::NoEntry(Paths::One(to.clone())))?;
  make_dir_if_not_exists(parent)?;
  fs::remove_dir_all(&to).unwrap_or_default();
  std::os::unix::fs::symlink(&from, &to).map_err(to_error)
}

#[cfg(windows)]
pub fn create_symlink(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let to = to.as_ref().to_path_buf();
  let parent = &to.parent().ok_or(Error::NoEntry(Paths::One(to.clone())))?;
  make_dir_if_not_exists(parent)?;
  fs::remove_dir_all(&to).unwrap_or_default();
  std::os::windows::fs::symlink_dir(&from, &to).map_err(to_error)
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
  use crate::utils::path::{clean_path_separator, try_to_run_in_base_dir};
  use crate::utils::result::convert_panic_to_result;

  use super::*;

  struct ExistsDirTestCase {
    dir: &'static str,
    expected: Result<PathBuf>,
  }

  test_each_serial!(
    test_exists_dir,
    (|case: &ExistsDirTestCase| {
      let result = exists_dir(clean_path_separator(case.dir));
      let expected = case.expected.as_ref().map(clean_path_separator);
      match expected {
        Ok(expected) => {
          let expected_original = if expected.is_symlink() {
            expected.read_link().unwrap()
          } else {
            expected
          };
          assert_eq!(
            result.unwrap(),
            expected_original.canonicalize().unwrap()
          )
        },
        Err(error) => assert_eq!(&result.unwrap_err(), error)
      }

    }),
    "dir" => &ExistsDirTestCase {
      dir: "tests/fixtures/utils/fs/exists_dir/dir1",
      expected: Ok(PathBuf::from("tests/fixtures/utils/fs/exists_dir/dir1")),
    },
    "symlink_dir" => {
      #[cfg(windows)]
      {
        // TODO: fix this test
        // &ExistsDirTestCase {
        //   dir: "tests/fixtures/utils/fs/exists_dir/dir2_windows.lnk",
        //   expected: Ok(PathBuf::from("tests/fixtures/utils/fs/exists_dir/dir2_windows.lnk")),
        // }
        &ExistsDirTestCase {
          dir: "tests/fixtures/utils/fs/exists_dir/dir1",
          expected: Ok(PathBuf::from("tests/fixtures/utils/fs/exists_dir/dir1")),
        }
      }
      #[cfg(not(windows))]
      {
        &ExistsDirTestCase {
          dir: "tests/fixtures/utils/fs/exists_dir/dir2_unix",
          expected: Ok(PathBuf::from("tests/fixtures/utils/fs/exists_dir/dir2_unix")),
        }
      }
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

    let result = try_to_run_in_base_dir(&base_dir, || {
      // rename file
      let base_dir1 = base_dir.join("foo/bar");
      make_dir_if_not_exists(&base_dir1)?;
      let from = base_dir1.join("file1");
      fs::File::create(&from).map_err(to_error)?;
      let to = base_dir1.join("file2");
      convert_panic_to_result(|| {
        assert!(from.exists());
        assert!(!to.exists());
      })?;
      rename(&from, &to)?;
      convert_panic_to_result(|| {
        assert!(!from.exists());
        assert!(to.exists());
      })?;

      // create destination parent directory if not exists
      let base_dir2 = base_dir.join("baz");
      let from = to;
      let to = base_dir2.join("file3");
      convert_panic_to_result(|| {
        assert!(!base_dir2.exists());
        assert!(from.exists());
        assert!(!to.exists());
      })?;
      rename(&from, &to)?;
      convert_panic_to_result(|| {
        assert!(base_dir2.is_dir());
        assert!(!from.exists());
        assert!(to.exists());
      })?;

      // overwrite destination
      let from = base_dir2.join("file4");
      fs::File::create(&from).map_err(to_error)?;
      convert_panic_to_result(|| {
        assert!(from.exists());
        assert!(to.exists());
      })?;
      rename(&from, &to)?;
      convert_panic_to_result(|| {
        assert!(!from.exists());
        assert!(to.exists());
      })?;

      Ok(())
    });

    fs::remove_dir_all(&base_dir).unwrap();
    assert_eq!(result, Ok(()));
  }

  // TODO* fix this test for windows
  #[test]
  #[cfg(not(windows))]
  #[serial_test::serial]

  fn test_create_symlink() {
    let base_dir = temp_dir()
      .canonicalize()
      .unwrap()
      .join("fs/test_create_symlink");
    make_dir_if_not_exists(&base_dir).unwrap();

    let result = try_to_run_in_base_dir(&base_dir, || {
      let base_dir1 = base_dir.join(clean_path_separator("foo/bar"));
      make_dir_if_not_exists(&base_dir1)?;
      let from = base_dir1.join("file1");
      fs::File::create(&from).map_err(to_error)?;
      let to = base_dir1.join("file2");
      convert_panic_to_result(|| {
        assert!(from.is_file());
        assert!(!to.exists());
      })?;
      create_symlink(&from, &to)?;
      convert_panic_to_result(|| {
        assert!(from.is_file());
        assert!(to.is_symlink());
      })?;

      // create destination parent directory if not exists
      let base_dir2 = base_dir.join("baz");
      let to = base_dir2.join("file3");
      convert_panic_to_result(|| {
        assert!(!base_dir2.exists());
        assert!(from.is_file());
        assert!(!to.exists());
      })?;
      create_symlink(&from, &to)?;
      convert_panic_to_result(|| {
        assert!(base_dir2.is_dir());
        assert!(from.is_file());
        assert!(to.is_symlink());
      })?;

      // overwrite destination
      let from = base_dir2.join("file4");
      fs::File::create(&from).map_err(to_error)?;
      convert_panic_to_result(|| {
        assert!(from.is_file());
        assert!(to.is_symlink());
        assert_ne!(
          from.canonicalize().unwrap(),
          to.read_link().unwrap().canonicalize().unwrap()
        );
      })?;
      create_symlink(&from, &to)?;
      convert_panic_to_result(|| {
        assert!(from.exists());
        assert!(to.is_symlink());
        assert_eq!(
          from.canonicalize().unwrap(),
          to.read_link().unwrap().canonicalize().unwrap()
        );
      })?;
      Ok(())
    });

    fs::remove_dir_all(&base_dir).unwrap();
    assert_eq!(result, Ok(()));
  }
}
