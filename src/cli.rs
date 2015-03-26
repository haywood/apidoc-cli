extern crate toml;

use apidoc::client;
use apidoc::models;
use rustc_serialize::Decodable;
use rustc_serialize::json;
use rustc_serialize::json::Json;
use std::error;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::prelude::*;
use std::io::stderr;
use std::io::stdout;

#[derive(Debug, RustcDecodable)]
pub struct Config {
    api_url: Option<String>,
    token: String
}

#[derive(Debug)]
pub struct CliError {
    desc: String
}

impl Error for CliError {
    fn description(&self) -> &str { &self.desc[..] }
    fn cause(&self) -> Option<&error::Error> { None }
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&self.desc[..])
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Config, CliError> {
        File::open(path).map_err(|err| {
            CliError { desc: format!("failed to open config at {}", path) }
        }).and_then(|mut file| {
            let mut buf = String::new();
            file.read_to_string(&mut buf);
            let result: Result<toml::Value, Vec<toml::ParserError>> = buf.parse();
            result.map_err(|errs| {
                let mut desc = String::new();
                for err in errs {
                    desc.push_str(err.description());
                    desc.push('\n');
                }
                CliError { desc: desc }
            }).and_then(|value: toml::Value| {
                let mut decoder = toml::Decoder::new(value);
                Config::decode(&mut decoder).map_err(|err| {
                    CliError { desc: err.description().to_string() }
                })
            })
        })
    }
}

pub struct Cli {
    config: Config,
    out: Box<Write>,
    err: Box<Write>
}

impl Cli {
    pub fn new(config: Config) -> Cli {
        Cli {
            config: config,
            out: Box::new(stdout()),
            err: Box::new(stderr())
        }
    }

    pub fn check(&mut self, path: &str) -> Result<(), CliError> {
        File::open(path).map_err(|err| {
            CliError { desc: format!("failed to open input at {}", path) }
        }).map(|mut file| {
            let mut input = String::new();
            file.read_to_string(&mut input);
            let validations = self.validations();
            validations.post(&input[..]).map_err(|err| {
                CliError { desc: err.description().to_string() }
            }).and_then(|mut res| {
                Json::from_reader(&mut res).map_err(|err| {
                    CliError { desc: err.description().to_string() }
                }).and_then(|json| {
                    let mut decoder = json::Decoder::new(json);
                    models::Validation::decode(&mut decoder).map(|validation| {
                        for err in validation.errors {
                            writeln!(&mut self.err, "error: {}", err);
                        }
                    }).map_err(|err| {
                        CliError { desc: err.description().to_string() }
                    })
                })
            });
        })
    }

    fn validations(&self) -> client::Validations {
        let api_url = self.config.api_url.clone().unwrap_or(
            "http://api.apidoc.me".to_string());
        client::Validations::new(api_url, self.config.token.clone())
    }
}
