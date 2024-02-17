use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
  collections::BTreeMap,
  fs,
  hash::Hash,
  path::{Path, PathBuf},
};
use strum::IntoEnumIterator;

use crate::{
  core::{PackageManagerKind, Result},
  errors::{Error, Paths},
  utils::hash::Hashable,
  workspaces::Workspaces,
};

type Dependencies = BTreeMap<String, String>;

fn to_package_json_path<T: AsRef<Path>>(base_dir: T) -> PathBuf {
  const PACKAGE_JSON: &str = "package.json";
  base_dir.as_ref().join(PACKAGE_JSON)
}

/// Ignore `node_modules` directory
fn is_valid_base_dir<T: AsRef<Path>>(base_dir: T) -> bool {
  const IGNORED: [&str; 1] = ["node_modules"];
  let path = base_dir.as_ref().to_path_buf();
  if !path.is_dir() {
    return false;
  }
  let path = path.to_string_lossy();
  !IGNORED.iter().any(|ignored| path.contains(ignored))
}

///
/// --------------------------------------------------
///
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Hash, Debug, Clone, PartialEq, Default)]
struct PackageJson {
  name: Option<String>,
  version: Option<String>,
  private: Option<bool>,
  packageManager: Option<String>,
  dependencies: Option<Dependencies>,
  devDependencies: Option<Dependencies>,
  peerDependencies: Option<Dependencies>,
  overrides: Option<Dependencies>,
  optionalDependencies: Option<Dependencies>,
  workspaces: Option<Vec<String>>,
}

impl PackageJson {
  fn new<T: AsRef<Path>>(base_dir: T) -> Result<Self> {
    let file_path = to_package_json_path(base_dir);
    let contents = fs::read_to_string(&file_path);
    match contents {
      Ok(contents) => serde_json::from_str::<Self>(&contents)
        .map_err(|_| Error::ParseError(Paths::One(file_path))),
      Err(_) => Err(Error::NoEntryError(Paths::One(file_path))),
    }
  }
}

///
/// --------------------------------------------------
///
#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq, Default)]
struct PackageDependencies {
  dependencies: Dependencies,
  dev_dependencies: Dependencies,
  peer_dependencies: Dependencies,
  overrides: Dependencies,
  optional_dependencies: Dependencies,
}

impl PackageDependencies {
  fn new(raw: PackageJson) -> Self {
    Self {
      dependencies: raw.dependencies.unwrap_or_default(),
      dev_dependencies: raw.devDependencies.unwrap_or_default(),
      peer_dependencies: raw.peerDependencies.unwrap_or_default(),
      optional_dependencies: raw.optionalDependencies.unwrap_or_default(),
      overrides: raw.overrides.unwrap_or_default(),
    }
  }
}

///
/// --------------------------------------------------
///
#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq, Default)]
struct WorkspacePackage {
  original: PackageJson,
  base_dir: PathBuf,
  kind: PackageManagerKind,
  dependencies: PackageDependencies,
}

impl WorkspacePackage {
  fn new<T: AsRef<Path>>(base_dir: T, kind: PackageManagerKind) -> Result<Self> {
    let base_dir = base_dir.as_ref().to_path_buf();
    if !is_valid_base_dir(&base_dir) {
      return Err(Error::InvalidWorkspaceError(base_dir));
    }
    let original = PackageJson::new(&base_dir)?;
    Ok(Self {
      original: original.clone(),
      base_dir,
      kind,
      dependencies: PackageDependencies::new(original),
    })
  }

  fn validate_package_json_fields<T: AsRef<Path>>(self, base_dir: T) -> Result<Self> {
    let package_json_path = to_package_json_path(&base_dir);
    match self.kind {
      PackageManagerKind::Yarn
        if self.original.name.is_none() || self.original.version.is_none() =>
      {
        Err(Error::InvalidPackageJsonFieldsForYarnError(
          package_json_path,
        ))
      }
      PackageManagerKind::Bun if self.original.name.is_none() => Err(
        Error::InvalidPackageJsonFieldsForBunError(package_json_path),
      ),
      _ => Ok(self),
    }
  }

  fn get_package_name(&self) -> (String, String) {
    let name = self.original.name.clone().unwrap_or(
      self
        .base_dir
        .file_name()
        .unwrap_or(self.base_dir.as_os_str())
        .to_string_lossy()
        .to_string(),
    );
    let fallback = self.base_dir.to_string_lossy().to_string();
    (name, fallback)
  }
}

///
/// --------------------------------------------------
///
#[derive(Serialize, Deserialize, Hash, Clone, PartialEq, Debug, Default)]
pub struct ProjectRoot {
  original: PackageJson,
  kind: PackageManagerKind,
  root: PackageDependencies,
  workspaces: BTreeMap<String, WorkspacePackage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProjectDependencies {
  root: PackageDependencies,
  workspaces: BTreeMap<String, (String, String, PackageDependencies)>,
}

impl Hashable for ProjectRoot {
  fn to_bytes(&self) -> serde_json::Result<Vec<u8>> {
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
    serde_json::to_string(base).map(|json| json.into_bytes())
  }
}

impl ProjectRoot {
  pub fn new<T: AsRef<Path>>(base_dir: T, kind: PackageManagerKind) -> Result<Self> {
    let original = PackageJson::new(&base_dir)?;
    Ok(
      Self {
        original: original.clone(),
        kind: Self::resolve_package_manager_kind(&original, kind),
        root: PackageDependencies::new(original.clone()),
        workspaces: Self::resolve_workspaces(&base_dir, kind, original.workspaces),
      }
      .validate_package_json_fields(&base_dir),
    )?
  }

  fn resolve_workspaces<T: AsRef<Path>>(
    base_dir: T,
    kind: PackageManagerKind,
    patterns: Option<Vec<String>>,
  ) -> BTreeMap<String, WorkspacePackage> {
    let workspaces = Workspaces::new(base_dir.as_ref().to_path_buf(), kind, patterns);
    let mut workspace_map = BTreeMap::<String, WorkspacePackage>::new();
    for path in workspaces.packages.iter() {
      if let Ok(w) =
        WorkspacePackage::new(&path, kind).and_then(|w| w.validate_package_json_fields(path))
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
    kind: PackageManagerKind,
  ) -> PackageManagerKind {
    let package_manager = match &original.packageManager {
      Some(package_manager) => package_manager,
      None => return kind,
    };
    let regex = {
      let package_managers = PackageManagerKind::iter()
        .filter_map(|kind| kind.corepack_name())
        .collect::<Vec<_>>()
        .join("|");
      let s = r"^(".to_owned() + &package_managers + r")(?:@.+)?";
      match Regex::new(&s) {
        Ok(r) => r,
        Err(_) => return kind,
      }
    };
    match regex
      .captures(&package_manager)
      .and_then(|c| c.get(1))
      .map(|m| m.as_str())
    {
      Some(p) => PackageManagerKind::iter()
        .find(|kind| {
          if let Some(name) = kind.corepack_name() {
            name == p
          } else {
            false
          }
        })
        .unwrap_or(kind),
      None => kind,
    }
  }

  fn validate_package_json_fields<T: AsRef<Path>>(self, base_dir: T) -> Result<Self> {
    match self.kind {
      PackageManagerKind::Yarn
        if self.original.workspaces.clone().unwrap_or_default().len() > 0
          && !self.original.private.unwrap_or_default() =>
      {
        Err(
          Error::InvalidPackageJsonPrivateForYarnError(to_package_json_path(&base_dir))
            .log_error(None),
        )
      }
      _ => Ok(self),
    }
  }
}

#[cfg(test)]
mod tests {
  use tempfile::TempDir;

  use super::*;
  use crate::{
    btree_map, core::PackageManagerKind, test_each, test_each_serial, utils::path::to_absolute_path,
  };

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
    input: (&'static str, PackageManagerKind),
    expected: PackageManagerKind,
  }

  fn test_resolve_package_manager_kind_each(case: ResolvePackageManagerKindTestCase) {
    let original = PackageJson {
      packageManager: Some(String::from(case.input.0)),
      ..Default::default()
    };
    assert_eq!(
      ProjectRoot::resolve_package_manager_kind(&original, case.input.1),
      PackageManagerKind::from(case.expected)
    );
  }

  test_each!(
    test_resolve_package_manager_kind,
    test_resolve_package_manager_kind_each,
    "npm" => ResolvePackageManagerKindTestCase {
      input: ("npm", PackageManagerKind::Bun),
      expected: PackageManagerKind::Npm,
    },
    "invalid_npm" => ResolvePackageManagerKindTestCase {
      input: (" npm", PackageManagerKind::Bun),
      expected: PackageManagerKind::Bun,
    },
    "npm_with_valid_version_1" => ResolvePackageManagerKindTestCase {
      input: ("npm@7.0.0", PackageManagerKind::Bun),
      expected: PackageManagerKind::Npm,
    },
    "detect_as_npm_with_invalid_version_1" => ResolvePackageManagerKindTestCase {
      input: ("npm@", PackageManagerKind::Bun),
      expected: PackageManagerKind::Npm,
    },
    "yarn" => ResolvePackageManagerKindTestCase {
      input: ("yarn", PackageManagerKind::Bun),
      expected: PackageManagerKind::Yarn,
    },
    "yarn_with_valid_version_1" => ResolvePackageManagerKindTestCase {
      input: ("yarn@4.1.0", PackageManagerKind::Bun),
      expected: PackageManagerKind::Yarn,
    },
    "yarn_with_valid_version_2" => ResolvePackageManagerKindTestCase {
      input: ("yarn@4.1.0+sha256.81a00df816059803e6b5148acf03ce313cad36b7f6e5af6efa040a15981a6ffb", PackageManagerKind::Bun),
      expected: PackageManagerKind::Yarn,
    },
    "pnpm" => ResolvePackageManagerKindTestCase {
      input: ("pnpm", PackageManagerKind::Bun),
      expected: PackageManagerKind::Pnpm,
    },
    "pnpm_with_valid_version" => ResolvePackageManagerKindTestCase {
      input: ("pnpm@9.0.0-alpha.4+sha256.2dfc103b0859426dc338ab2796cad7bf83ffb92be0fdd79f65f26ffeb5114ce2", PackageManagerKind::Bun),
      expected: PackageManagerKind::Pnpm,
    },
    "detect_as_pnpm_with_invalid_semver" => ResolvePackageManagerKindTestCase {
      input: ("pnpm@8", PackageManagerKind::Bun),
      expected: PackageManagerKind::Pnpm,
    },
    "bun_is_not_supported_by_corepack" => ResolvePackageManagerKindTestCase {
      input: ("bun", PackageManagerKind::Npm),
      expected: PackageManagerKind::Npm,
    },
  );

  struct NewTestCase {
    input: (PathBuf, PackageManagerKind),
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
      assert_eq!(project_root.unwrap_err(), case.expected.unwrap_err());
    }
  }

  test_each_serial!(
    test_new,
    test_new_each,
    "npm" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/npm"),
        PackageManagerKind::Npm,
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
        PackageManagerKind::Yarn,
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
        PackageManagerKind::Yarn,
      ),
      expected:Err(Error::InvalidPackageJsonPrivateForYarnError(PathBuf::from("tests/fixtures/workspaces/yarn_private_false/package.json")))
    },
    "pnpm" => NewTestCase {
      input: (
        PathBuf::from("tests/fixtures/workspaces/pnpm"),
        PackageManagerKind::Pnpm,
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
        PackageManagerKind::Bun,
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
