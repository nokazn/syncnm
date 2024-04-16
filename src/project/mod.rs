mod dependencies;
mod lib;
mod lockfile;
mod package_json;
mod package_manager;
mod workspaces;

pub use crate::project::lib::ProjectRoot;
pub use crate::project::lockfile::Lockfile;
pub use crate::project::package_manager::PackageManager;
