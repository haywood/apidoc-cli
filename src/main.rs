#![feature(io)]
#![feature(exit_status)]
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
    apidoc [options] push <tag> <input>
    apidoc --help

Options:
    --config <path-to-config>  [Default: {}/.apidoc/config]
    --profile <profile>  [Default: default]
    --help, -h  Print this help.
", home_dir.display());

    let args = Docopt::new(usage)
        .and_then(|d| d.parse())
        .unwrap_or_else(|e| e.exit());
    let config_path = args.get_str("--config");
    let profile_name = args.get_str("--profile");
    let result = Config::load(config_path, profile_name).and_then(|config| {
        let mut cli = Cli::new(config);
        if args.get_bool("check") {
            cli.check(args.get_str("<input>"))
        } else if args.get_bool("push") {
            let tag = args.get_str("<tag>");
            let input = args.get_str("<input>");
            cli.push(tag, input)
        } else {
            panic!("unkown command")
        }
    });
    match result {
        Ok(_) => (),
        Err(err) => {
            writeln!(&mut stderr(), "{}", err.description()).unwrap();
            env::set_exit_status(1)
        }
    }
}
