extern crate hyper;
extern crate toml;

use apidoc::client;
use apidoc::models;
use rustc_serialize::Decodable;
use rustc_serialize::Decoder;
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
use std::str::FromStr;

macro_rules! out {
    ($cli:expr, $fmt:expr, $($args:tt)*) => (
        stream!(
            $cli.out, $fmt,
            "failed writing to output stream: {}", $($args)*)
    );
}

macro_rules! err {
    ($cli:expr, $fmt:expr, $($args:tt)*) => (
        stream!(
            $cli.err, $fmt,
            "failed writing to error stream: {}", $($args)*)
    );
}

macro_rules! stream {
    ($stream:expr, $fmt:expr, $err_fmt:expr, $($args:tt)*) => (
        cli_try!(writeln!($stream, $fmt, $($args)*), $err_fmt)
    );
}

macro_rules! cli_opt {
    ($op:expr, $fmt:expr) => (
        match $op {
            Some(val) => val,
            None => return Err(CliError {
                desc: format!($fmt)
            }),
        }
    );
    ($op:expr, $fmt:expr, $($args:tt)*) => (
        match $op {
            Some(val) => val,
            None => return Err(CliError {
                desc: format!($fmt, $($args)*)
            }),
        }
    );
}

macro_rules! cli_try {
    ($op:expr) => (
        try!($op.map_err(|err| CliError::from_err(&err)))
    );
    ($op:expr, $fmt:expr) => (
        try!($op.map_err(|err| CliError {
            desc: format!($fmt, err)
        }))
    );
    ($op:expr, $fmt:expr, $($args:tt)*) => (
        try!($op.map_err(|err| CliError {
            desc: format!($fmt, $($args)*, err)
        }))
    );
}

#[derive(Debug, RustcDecodable)]
pub struct Config {
    api_url: Option<String>,
    token: String
}

#[derive(Debug)]
pub struct CliError {
    desc: String
}

impl CliError {
    fn from_err(err: &Error) -> CliError {
        CliError {
            desc: err.description().to_string()
        }
    }
}

pub struct Project<'a> {
    dependencies: Vec<Revision<'a>>
}

pub struct Repo<'a>(&'a str, &'a str);

impl<'a> Repo<'a> {
    fn from_str(tag: &'a str) -> Result<Repo<'a>, CliError> {
        let slash_idx = cli_opt!(
            tag.find('/'),
            "failed to locate `/` in tag: {}", tag);
        let org_key = &tag[..slash_idx];
        let app_key = &tag[slash_idx + 1..];
        if org_key.is_empty() {
            Err(CliError { desc: format!("organization was empty in tag: {}", tag) })
        } else if app_key.is_empty() {
            Err(CliError { desc: format!("application was empty in tag: {}", tag) })
        } else {
            Ok(Repo(org_key, app_key))
        }
    }
}

pub struct Revision<'a>(Repo<'a>, &'a str);

impl <'a> Revision<'a> {
    fn from_str(tag: &'a str) -> Result<Revision<'a>, CliError> {
        let Repo(org, repo_app) = cli_try!(Repo::from_str(tag));
        let colon_idx = cli_opt!(
            repo_app.find(':'), "failed to locate `:` in {}", tag);
        let app = &repo_app[..colon_idx];
        let version = &repo_app[colon_idx + 1..];
        if app.is_empty() {
            Err(CliError { desc: format!("application was empty in tag: {}", tag) })
        } else if version.is_empty() {
            Err(CliError { desc: format!("version was empty in tag: {}", tag) })
        } else {
            Ok(Revision(Repo(org, app), version))
        }
    }
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
        let mut file = cli_try!(
            File::open(path),
            "failed to open config at `{}`: {}",
            path);
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        let result: Result<toml::Value, Vec<toml::ParserError>> = buf.parse();
        let value: toml::Value = try!(result.map_err(|errs| {
            let mut desc = String::new();
            for err in errs {
                desc.push_str(err.description());
                desc.push('\n');
            }
            CliError { desc: desc }
        }));
        let profile = cli_opt!(
            value.lookup(profile_name),
            "no profile found for {}",
            profile_name);
        let mut decoder = toml::Decoder::new(profile.clone());
        Ok(cli_try!(Config::decode(&mut decoder)))
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
        let mut file = cli_try!(
            File::open(path),
            "failed to open input at `{}`: {}",
            path);
        let mut input = String::new();
        file.read_to_string(&mut input).unwrap();
        let validations = self.validations();
        let mut res = cli_try!(validations.post(&input[..]));
        let json = cli_try!(Json::from_reader(&mut res));
        let mut decoder = json::Decoder::new(json);
        let validation = cli_try!(models::Validation::decode(&mut decoder));
        if validation.valid {
            Ok(())
        } else {
            for err in validation.errors {
                err!(self, "validation error: {}", err);
            }
            Err(CliError { desc: "input invalid".to_string() })
        }
    }

    pub fn generate(&mut self, tag: &str, target: &str) -> Result<(), CliError> {
        let Revision(Repo(org, app), version) = cli_try!(
            Revision::from_str(tag));
        let mut res = cli_try!(self.code()
            .get_by_organization_key_and_application_key_and_version_and_generator_key(
                org, app, version, target));
        Ok(cli_try!(
            self.handle_response(
                &mut res,
                |mut decoder| models::Code::decode(decoder),
                |mut cli, code| Ok(out!(cli, "{}", code.source)))))
    }

    pub fn push(
        &mut self,
        tag: &str,
        path: &str,
        visibility: &models::Visibility
    ) -> Result<(), CliError> {
        cli_try!(visibility.valid(), "invalid visiblity: {}");
        let Revision(Repo(org, app), version) = try!(Revision::from_str(tag));
        let mut file = cli_try!(
            File::open(path), "failed to open {}: {}", path);
        let mut input = String::new();
        cli_try!(file.read_to_string(&mut input));
        let form = models::VersionForm {
            visibility: Some(visibility.clone()),
            original_form: models::OriginalForm {
                original_type: None,
                data: input
            }
        };
        out!(self, "pushing to {}/{}:{}", org, app, version);
        let mut res = cli_try!(self.versions()
            .put_by_organization_key_and_application_key_and_version(
                org, app, version, form));
        self.handle_response(
            &mut res,
            |mut decoder| { models::Version::decode(decoder) },
            |_, _| { Ok(()) })
    }

    fn handle_response<
        T,
        D: Fn(&mut json::Decoder) -> json::DecodeResult<T>,
        CB: Fn(&mut Cli, &mut T) -> Result<(), CliError>
    >(
        &mut self,
        mut res: &mut hyper::client::Response,
        decode: D,
        cb: CB
    ) -> Result<(), CliError> {
        let mut body = String::new();
        cli_try!(res.read_to_string(&mut body));
        let status = res.status;
        let json: Json = cli_try!(
            body.parse(),
            "HTTP request failed; status: {}\nbody: {}\ndecode_error: {}",
            status, body);
        let mut decoder = json::Decoder::new(json.clone());
        match decode(&mut decoder) {
            Ok(mut t) => cb(self, &mut t),
            Err(original_err) => {
                let mut decoder = json::Decoder::new(json);
                let errors = cli_try!(
                    Vec::<models::Error>::decode(&mut decoder),
                    "HTTP request failed; status: {}\nbody: {}\noriginal_error: {}\ndecode_error: {}\n",
                    status, body, original_err);
                for error in errors {
                    err!(self, "error: {}", error.message);
                }
                Err(CliError { desc: format!("HTTP request failed; status: {}", status) })
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
        client::Validations::new(api_url)
    }

    fn versions(&self) -> client::Versions {
        let api_url = self.config.api_url.clone().unwrap_or(
            "http://api.apidoc.me".to_string());
        client::Versions::new(api_url, self.config.token.clone())
    }
}
