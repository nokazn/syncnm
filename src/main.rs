mod cache;
mod core;
mod errors;
mod lockfile;
mod package_json;
mod utils;
mod workspaces;

use crate::{lockfile::Lockfile, package_json::ProjectRoot, utils::hash::Hashable};
use env_logger;

fn main() {
  env_logger::init();

  let lockfile = Lockfile::new("./examples/").unwrap();
  let result = lockfile.generate_hash();
  dbg!(&lockfile, &result.unwrap());

  // TODO: 後で消す
  let package_json = ProjectRoot::new("./examples", lockfile.kind).unwrap();
  let result = package_json.generate_hash();
  dbg!(&result.unwrap());
}
