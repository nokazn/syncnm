use path_clean::PathClean;
use std::{
  env::{current_dir, set_current_dir},
  io,
  path::{Path, PathBuf},
};

use crate::errors::Error;
use crate::errors::Paths;

pub fn to_absolute_path<T: AsRef<Path>>(path: T) -> Result<PathBuf, Error> {
  let path = path.as_ref();
  let absolute_path = if path.is_absolute() {
    path.to_path_buf()
  } else {
    current_dir()
      .map_err(|error| {
        Error::NotAccessibleError(Paths::One(PathBuf::from("./")))
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
    Error::NotAccessibleError(Paths::One(base_dir.as_ref().to_path_buf()))
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
