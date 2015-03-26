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
    apidoc [options] push <tag> <input>

Options:
    --config <path-to-config>  [Default: {}/.apidoc/config]
    --help  Print this help.
", home_dir.display());

    let args = Docopt::new(usage)
        .and_then(|d| d.parse())
        .unwrap_or_else(|e| e.exit());
    let config_path = args.get_str("--config");
    let config = Config::load(config_path).unwrap();
    let mut cli = Cli::new(config);
    let result = if args.get_bool("check") {
        cli.check(args.get_str("<input>"))
    } else if args.get_bool("push") {
        let tag = args.get_str("<tag>");
        let input = args.get_str("<input>");
        cli.push(tag, input)
    } else {
        panic!("unkown command")
    };
    match result {
        Ok(_) => (),
        Err(err) => writeln!(&mut stderr(), "{}", err.description()).unwrap()
    }
}
