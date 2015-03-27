extern crate hyper;
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
    pub fn load(path: &str, profile_name: &str) -> Result<Config, CliError> {
        File::open(path).map_err(|err| {
            CliError { desc: format!("failed to open config at {}: {}",
                                     path, err.description().to_string()) }
        }).and_then(|mut file| {
            let mut buf = String::new();
            file.read_to_string(&mut buf).unwrap();
            let result: Result<toml::Value, Vec<toml::ParserError>> = buf.parse();
            result.map_err(|errs| {
                let mut desc = String::new();
                for err in errs {
                    desc.push_str(err.description());
                    desc.push('\n');
                }
                CliError { desc: desc }
            }).and_then(|value: toml::Value| {
                match value.lookup(profile_name) {
                    None => Err(CliError {
                        desc: format!("no profile found for {}", profile_name)
                    }),
                    Some(profile) => {
                        let mut decoder = toml::Decoder::new(profile.clone());
                        Config::decode(&mut decoder).map_err(|err| {
                            CliError { desc: err.description().to_string() }
                        })
                    }
                }
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
            CliError { desc: format!("failed to open input at {}: {}",
                                     path, err.description().to_string()) }
        }).and_then(|mut file| {
            let mut input = String::new();
            file.read_to_string(&mut input).unwrap();
            let validations = self.validations();
            validations.post(&input[..]).map_err(|err| {
                CliError { desc: err.description().to_string() }
            }).and_then(|mut res| {
                Json::from_reader(&mut res).map_err(|err| {
                    CliError { desc: err.description().to_string() }
                }).and_then(|json| {
                    let mut decoder = json::Decoder::new(json);
                    models::Validation::decode(&mut decoder).map_err(|err| {
                        CliError { desc: err.description().to_string() }
                    }).and_then(|validation| {
                        if validation.valid {
                            Ok(())
                        } else {
                            for err in validation.errors {
                                writeln!(&mut self.err, "validation error: {}", err).unwrap();
                            }
                            Err(CliError { desc: "input invalid".to_string() })
                        }
                    })
                })
            })
        })
    }

    pub fn generate(&mut self, tag: &str, target: &str) -> Result<(), CliError> {
        Cli::parse_tag(tag).and_then(|(org_key, app_key, version)| {
            self.code().get_by_organization_key_and_application_key_and_version_and_generator_key(
                org_key, app_key, version, target).map_err(|err| {
                    CliError { desc: err.description().to_string() }
                }).and_then(|mut res| {
                    Json::from_reader(&mut res).map_err(|err| {
                        CliError { desc: err.description().to_string() }
                    }).and_then(|json| {
                        let mut decoder = json::Decoder::new(json);
                        models::Code::decode(&mut decoder).map_err(|err| {
                            CliError { desc: err.description().to_string() }
                        }).and_then(|code| {
                            writeln!(&mut self.out, "{}", code.source).map_err(|err| {
                                CliError { desc: err.description().to_string() }
                            })
                        })
                    })
                })
        })
    }

    pub fn push(&self, tag: &str, path: &str) -> Result<(), CliError> {
        Cli::parse_tag(tag).and_then(|(org_key, app_key, version)| {
            File::open(path)
                .map_err(|err| { CliError { desc: err.description().to_string() } })
                .and_then(|mut file| {
                    let mut input = String::new();
                    file.read_to_string(&mut input).unwrap();
                    let form = models::VersionForm {
                        // TODO
                        visibility: None,
                        original_form: models::OriginalForm {
                            original_type: None,
                            data: input
                        }
                    };
                    self.versions().put_by_organization_key_and_application_key_and_version(
                        org_key, app_key, version, form
                    ).map_err(|err| {
                        CliError { desc: err.description().to_string() }
                    }).and_then(|mut res| {
                        match res.status {
                            hyper::Ok | hyper::status::StatusCode::Created => Ok(()),
                            status => {
                                let mut body = String::new();
                                res.read_to_string(&mut body);
                                let message = format!("HTTP request failed; status: {}, body: {}", status, body);
                                Err(CliError { desc: message })
                            }
                        }
                    })
                })
        })
    }

    fn parse_tag(tag: &str) -> Result<(&str, &str, &str), CliError> {
        match tag.find('/') {
            None => Err(CliError { desc: format!("failed to locate `/` in tag: {}", tag) }),
            Some(slash_idx) => {
                let rest = &tag[slash_idx..];
                match rest.find(':') {
                    None => Err(CliError { desc: format!("failed to locate `:` in {}", rest) }),
                    Some(mut colon_idx) => {
                        colon_idx += slash_idx;
                        let org_key = &tag[..slash_idx];
                        let app_key = &tag[(slash_idx + 1)..colon_idx];
                        let version = &tag[(colon_idx + 1)..];
                        if org_key.is_empty() {
                            Err(CliError { desc: format!("organization was empty in tag: {}", rest) })
                        } else if app_key.is_empty() {
                            Err(CliError { desc: format!("application was empty in tag: {}", rest) })
                        } else if version.is_empty() {
                            Err(CliError { desc: format!("version was empty in tag: {}", rest) })
                        } else {
                            Ok((org_key, app_key, version))
                        }
                    }
                }
            }
        }
    }

    fn code(&self) -> client::Code {
        let api_url = self.config.api_url.clone().unwrap_or(
            "http://api.apidoc.me".to_string());
        client::Code::new(api_url, self.config.token.clone())
    }

    fn validations(&self) -> client::Validations {
        let api_url = self.config.api_url.clone().unwrap_or(
            "http://api.apidoc.me".to_string());
        client::Validations::new(api_url, self.config.token.clone())
    }

    fn versions(&self) -> client::Versions {
        let api_url = self.config.api_url.clone().unwrap_or(
            "http://api.apidoc.me".to_string());
        client::Versions::new(api_url, self.config.token.clone())
    }
}
