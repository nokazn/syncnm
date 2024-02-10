use glob::glob;
use itertools::Itertools;
use std::env::set_current_dir;
use std::path::PathBuf;

use crate::errors::{Error, Paths};
use crate::utils::path::to_absolute_path;

const NEGATE: char = '!';

pub fn collect(
  base_dir: &PathBuf,
  patterns: Option<Vec<String>>,
  enable_negate: bool,
) -> Vec<PathBuf> {
  let mut entries = Vec::<PathBuf>::new();
  for pattern in patterns.unwrap_or_default() {
    if let (Some(matched), negate) = resolve_glob(&base_dir, pattern, enable_negate) {
      if negate {
        entries = entries
          .iter()
          .filter_map(|entry| {
            if matched.iter().any(|p| entry.starts_with(p)) {
              None
            } else {
              Some(entry)
            }
          })
          .cloned()
          .collect::<Vec<_>>();
      } else {
        entries.extend(matched);
      }
    }
  }
  let result = entries
    .iter()
    .unique()
    .filter_map(|entry| to_absolute_path(entry).ok())
    .collect::<Vec<_>>();
  result
}

fn resolve_glob(
  base_dir: &PathBuf,
  pattern: String,
  enable_negate: bool,
) -> (Option<Vec<PathBuf>>, bool) {
  if let Err(error) = set_current_dir(&base_dir) {
    Error::NotAccessibleError(Paths::One(base_dir.clone()))
      .log_debug(error)
      .log_warn(None);
    return (None, false);
  }
  let (pattern, negate) = parse_negate(pattern, enable_negate);
  match glob(&pattern) {
    Ok(entries) => {
      let entries = entries
        .filter_map(|entry| {
          if let Err(error) = &entry {
            Error::NotAccessibleError(Paths::One(error.path().to_path_buf()))
              .log_debug(error)
              .log_warn(None);
          }
          entry.ok()
        })
        .collect::<Vec<_>>();
      (Some(entries), negate)
    }
    Err(error) => {
      Error::InvalidGlobPatternError(error).log_warn(None);
      (None, negate)
    }
  }
}

/// npm CLI [uses mimimatch](https://github.com/npm/cli/blob/latest/lib/workspaces/get-workspaces.js) for glob pattern matching, and supports "negates" patterns.
/// See minimatch"s [docs](https://github.com/isaacs/minimatch?tab=readme-ov-file#nonegate) and [implementation](https://github.com/isaacs/minimatch/blob/ef8f2672bdbbf6a632ea815636659fb31b5169aa/src/index.ts#L736-L750) for details.
fn parse_negate(pattern: String, enable_negate: bool) -> (String, bool) {
  if !enable_negate {
    return (pattern, false);
  }
  let (mut negate, mut counts) = (false, 0);
  for char in pattern.chars() {
    if char != NEGATE {
      break;
    }
    negate = !negate;
    counts += 1;
  }
  (pattern[counts..].to_string(), negate)
}

#[cfg(test)]
mod tests {
  use crate::test_each;

  use super::*;
  use std::fs::File;
  use std::path::PathBuf;
  use std::{fs, vec};
  use tempfile::TempDir;

  struct CollectTestCase {
    input: (Vec<String>, bool),
    file_system: Vec<PathBuf>,
    expected: Vec<PathBuf>,
  }

  fn test_collect_each(case: &CollectTestCase) {
    let tmp_dir = TempDir::new().unwrap();
    case.file_system.iter().for_each(|path| {
      fs::create_dir_all(&tmp_dir.path().join(path.parent().unwrap())).unwrap();
      File::create(&tmp_dir.path().join(path)).unwrap();
    });

    assert_eq!(
      collect(
        &tmp_dir.as_ref().to_path_buf(),
        Some(case.input.clone().0),
        case.input.1
      ),
      case
        .expected
        .iter()
        .map(|path| tmp_dir.path().join(path).canonicalize().unwrap())
        .collect::<Vec<_>>()
    );
    tmp_dir.close().unwrap();
  }

  test_each! {
    test_collect,
    0 => &CollectTestCase {
      input: (vec!["foo".to_string()], true),
      file_system: vec![PathBuf::from("./foo")],
      expected: vec![PathBuf::from("./foo")],
    },
    1 => &CollectTestCase {
      input: (vec!["bar".to_string()], true),
      file_system: vec![PathBuf::from("./foo")],
      expected: vec![],
    },
    2 => &CollectTestCase {
      input: (vec!["f*".to_string()], true),
      file_system: vec![PathBuf::from("./foo")],
      expected: vec![PathBuf::from("./foo")],
    },
    3 => &CollectTestCase {
      input: (vec!["*fo*".to_string()], true),
      file_system: vec![PathBuf::from("./foo")],
      expected: vec![PathBuf::from("./foo")],
    },
    4 => &CollectTestCase {
      input: (vec!["**/foo".to_string()], true),
      file_system: vec![PathBuf::from("./foo")],
      expected: vec![PathBuf::from("./foo")],
    },
    5 => &CollectTestCase {
      input: (vec!["**/baz".to_string()], true),
      file_system: vec![PathBuf::from("./foo/bar/baz/qux")],
      expected: vec![PathBuf::from("./foo/bar/baz/")],
    },
    6 => &CollectTestCase {
      input: (vec!["**/bar".to_string()], true),
      file_system: vec![PathBuf::from("./foo/bar")],
      expected: vec![PathBuf::from("./foo/bar")],
    },
    7 => &CollectTestCase {
      input: (vec!["foo".to_string(), "!foo".to_string()], true),
      file_system: vec![PathBuf::from("./foo/bar")],
      expected: vec![],
    },
    8 => &CollectTestCase {
      input: (vec!["!foo".to_string(), "foo".to_string()], true),
      file_system: vec![PathBuf::from("./foo/bar")],
      expected: vec![PathBuf::from("./foo/")],
    },
    9 => &CollectTestCase {
      input: (
        vec!["foo".to_string(), "!foo".to_string(), "bar".to_string()],
        true,
      ),
      file_system: vec![PathBuf::from("./foo"), PathBuf::from("./bar")],
      expected: vec![PathBuf::from("./bar")],
    },
    10 => &CollectTestCase {
      input: (
        vec!["!foo".to_string(), "foo".to_string(), "bar".to_string()],
        true,
      ),
      file_system: vec![PathBuf::from("./foo"), PathBuf::from("./bar")],
      expected: vec![PathBuf::from("./foo"), PathBuf::from("./bar")],
    },
    11 => &CollectTestCase {
      input: (
        vec![
          "foo".to_string(),
          "!foo".to_string(),
          "bar".to_string(),
          "!bar".to_string(),
        ],
        true,
      ),
      file_system: vec![PathBuf::from("./foo"),PathBuf::from("./bar")],
      expected: vec![],
    },
  }

  struct ParseNegateTestCase {
    input: (String, bool),
    expected: (String, bool),
  }

  fn test_parse_negate_each(case: &ParseNegateTestCase) {
    use super::*;

    assert_eq!(
      parse_negate(case.input.0.clone(), case.input.1),
      case.expected
    );
  }

  test_each!(
    test_parse_negate,
    0 => &ParseNegateTestCase {
      input: (String::from("foo"), true),
      expected: (String::from("foo"), false),
    },
    1 => &ParseNegateTestCase {
      input: (String::from("!foo"), true),
      expected: (String::from("foo"), true),
    },
    2 => &ParseNegateTestCase {
      input: (String::from("!!foo"), true),
      expected: (String::from("foo"), false),
    },
    3 => &ParseNegateTestCase {
      input: (String::from("!!!foo"), true),
      expected: (String::from("foo"), true),
    },
    4 => &ParseNegateTestCase {
      input: (String::from("foo!bar"), true),
      expected: (String::from("foo!bar"), false),
    },
    5 => &ParseNegateTestCase {
      input: (String::from("foo!!!!!!!!!!bar"), true),
      expected: (String::from("foo!!!!!!!!!!bar"), false),
    },
  );
}
