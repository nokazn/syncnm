mod cache;
mod cli;
mod core;
mod errors;
mod project;
mod utils;

fn main() {
  env_logger::init();

  cli::run();
}
