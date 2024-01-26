use path_clean::PathClean;
use std::{
  env::current_dir,
  path::{Path, PathBuf},
};

pub fn to_absolute_path<T: AsRef<Path>>(path: T) -> PathBuf {
  let path = path.as_ref();
  let absolute_path = if path.is_absolute() {
    path.to_path_buf()
  } else {
    current_dir().unwrap_or_default().join(path)
  };
  PathClean::clean(&absolute_path)
}
