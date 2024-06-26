use std::{collections::BTreeMap, hash::Hash, path::Path};

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
  errors::{to_error, Error},
  project::workspaces::Workspaces,
  utils::hash::Hashable,
};

use super::{
  dependencies::{PackageDependencies, WorkspacePackage},
  package_json::{to_package_json_path, PackageJson},
  package_manager::PackageManagerKind,
};

pub type Dependencies = BTreeMap<String, String>;

/// Ignore `node_modules` directory
pub fn is_valid_base_dir(base_dir: impl AsRef<Path>) -> bool {
  const IGNORED: [&str; 1] = ["node_modules"];
  let path = base_dir.as_ref().to_path_buf();
  if !path.is_dir() {
    return false;
  }
  let path = path.to_string_lossy();
  !IGNORED.iter().any(|ignored| path.contains(ignored))
}

#[derive(Serialize, Deserialize, Hash, Clone, PartialEq, Debug, Default)]
pub struct ProjectRoot {
  original: PackageJson,
  pub kind: PackageManagerKind,
  root: PackageDependencies,
  workspaces: BTreeMap<String, WorkspacePackage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProjectDependencies {
  root: PackageDependencies,
  workspaces: BTreeMap<String, (String, String, PackageDependencies)>,
}

impl Hashable for ProjectRoot {
  fn to_hash_target(&self) -> Result<impl AsRef<[u8]>> {
    let base = &ProjectDependencies {
      root: self.root.clone(),
      workspaces: self
        .workspaces
        .iter()
        .map(|(k, v)| {
          (
            k.to_owned(),
            (
              v.original.name.clone().unwrap_or_default(),
              v.original.version.clone().unwrap_or_default(),
              v.dependencies.clone(),
            ),
          )
        })
        .collect::<BTreeMap<_, _>>(),
    };
    serde_json::to_string(base).map_err(to_error)
  }
}

impl ProjectRoot {
  pub fn new(base_dir: impl AsRef<Path>, kind: Option<PackageManagerKind>) -> Result<Self> {
    let original = PackageJson::new(&base_dir)?;
    let kind = Self::resolve_package_manager_kind(&original, kind);
    if let Some(kind) = kind {
      Self {
        original: original.clone(),
        kind,
        root: PackageDependencies::new(original.clone()),
        workspaces: Self::resolve_workspaces(&base_dir, kind, original.workspaces),
      }
      .validate_package_json_fields(&base_dir)
    } else {
      Err(Error::NoLockfile(base_dir.as_ref().to_path_buf()).into())
    }
  }

  fn resolve_workspaces(
    base_dir: impl AsRef<Path>,
    kind: PackageManagerKind,
    patterns: Option<Vec<String>>,
  ) -> BTreeMap<String, WorkspacePackage> {
    let workspaces = Workspaces::new(base_dir.as_ref().to_path_buf(), kind, patterns);
    let mut workspace_map = BTreeMap::<String, WorkspacePackage>::new();
    for path in workspaces.packages.iter() {
      if let Ok(w) =
        WorkspacePackage::new(path, kind).and_then(|w| w.validate_package_json_fields(path))
      {
        let (name, fallback) = w.get_package_name();
        if workspace_map.get(&name).is_none() {
          workspace_map.insert(name, w);
        } else {
          workspace_map.insert(fallback, w);
        }
      };
    }
    workspace_map
  }

  fn resolve_package_manager_kind(
    original: &PackageJson,
    kind: Option<PackageManagerKind>,
  ) -> Option<PackageManagerKind> {
    let package_manager = match &original.packageManager {
      Some(package_manager) => package_manager,
      None => return kind,
    };
    let regex = {
      let package_managers = PackageManagerKind::iter()
        .filter_map(|kind| kind.to_corepack_name())
        .collect::<Vec<_>>()
        .join("|");
      let s = r"^(".to_owned() + &package_managers + r")(?:@.+)?";
      match Regex::new(&s) {
        Ok(r) => r,
        Err(_) => return kind,
      }
    };
    match regex
      .captures(package_manager)
      .and_then(|c| c.get(1))
      .map(|m| m.as_str())
    {
      Some(p) => PackageManagerKind::iter()
        .find(|kind| {
          if let Some(name) = kind.to_corepack_name() {
            name == p
          } else {
            false
          }
        })
        .or(kind),
      None => kind,
    }
  }

  fn validate_package_json_fields(self, base_dir: impl AsRef<Path>) -> Result<Self> {
    match self.kind {
      PackageManagerKind::Yarn
        if !self
          .original
          .workspaces
          .clone()
          .unwrap_or_default()
          .is_empty()
          && !self.original.private.unwrap_or_default() =>
      {
        Err(
          Error::InvalidPackageJsonPrivateForYarn(to_package_json_path(&base_dir))
            .log_error(None)
            .into(),
        )
      }
      _ => Ok(self),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{fs, path::PathBuf};
  use tempfile::TempDir;

  use super::*;
  use crate::{btree_map, test_each, test_each_serial, utils::path::to_absolute_path};

  struct ToPackageJsonTestCase {
    input: PathBuf,
    expected: PathBuf,
  }

  fn test_to_package_json_path_each(case: ToPackageJsonTestCase) {
    assert_eq!(to_package_json_path(&case.input), case.expected);
  }

  test_each!(
    test_to_package_json_path,
    test_to_package_json_path_each,
    "0" => ToPackageJsonTestCase {
      input: PathBuf::from("./foo"),
      expected: PathBuf::from("./foo/package.json"),
    },
    "1" => ToPackageJsonTestCase {
      input: PathBuf::from("./"),
      expected: PathBuf::from("./package.json"),
    },
    "2" => ToPackageJsonTestCase {
      input: PathBuf::from("/"),
      expected: PathBuf::from("/package.json"),
    },
    "3" => ToPackageJsonTestCase {
      input: PathBuf::from("/foo/bar"),
      expected: PathBuf::from("/foo/bar/package.json"),
    },
    "4" => ToPackageJsonTestCase {
      input: PathBuf::from(""),
      expected: PathBuf::from("package.json"),
    },
  );

  struct IsValidBaseDirTestCase {
    input: &'static str,
    expected: bool,
  }

  fn test_is_valid_base_dir_each(case: IsValidBaseDirTestCase) {
    let temp_dir = TempDir::new().unwrap();
    let base_dir = temp_dir.path().join(case.input);
    fs::create_dir_all(&base_dir).unwrap();
    assert_eq!(is_valid_base_dir(&base_dir), case.expected);
    temp_dir.close().unwrap();
  }

  test_each_serial!(
    test_is_valid_base_dir,
    test_is_valid_base_dir_each,
    "valid_0" => IsValidBaseDirTestCase{
      input: "./",
      expected: true,
    },
    "valid_1" => IsValidBaseDirTestCase{
      input: "./foo/bar/",
      expected: true,
    },
    "invalid_0" => IsValidBaseDirTestCase{
      input: "./node_modules",
      expected: false,
    },
    "invalid_1" => IsValidBaseDirTestCase{
      input: "./foo/bar/packages/node_modules",
      expected: false,
    },
  );

  struct ResolvePackageManagerKindTestCase {
    input: (&'static str, Option<PackageManagerKind>),
    expected: Option<PackageManagerKind>,
  }

  fn test_resolve_package_manager_kind_each(case: ResolvePackageManagerKindTestCase) {
    let original = PackageJson {
      packageManager: Some(String::from(case.input.0)),
      ..Default::default()
    };
    assert_eq!(
      ProjectRoot::resolve_package_manager_kind(&original, case.input.1),
      case.expected,
    );
  }

  test_each_serial!(
    test_resolve_package_manager_kind,
    test_resolve_package_manager_kind_each,
    "npm" => ResolvePackageManagerKindTestCase {
      input: ("npm", Some(PackageManagerKind::Bun)),
      expected: Some(PackageManagerKind::Npm),
    },
    "invalid_npm" => ResolvePackageManagerKindTestCase {
      input: (" npm", Some(PackageManagerKind::Bun)),
      expected: Some(PackageManagerKind::Bun),
    },
    "npm_with_valid_version_1" => ResolvePackageManagerKindTestCase {
      input: ("npm@7.0.0", Some(PackageManagerKind::Bun)),
      expected: Some(PackageManagerKind::Npm),
    },
    "detect_as_npm_with_invalid_version_1" => ResolvePackageManagerKindTestCase {
      input: ("npm@", Some(PackageManagerKind::Bun)),
      expected: Some(PackageManagerKind::Npm),
    },
    "yarn" => ResolvePackageManagerKindTestCase {
      input: ("yarn", Some(PackageManagerKind::Bun)),
      expected: Some(PackageManagerKind::Yarn),
    },
    "yarn_with_valid_version_1" => ResolvePackageManagerKindTestCase {
      input: ("yarn@4.1.0", Some(PackageManagerKind::Bun)),
      expected: Some(PackageManagerKind::Yarn),
    },
    "yarn_with_valid_version_2" => ResolvePackageManagerKindTestCase {
      input: ("yarn@4.1.0+sha256.81a00df816059803e6b5148acf03ce313cad36b7f6e5af6efa040a15981a6ffb", Some(PackageManagerKind::Bun)),
      expected: Some(PackageManagerKind::Yarn),
    },
    "pnpm" => ResolvePackageManagerKindTestCase {
      input: ("pnpm", Some(PackageManagerKind::Bun)),
      expected: Some(PackageManagerKind::Pnpm),
    },
    "pnpm_with_valid_version" => ResolvePackageManagerKindTestCase {
      input: ("pnpm@9.0.0-alpha.4+sha256.2dfc103b0859426dc338ab2796cad7bf83ffb92be0fdd79f65f26ffeb5114ce2", Some(PackageManagerKind::Bun)),
      expected: Some(PackageManagerKind::Pnpm),
    },
    "detect_as_pnpm_with_invalid_semver" => ResolvePackageManagerKindTestCase {
      input: ("pnpm@8", Some(PackageManagerKind::Bun)),
      expected: Some(PackageManagerKind::Pnpm),
    },
    "bun_is_not_supported_by_corepack" => ResolvePackageManagerKindTestCase {
      input: ("bun", Some(PackageManagerKind::Npm)),
      expected: Some(PackageManagerKind::Npm),
    },
  );

  struct NewTestCase {
    input: (PathBuf, Option<PackageManagerKind>),
    expected: Result<ProjectRoot>,
  }

  fn test_new_each(case: NewTestCase) {
    let base_dir = case.input.0;
    let project_root = ProjectRoot::new(base_dir.clone(), case.input.1);
    if let Ok(expected) = case.expected {
      let project_root = project_root.unwrap();
      assert_eq!(project_root.original, expected.original);
      assert_eq!(project_root.kind, expected.kind);
      assert_eq!(project_root.root, expected.root);
      assert_eq!(project_root.workspaces, expected.workspaces);
    } else {
      assert_eq!(
        project_root.unwrap_err().downcast::<Error>().unwrap(),
        case.expected.unwrap_err().downcast::<Error>().unwrap()
      );
    }
  }

  test_each_serial!(
    test_new,
    test_new_each,
    "npm" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/npm"),
        Some(PackageManagerKind::Npm),
      ),
      expected: {
        let dev_dependencies = btree_map!(
          String::from("typescript") => String::from("^5.3.3"),
        );
        Ok(ProjectRoot {
          original: PackageJson {
            workspaces: Some(vec![
              String::from("packages/*"),
              String::from("!packages/c")
            ]),
            devDependencies: Some(dev_dependencies.clone()),
            ..Default::default()
          },
          kind: PackageManagerKind::Npm,
          root: PackageDependencies {
            dev_dependencies,
            ..Default::default()
          },
          workspaces: btree_map!(
            String::from("a") => WorkspacePackage {
              base_dir: to_absolute_path("tests/fixtures/workspaces/npm/packages/a").unwrap(),
              ..Default::default()
            },
          ),
        })
      }
    },
    "yarn" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/yarn"),
        Some(PackageManagerKind::Yarn),
      ),
      expected: {
        let dev_dependencies = btree_map!(
          String::from("typescript") => String::from("^5.3.3"),
        );
        Ok(ProjectRoot {
          original: PackageJson {
            workspaces: Some(vec![String::from("packages/*"), String::from("!packages/c")]),
            private: Some(true),
            devDependencies: Some(dev_dependencies.clone()),
            ..Default::default()
          },
          kind: PackageManagerKind::Yarn,
          root: PackageDependencies {
            dev_dependencies,
            ..Default::default()
          },
          workspaces: btree_map!(
            String::from("@yarn/a") => WorkspacePackage {
              original: PackageJson {
                name: Some(String::from("@yarn/a")),
                version: Some(String::from("0.1.0")),
                ..Default::default()
              },
              kind: PackageManagerKind::Yarn,
              base_dir: to_absolute_path("tests/fixtures/workspaces/yarn/packages/a").unwrap(),
              ..Default::default()
            },
            String::from("@yarn/c") => WorkspacePackage {
              original: PackageJson {
                name: Some(String::from("@yarn/c")),
                version: Some(String::from("0.0.0")),
                ..Default::default()
              },
              kind: PackageManagerKind::Yarn,
              base_dir: to_absolute_path("tests/fixtures/workspaces/yarn/packages/c").unwrap(),
              ..Default::default()
            },
          ),
        })
      }
    },
    "yarn_private_false" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/yarn_private_false"),
        Some(PackageManagerKind::Yarn),
      ),
      expected:Err(Error::InvalidPackageJsonPrivateForYarn(PathBuf::from("tests/fixtures/workspaces/yarn_private_false/package.json")).into())
    },
    "pnpm" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/pnpm"),
        Some(PackageManagerKind::Pnpm),
      ),
      expected: {
        let dev_dependencies = btree_map!(
          String::from("typescript") => String::from("^5.3.3"),
        );
        Ok(ProjectRoot {
          original: PackageJson {
            devDependencies: Some(dev_dependencies.clone()),
            ..Default::default()
          },
          kind: PackageManagerKind::Pnpm,
          root: PackageDependencies {
            dev_dependencies,
            ..Default::default()
          },
          workspaces: btree_map!(
            String::from("a") => WorkspacePackage {
              original: PackageJson {
                ..Default::default()
              },
              kind: PackageManagerKind::Pnpm,
              base_dir: to_absolute_path("tests/fixtures/workspaces/pnpm/packages/a").unwrap(),
              ..Default::default()
            },
          ),
        })
      }
    },
    "bun" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/bun"),
        Some(PackageManagerKind::Bun),
      ),
      expected: {
        let dev_dependencies = btree_map!(
          String::from("typescript") => String::from("^5.3.3"),
        );
        Ok(ProjectRoot {
          original: PackageJson {
            workspaces: Some(vec![String::from("packages/*")]),
            devDependencies: Some(dev_dependencies.clone()),
            ..Default::default()
          },
          kind: PackageManagerKind::Bun,
          root: PackageDependencies {
            dev_dependencies,
            ..Default::default()
          },
          workspaces: btree_map!(
            String::from("@bun/a") => WorkspacePackage {
              original: PackageJson {
                name: Some(String::from("@bun/a")),
                ..Default::default()
              },
              kind: PackageManagerKind::Bun,
              base_dir: to_absolute_path("tests/fixtures/workspaces/bun/packages/a").unwrap(),
              ..Default::default()
            },
            String::from("@bun/c") => WorkspacePackage {
              original: PackageJson {
                name: Some(String::from("@bun/c")),
                ..Default::default()
              },
              kind: PackageManagerKind::Bun,
              base_dir: to_absolute_path("tests/fixtures/workspaces/bun/packages/c").unwrap(),
              ..Default::default()
            },
          ),
        })
      }
    },
  );
}
