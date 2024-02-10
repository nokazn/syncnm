use path_clean::PathClean;
use std::{
  env::current_dir,
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
