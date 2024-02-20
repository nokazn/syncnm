mod cache;
mod core;
mod errors;
mod lockfile;
mod package_manager;
mod project;
mod utils;
mod workspaces;

use crate::{core::run, lockfile::Lockfile, project::ProjectRoot, utils::hash::Hashable};

fn main() {
  env_logger::init();

  let lockfile = Lockfile::new("./examples/").unwrap();
  let result = lockfile.generate_hash();
  dbg!(&lockfile, &result.unwrap());

  // TODO: 後で消す
  let package_json = ProjectRoot::new("./examples", Some(lockfile.kind)).unwrap();
  let result = package_json.generate_hash();
  dbg!(&result.unwrap());

  let result = run("./examples", None::<&str>);
  dbg!(&result);
}
