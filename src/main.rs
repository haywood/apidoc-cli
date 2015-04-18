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

#[derive(RustcDecodable)]
struct Args {
    arg_input: Option<String>,
    arg_tag: String,

    cmd_check: bool,
    cmd_generate: bool,
    cmd_push: bool,

    flag_config: String,
    flag_profile: String,
    flag_visibility: apidoc::models::Visibility,
}

impl Args {
    fn spec<'a>(&'a self) -> &'a str {
        match self.arg_input {
            Some(ref x) => x,
            None => "api.json"
        }
    }
}

fn main() {
    let home_dir = env::home_dir().expect("unable to get home directory");

    let usage = format!("
Usage:
    apidoc [options] check [<input>]
    apidoc [options] generate <tag>
    apidoc [options] push <tag> [<input>]
    apidoc --help

Options:
    --config <path-to-config>   [Default: {}/.apidoc/config]
    --visibility <visibility>   [Default: user]
    --profile <profile>         [Default: default]
    --help, -h  Print this help.
", home_dir.display());

    let args: Args = Docopt::new(usage)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    let ref config_path = args.flag_config;
    let ref profile_name = args.flag_profile;
    let result = Config::load(&config_path, &profile_name).and_then(|config| {
        let mut cli = Cli::new(config);
        if args.cmd_check {
            cli.check(args.spec())
        } else if args.cmd_generate {
            let ref tag = args.arg_tag;
            cli.generate(tag)
        } else if args.cmd_push {
            let ref tag = args.arg_tag;
            cli.push(tag, args.spec(), &args.flag_visibility)
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
