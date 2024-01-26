mod package_json;
mod utils;

use package_json::PackageJson;

fn main() {
  // TODO: 後で消す
  let result = PackageJson::new("./examples/package.json").map(|p| p.resolve().generate_hash());
  dbg!(result.unwrap());
}
