#![feature(io)]
extern crate rustc_serialize;
extern crate docopt;

use cli::*;
use docopt::Docopt;
use std::env;
use std::error::Error;
use std::io::stderr;
use std::io::Write;

mod apidoc;
mod cli;

fn main() {
    let home_dir = env::home_dir().expect("unable to get home directory");

    let usage = format!("
Usage:
    apidoc [options] check <input>

Options:
    --config, -c <path-to-config>  [Default: {}/.apidoc/config]
    --help, -h  Print this help.
", home_dir.display());

    let args = Docopt::new(usage)
        .and_then(|d| d.parse())
        .unwrap_or_else(|e| e.exit());
    let config_path = args.get_str("--config");
    let config = Config::load(config_path).unwrap();
    let mut cli = Cli::new(config);
    let result = if args.get_bool("check") {
        cli.check(args.get_str("<input>"))
    } else {
        panic!("unkown command")
    };
    result.map_err(|err| {
        writeln!(&mut stderr(), "{}", err.description()).unwrap();
    });
}
