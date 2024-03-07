use std::path::PathBuf;

use clap::{Arg, Command};

use crate::{
  cache::DEFAULT_CACHE_DIR,
  core::{self, APP_NAME},
};

const INSTALL_CMD: &str = "install";
const RUN_CMD: &str = "run";

const BASE_DIR_ARG: &str = "base_dir";
const CACHE_DIR_ARG: &str = "cache_dir";

pub fn run() {
  let cli = Command::new(APP_NAME)
    .about("Sync node_modules when your local dependency graph changes")
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(
      Command::new(INSTALL_CMD)
        .about("Install syncnm at your local project")
        .args(&[
          Arg::new(BASE_DIR_ARG).help("A path to a local project to install"),
          Arg::new(CACHE_DIR_ARG)
            .long("cache-dir")
            .short('c')
            .help("A path to a local project to install"),
        ]),
    )
    .subcommand(
      Command::new(RUN_CMD).about("run syncnm").args(&[
        Arg::new(BASE_DIR_ARG).help("A path to a local project to install"),
        Arg::new(CACHE_DIR_ARG)
          .long("cache-dir")
          .short('c')
          .help(format!(
            "A path to a local project to install (./{} by default) ",
            DEFAULT_CACHE_DIR
          )),
      ]),
    )
    .subcommand(Command::new("uninstall"));

  let matches = cli.get_matches();
  match matches.subcommand() {
    Some((INSTALL_CMD, _args)) => {
      // TODO: implement
      unimplemented!()
    }
    Some((RUN_CMD, args)) => {
      let base_dir = args
        .get_one::<String>(BASE_DIR_ARG)
        .map(PathBuf::from)
        .unwrap_or_default();
      let cache_dir = args.get_one::<String>(CACHE_DIR_ARG).map(PathBuf::from);
      let result = core::run(&base_dir, cache_dir);
      dbg!(&result);
    }
    _ => {
      // TODO: implement
      unimplemented!()
    }
  }
}
