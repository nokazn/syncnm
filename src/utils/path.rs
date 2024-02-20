use std::{
  env::{current_dir, set_current_dir},
  io,
  path::{Path, PathBuf},
};

use path_clean::PathClean;

use crate::{core::Result, errors::Error};

pub fn to_absolute_path(path: impl AsRef<Path>) -> Result<PathBuf> {
  let path = path.as_ref();
  let absolute_path = if path.is_absolute() {
    path.to_path_buf()
  } else {
    current_dir()
      .map_err(|error| {
        Error::NotAccessibleError(PathBuf::from("./"))
          .log_debug(error)
          .log_warn(None)
      })?
      .join(path)
  };
  Ok(PathClean::clean(&absolute_path))
}

/// Run a function `f` in the base directory `base_dir`, and go back to the original cwd.
pub fn run_in_base_dir<T, U, F>(base_dir: T, f: F, fallback: Option<U>) -> F::Output
where
  T: AsRef<Path>,
  F: FnOnce() -> U,
  U: Default,
{
  let on_io_error = |error: io::Error| {
    Error::NotAccessibleError(base_dir.as_ref().to_path_buf())
      .log_debug(&error)
      .log_error(None);
    fallback.unwrap_or_default()
  };

  let cwd = match current_dir() {
    Ok(cwd) => cwd,
    Err(error) => {
      return on_io_error(error);
    }
  };

  if let Err(error) = set_current_dir(&base_dir) {
    return on_io_error(error);
  };

  let result = f();

  if let Err(error) = set_current_dir(&cwd) {
    on_io_error(error);
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;
  use serial_test::serial;
  use std::env::temp_dir;
  use std::path::PathBuf;

  use crate::{test_each_serial, utils::path::to_absolute_path};

  struct ToAbsolutePathTestCase {
    input: &'static str,
    base_dir: Option<PathBuf>,
    expected: PathBuf,
  }

  fn test_to_absolute_path_each(case: &ToAbsolutePathTestCase) {
    if let Some(base_dir) = &case.base_dir {
      run_in_base_dir(
        base_dir,
        || {
          let result = to_absolute_path(&case.input).unwrap();
          assert_eq!(result.starts_with(&base_dir.canonicalize().unwrap()), true);
          assert_eq!(result.ends_with(&case.expected), true);
        },
        None,
      )
    } else {
      assert_eq!(to_absolute_path(&case.input).unwrap(), case.expected);
    }
  }

  test_each_serial!(
    test_to_absolute_path,
    test_to_absolute_path_each,
    "1" => &ToAbsolutePathTestCase {
      input: "/Users/user-a/app/1",
      base_dir: None,
      expected: PathBuf::from("/Users/user-a/app/1"),
    },
    "2" => {
      let tmp_dir = temp_dir();
      &ToAbsolutePathTestCase {
        input: "./foo",
        base_dir: Some(tmp_dir.clone()),
        expected: tmp_dir.canonicalize().unwrap().join("foo"),
      }
    },
  );

  #[test]
  #[serial]
  fn test_run_in_base_dir() {
    let tmp_dir = temp_dir().canonicalize().unwrap();
    let result = run_in_base_dir(
      tmp_dir.clone(),
      || {
        assert_eq!(current_dir().unwrap(), tmp_dir);
        1 + 1
      },
      None,
    );
    assert_eq!(result, 2);
  }
}
