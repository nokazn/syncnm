mod cache;
mod cli;
mod core;
mod errors;
mod lockfile;
mod package_manager;
mod project;
mod utils;
mod workspaces;

fn main() {
  env_logger::init();

  cli::run();
}
