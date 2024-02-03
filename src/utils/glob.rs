use glob::glob;
use std::path::PathBuf;

const NEGATE: char = '!';

pub fn collect(
  base_dir: &PathBuf,
  patterns: Option<Vec<String>>,
  enable_negate: bool,
) -> Vec<PathBuf> {
  let mut include = Vec::<PathBuf>::new();
  let mut exclude = Vec::<PathBuf>::new();
  for pattern in patterns.unwrap_or_default() {
    if let (Some(matched), negate) = resolve_glob(&base_dir, pattern, enable_negate) {
      if negate {
        exclude.extend(matched);
      } else {
        include.extend(matched);
      }
    }
  }
  include
    .iter()
    .filter(|path| !exclude.contains(path))
    .cloned()
    .collect::<Vec<_>>()
}

fn resolve_glob(
  base_dir: &PathBuf,
  pattern: String,
  enable_negate: bool,
) -> (Option<Vec<PathBuf>>, bool) {
  let (pattern, negate) = parse_negate(pattern, enable_negate);
  let pattern = base_dir.join(pattern);
  match glob(&pattern.to_string_lossy().as_ref()) {
    Ok(entries) => {
      let entries = entries
        .filter_map(|entry| {
          if let Err(error) = &entry {
            log::warn!("Cannot access to a file or directory: {:?}", error.path());
          }
          entry.ok()
        })
        .collect::<Vec<_>>();
      (Some(entries), negate)
    }
    Err(error) => {
      log::warn!("Invalid glob pattern: {:?}", error);
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
  #[test]
  fn test_parse_negate() {
    use super::*;
    struct TestCase {
      input: (String, bool),
      expected: (String, bool),
    }

    [
      TestCase {
        input: (String::from("foo"), true),
        expected: (String::from("foo"), false),
      },
      TestCase {
        input: (String::from("!foo"), true),
        expected: (String::from("foo"), true),
      },
      TestCase {
        input: (String::from("!!foo"), true),
        expected: (String::from("foo"), false),
      },
      TestCase {
        input: (String::from("!!!foo"), true),
        expected: (String::from("foo"), true),
      },
      TestCase {
        input: (String::from("foo!bar"), true),
        expected: (String::from("foo!bar"), false),
      },
      TestCase {
        input: (String::from("foo!!!!!!!!!!bar"), true),
        expected: (String::from("foo!!!!!!!!!!bar"), false),
      },
    ]
    .iter()
    .for_each(|case| {
      assert_eq!(
        parse_negate(case.input.0.clone(), case.input.1),
        case.expected
      );
    });
  }
}
