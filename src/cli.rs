use std::path::PathBuf;

use clap::{value_parser, Arg, Command};

use crate::core::{self, APP_NAME};

const INSTALL_CMD: &str = "install";
const RUN_CMD: &str = "run";
const UNINSTALL_CMD: &str = "uninstall";

const BASE_DIR_ARG: &str = "base_dir";
const CACHE_DIR_ARG: &str = "cache_dir";

fn path_buf_arg(id: &'static str) -> Arg {
  Arg::new(id).value_parser(value_parser!(PathBuf))
}

pub fn run() {
  let base_dir_arg = path_buf_arg(BASE_DIR_ARG).help(format!(
    "A path to a local project to install {APP_NAME} (the current directory by default)"
  ));
  let cache_dir_arg = path_buf_arg(CACHE_DIR_ARG)
    .long("cache-dir")
    .short('c')
    .help(
      #[cfg(target_os = "linux")]
      format!("A path to a cache store directory ($XDG_CACHE_HOME/{APP_NAME} or ~/.cache/{APP_NAME} by default) ",),
      #[cfg(target_os = "macos")]
      format!("A path to a cache store directory (~/Library/Caches/{APP_NAME} by default) ",),
      #[cfg(target_os = "windows")]
      format!("A path to a cache store directory (%LOCALAPPDATA%/{APP_NAME} or ~\\AppData\\Local\\{APP_NAME} by default) ",),
    );

  let cli = Command::new(APP_NAME)
    .about("Sync node_modules when your local dependency list changes")
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(
      Command::new(INSTALL_CMD)
        .about(format!("Install {APP_NAME} at your local project"))
        .arg(base_dir_arg.clone())
        .arg(cache_dir_arg.clone()),
    )
    .subcommand(
      Command::new(RUN_CMD)
        .about(format!("Run {APP_NAME}"))
        .arg(base_dir_arg.clone())
        .arg(cache_dir_arg.clone()),
    )
    .subcommand(
      Command::new(UNINSTALL_CMD)
        .about(format!("Uninstall {APP_NAME} from your local project"))
        .arg(base_dir_arg.clone()),
    );

  let matches = cli.get_matches();
  match matches.subcommand() {
    Some((INSTALL_CMD, _args)) => {
      // TODO: implement
      unimplemented!()
    }
    Some((RUN_CMD, args)) => {
      let base_dir = args
        .get_one::<PathBuf>(BASE_DIR_ARG)
        .map(PathBuf::from)
        .unwrap_or_default();
      let cache_dir = args.get_one::<PathBuf>(CACHE_DIR_ARG).map(PathBuf::from);
      let result = core::run(base_dir, cache_dir);
      dbg!(&result);
    }
    _ => {
      // TODO: implement
      unimplemented!()
    }
  }
}
