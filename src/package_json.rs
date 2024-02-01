use serde::{Deserialize, Serialize};
use std::{
  collections::BTreeMap,
  fs,
  hash::Hash,
  path::{Path, PathBuf},
};

use crate::{
  core::PackageManagerKind, error::InvalidPackageJsonForWorkspacesError, utils::hash::Hashable,
};
use crate::{
  error::{Error, IoError, NoEntryError, ParseJsonError},
  workspaces::Workspaces,
};

const PACKAGE_JSON: &str = "package.json";

type Dependencies = BTreeMap<String, String>;

fn to_package_json_path<T: AsRef<Path>>(base_dir: T) -> PathBuf {
  base_dir.as_ref().join(PACKAGE_JSON)
}

///
/// --------------------------------------------------
///
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Hash, Debug, Clone)]
struct PackageJson {
  name: Option<String>,
  version: Option<String>,
  dependencies: Option<Dependencies>,
  devDependencies: Option<Dependencies>,
  peerDependencies: Option<Dependencies>,
  overrides: Option<Dependencies>,
  optionalDependencies: Option<Dependencies>,
  workspaces: Option<Vec<String>>,
}

impl PackageJson {
  fn new<T: AsRef<Path>>(base_dir: T) -> Result<PackageJson, Error> {
    let file_path = to_package_json_path(base_dir);
    let contents = fs::read_to_string(&file_path)
      .map_err(|_| Error::NoEntryError(NoEntryError::new(&file_path)))?;
    serde_json::from_str::<PackageJson>(&contents)
      .map_err(|_| Error::ParseJsonError(ParseJsonError::new(&file_path)))
  }
}

///
/// --------------------------------------------------
///
#[derive(Serialize, Deserialize, Hash, Clone, Debug)]
struct PackageDependencies {
  dependencies: Dependencies,
  dev_dependencies: Dependencies,
  peer_dependencies: Dependencies,
  overrides: Dependencies,
  optional_dependencies: Dependencies,
}

impl PackageDependencies {
  fn new(raw: PackageJson) -> Self {
    PackageDependencies {
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
#[derive(Serialize, Deserialize, Hash, Clone, Debug)]
struct WorkspacePackage {
  original: PackageJson,
  base_dir: PathBuf,
  kind: PackageManagerKind,
  dependencies: PackageDependencies,
}

impl WorkspacePackage {
  fn new<T: AsRef<Path>>(base_dir: T, kind: PackageManagerKind) -> Result<Self, Error> {
    let original = PackageJson::new(&base_dir)?;
    Ok(WorkspacePackage {
      original: original.clone(),
      base_dir: base_dir.as_ref().to_path_buf(),
      kind,
      dependencies: PackageDependencies::new(original),
    })
  }

  fn validate_package_json_fields<T: AsRef<Path>>(self, base_dir: T) -> Result<Self, Error> {
    let package_json_path = to_package_json_path(&base_dir);
    match self.kind {
      PackageManagerKind::YarnLock
        if self.original.name.is_none() || self.original.version.is_none() =>
      {
        // TODO: better error message
        log::warn!(
          "name and version are required in `{}`",
          &package_json_path.to_string_lossy()
        );
        Err(Error::InvalidPackageJsonForWorkspacesError(
          InvalidPackageJsonForWorkspacesError::new(&package_json_path),
        ))
      }
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
#[derive(Serialize, Deserialize, Hash, Clone, Debug)]
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
  pub fn new<T: AsRef<Path>>(base_dir: T, kind: PackageManagerKind) -> Result<ProjectRoot, Error> {
    let original = PackageJson::new(&base_dir)?;
    Ok(ProjectRoot {
      original: original.clone(),
      kind,
      root: PackageDependencies::new(original.clone()),
      workspaces: ProjectRoot::resolve_workspaces(base_dir, kind, original.workspaces),
    })
  }

  fn resolve_workspaces<T: AsRef<Path>>(
    base_dir: T,
    kind: PackageManagerKind,
    patterns: Option<Vec<String>>,
  ) -> BTreeMap<String, WorkspacePackage> {
    let workspaces = Workspaces::new(base_dir.as_ref().to_path_buf(), kind, patterns);
    let mut workspace_map = BTreeMap::<String, WorkspacePackage>::new();
    for p in workspaces.packages.iter() {
      if let Ok(w) = WorkspacePackage::new(&p, kind).and_then(|w| w.validate_package_json_fields(p))
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
}
