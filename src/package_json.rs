use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, hash::Hash, io, path::Path};

use crate::utils::{hash::Hashable, path::to_absolute_path};

const PACKAGE_JSON: &str = "package.json";

type Dependencies = BTreeMap<String, String>;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Hash, Debug)]
pub struct PackageJson {
  dependencies: Option<Dependencies>,
  devDependencies: Option<Dependencies>,
  peerDependencies: Option<Dependencies>,
  overrides: Option<Dependencies>,
  optionalDependencies: Option<Dependencies>,
  workspaces: Option<Vec<String>>,
}

impl PackageJson {
  pub fn new<T: AsRef<Path>>(file_path: T) -> io::Result<PackageJson> {
    let contents = fs::read_to_string(&file_path).expect(
      format!(
        "No such file `{}`",
        to_absolute_path(&file_path).to_string_lossy()
      )
      .as_str(),
    );
    let json = serde_json::from_str::<PackageJson>(&contents)?;
    Ok(json)
  }

  pub fn resolve(&self) -> ProjectDependencies {
    ProjectDependencies {
      root: PackageDependencies {
        dependencies: self.dependencies.clone().unwrap_or_default(),
        dev_dependencies: self.devDependencies.clone().unwrap_or_default(),
        peer_dependencies: self.peerDependencies.clone().unwrap_or_default(),
        optional_dependencies: self.optionalDependencies.clone().unwrap_or_default(),
        overrides: self.overrides.clone().unwrap_or_default(),
      },
      workspaces: self.resolve_workspaces(),
    }
  }

  fn resolve_workspaces(&self) -> BTreeMap<String, ProjectDependencies> {
    let mut package_map = BTreeMap::<String, ProjectDependencies>::new();
    match &self.workspaces {
      Some(workspaces) => {
        for workspace in workspaces.iter() {
          let package_json_path = format!("{}/${PACKAGE_JSON}", workspace).to_string();
          if let Ok(p) = PackageJson::new(&package_json_path) {
            package_map.insert(package_json_path, p.resolve());
          }
        }
        package_map
      }
      None => package_map,
    }
  }
}

///
/// --------------------------------------------------
///
#[derive(Serialize, Deserialize, Hash, Debug)]
pub struct PackageDependencies {
  pub dependencies: Dependencies,
  pub dev_dependencies: Dependencies,
  pub peer_dependencies: Dependencies,
  pub overrides: Dependencies,
  pub optional_dependencies: Dependencies,
}

#[derive(Serialize, Deserialize, Hash, Debug)]
pub struct ProjectDependencies {
  pub root: PackageDependencies,
  pub workspaces: BTreeMap<String, ProjectDependencies>,
}

impl Hashable for ProjectDependencies {}

impl Default for ProjectDependencies {
  fn default() -> Self {
    ProjectDependencies {
      root: PackageDependencies {
        dependencies: BTreeMap::<String, String>::new(),
        dev_dependencies: BTreeMap::<String, String>::new(),
        optional_dependencies: BTreeMap::<String, String>::new(),
        overrides: BTreeMap::<String, String>::new(),
        peer_dependencies: BTreeMap::<String, String>::new(),
      },
      workspaces: BTreeMap::<String, ProjectDependencies>::new(),
    }
  }
}
