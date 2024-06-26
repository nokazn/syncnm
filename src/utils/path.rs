use std::{
  env::{current_dir, set_current_dir},
  fmt::Display,
  io,
  path::{Component, Path, PathBuf},
};

use anyhow::Result;
use itertools::Itertools;
use path_clean::PathClean;

#[cfg(test)]
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::errors::Error;

pub fn to_absolute_path(path: impl AsRef<Path>) -> Result<PathBuf> {
  let path = path.as_ref();
  let absolute_path = if path.is_absolute() {
    path.to_path_buf()
  } else {
    current_dir()
      .map_err(|error| {
        Error::NotAccessible(PathBuf::from("./"))
          .log_debug(error)
          .log_warn(None)
      })?
      .join(path)
  };
  Ok(PathClean::clean(&absolute_path))
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct DirKey(pub String);

impl Display for DirKey {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

pub fn to_dir_key(path: impl AsRef<Path>) -> DirKey {
  type ToDirComponent = Box<dyn FnMut(&Component<'_>) -> Option<String>>;
  let mut to_dir_component: ToDirComponent = {
    let home_dir = dirs::home_dir();
    match home_dir {
      Some(home_dir) => {
        let to_os_ascii_lowercase = |c: Component<'_>| c.as_os_str().to_ascii_lowercase();
        let home_components = home_dir
          .components()
          .map(to_os_ascii_lowercase)
          .collect_vec();
        Box::new(move |c: &Component<'_>| match c {
          Component::Normal(p) => {
            if home_components.contains(&to_os_ascii_lowercase(*c)) {
              None
            } else {
              Some(p.to_string_lossy().to_string())
            }
          }
          _ => None,
        })
      }
      None => Box::new(|c: &Component<'_>| match c {
        Component::Normal(p) => Some(p.to_string_lossy().to_string()),
        _ => None,
      }),
    }
  };
  let path = {
    let p = path.as_ref();
    to_absolute_path(p).unwrap_or(p.to_path_buf())
  };
  DirKey(
    path
      .components()
      .filter_map(|c| to_dir_component(&c))
      .join("_"),
  )
}

#[cfg(test)]
pub fn clean_path_separator(path: impl AsRef<Path>) -> PathBuf {
  let path = path.as_ref().to_string_lossy().to_string();
  #[cfg(unix)]
  {
    // if this passes tests, this unwrap never fails
    let r = Regex::new(r"(?:\\)+").unwrap();
    PathBuf::from(r.replace_all(&path, "/").to_string())
  }

  #[cfg(windows)]
  {
    // if this passes tests, this unwrap never fails
    let separator = Regex::new(r"(^|[^\\])/+").unwrap();
    let prefix = Regex::new(r"^\\{2}\?\\").unwrap();
    let path = separator.replace_all(&path, "${1}\\");
    let path = prefix.replace(&path, "");
    PathBuf::from(path.to_string())
  }
}

#[cfg(test)]
#[cfg(windows)]
pub fn remove_windows_path_prefix(path: impl AsRef<Path>) -> PathBuf {
  #[cfg(windows)]
  {
    let path = path.as_ref().to_string_lossy().to_string();
    // if this passes tests, this unwrap never fails
    let r = Regex::new(r"^[a-zA-Z]:").unwrap();
    PathBuf::from(r.replace(&path, "").to_string())
  }
  #[cfg(unix)]
  {
    path.as_ref().to_path_buf()
  }
}

/// Run a function `f` in the base directory `base_dir`, and go back to the original cwd.
pub fn run_in_base_dir<T, F>(base_dir: impl AsRef<Path>, f: F, fallback: Option<T>) -> F::Output
where
  T: Default,
  F: FnOnce() -> T,
{
  let on_io_error = |error: io::Error| {
    Error::NotAccessible(base_dir.as_ref().to_path_buf())
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
  if let Err(error) = set_current_dir(cwd) {
    on_io_error(error);
  }
  result
}

#[cfg(test)]
pub fn try_to_run_in_base_dir<T, F>(base_dir: impl AsRef<Path>, f: F) -> Result<T>
where
  F: FnOnce() -> Result<T>,
{
  let on_io_error = |error: io::Error| {
    Err(
      Error::NotAccessible(base_dir.as_ref().to_path_buf())
        .log_debug(&error)
        .log_error(None)
        .into(),
    )
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
  if let Err(error) = set_current_dir(cwd) {
    let _no_ret = on_io_error(error);
  }
  result
}

#[cfg(test)]
mod tests {
  use super::*;
  use serial_test::serial;
  use std::env::temp_dir;
  use std::path::PathBuf;

  use crate::{test_each, test_each_serial, utils::path::to_absolute_path};

  struct ToAbsolutePathTestCase {
    input: &'static str,
    base_dir: Option<PathBuf>,
    expected: PathBuf,
  }

  #[cfg(windows)]
  fn test_to_absolute_path_each(case: &ToAbsolutePathTestCase) {
    if let Some(base_dir) = &case.base_dir {
      run_in_base_dir(
        base_dir,
        || {
          let result = to_absolute_path(&case.input).unwrap();
          assert!(result.starts_with(&base_dir));
          assert!(result.ends_with(&case.expected));
        },
        None,
      )
    } else {
      assert_eq!(
        remove_windows_path_prefix(to_absolute_path(&case.input).unwrap()),
        case.expected
      );
    }
  }

  #[cfg(windows)]
  test_each_serial!(
    test_to_absolute_path,
    test_to_absolute_path_each,
    "1" => &ToAbsolutePathTestCase {
      input: "\\Users\\user-a\\app\\1",
      base_dir: None,
      expected: PathBuf::from("\\Users\\user-a\\app\\1"),
    },
    "2" => {
      let tmp_dir = temp_dir();
      &ToAbsolutePathTestCase {
        input: ".\\foo",
        base_dir: Some(tmp_dir.clone()),
        expected: tmp_dir.join("foo"),
      }
    },
  );
  #[cfg(not(windows))]
  fn test_to_absolute_path_each(case: &ToAbsolutePathTestCase) {
    if let Some(base_dir) = &case.base_dir {
      run_in_base_dir(
        base_dir,
        || {
          let result = to_absolute_path(&case.input).unwrap();
          assert!(result.starts_with(&base_dir.canonicalize().unwrap()));
          assert!(result.ends_with(&case.expected));
        },
        None,
      )
    } else {
      assert_eq!(to_absolute_path(&case.input).unwrap(), case.expected);
    }
  }

  #[cfg(not(windows))]
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

  struct ToDirKeyTestCase {
    input: PathBuf,
    expected: &'static str,
  }

  fn test_to_dir_key_each(case: &ToDirKeyTestCase) {
    let dir_key = to_dir_key(&case.input).to_string();
    if case.input.is_relative() {
      assert!(dir_key.ends_with(case.expected));
    } else {
      assert_eq!(dir_key, case.expected);
    }
  }

  test_each!(
    test_to_dir_key,
    test_to_dir_key_each,
    "1" => &ToDirKeyTestCase {
      input: dirs::home_dir().unwrap().join("a/b/c/d"),
      expected: "a_b_c_d"
    },
    "2" => &ToDirKeyTestCase {
      input: dirs::home_dir().unwrap().join("."),
      expected: ""
    },
    "3" => &ToDirKeyTestCase {
      input: PathBuf::from("a/b/c/d"),
      expected: "a_b_c_d"
    },
    "4" => &ToDirKeyTestCase {
      input: PathBuf::from("./a/b/c/d"),
      expected: "a_b_c_d"
    },
    "5" => &ToDirKeyTestCase {
      input: PathBuf::from("./a/b/c/../c/d"),
      expected: "a_b_c_d"
    },
  );

  struct CleanPathSeparatorTestCase {
    input: &'static str,
    expected: &'static str,
  }

  #[cfg(windows)]
  test_each!(
    test_clean_path_separator,
    |case: &CleanPathSeparatorTestCase| {
      assert_eq!(
        clean_path_separator(case.input).to_string_lossy(),
        case.expected.to_string()
      );
    },
    "1" => &CleanPathSeparatorTestCase {
      input: "/Users/user-a/app/1",
      expected: "\\Users\\user-a\\app\\1",
    },
    "2" => &CleanPathSeparatorTestCase {
      input: "/Users/user-a////app/1",
      expected: "\\Users\\user-a\\app\\1",
    },
    "3" => &CleanPathSeparatorTestCase {
      input: "./////app/1//////////////",
      expected: ".\\app\\1\\",
    },
    "4" => &CleanPathSeparatorTestCase{
      input: "./////app/1//.////////////",
      expected: ".\\app\\1\\.\\",
    },
  );

  #[cfg(not(windows))]
  test_each!(
    test_clean_path_separator,
    |case: &CleanPathSeparatorTestCase| {
      assert_eq!(
        clean_path_separator(case.input).to_string_lossy(),
        case.expected.to_string()
      );
    },
    "1" => &CleanPathSeparatorTestCase {
      input: "\\Users\\user-a\\app\\1",
      expected: "/Users/user-a/app/1",
    },
    "2" => &CleanPathSeparatorTestCase {
      input: "\\Users\\user-a\\\\app\\1",
      expected: "/Users/user-a/app/1",
    },
    "3" => &CleanPathSeparatorTestCase {
      input: ".\\\\\\\\\\\\app\\\\\\1\\",
      expected: "./app/1/",
    },
    "4" => &CleanPathSeparatorTestCase{
      input: ".\\app\\1\\.\\",
      expected: "./app/1/./",
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
