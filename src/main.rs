mod lockfile;
mod package_json;
mod utils;

use crate::utils::hash::Hashable;
use lockfile::Lockfile;
use package_json::PackageJson;

fn main() {
  // TODO: 後で消す
  let package_json = PackageJson::new("./examples/package.json").unwrap();
  let result = package_json.resolve().generate_hash();
  dbg!(&result.unwrap());
  let lockfile = Lockfile::new("./examples/").unwrap();
  let result = lockfile.generate_hash();
  dbg!(lockfile, &result.unwrap());
}
