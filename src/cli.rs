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
use std::result::Result as StdResult;
use std::str::FromStr;
use self::hyper::client::Response;
use self::hyper::status::StatusCode;


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

pub type CliResult<T> = StdResult<T, CliError>;

impl CliError {
    fn from_err(err: &Error) -> CliError {
        CliError {
            desc: err.description().to_string()
        }
    }
}

// TODO implement support for parsing
// some kind of project manifest file,
// resolving dependencies, and generating
// code for them as described in said file.
//
// Thinking that this would be the default
// mode of the generate command (without args).
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
        let task = Check { path: path };
        task.run(self)
    }

    pub fn generate(&mut self, tag: &str, target: &str) -> Result<(), CliError> {
        let task = Generate { tag: tag, target: target };
        task.run(self)
    }

    pub fn push(
        &mut self,
        tag: &str,
        path: &str,
        visibility: &models::Visibility
    ) -> Result<(), CliError> {
        let task = Push { tag: tag, path: path, visibility: visibility };
        task.run(self)
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

trait Task {
    type Result;

    fn perform_request(&self, cli: &mut Cli) -> CliResult<Response>;

    fn parse_json(&self, status: StatusCode, json: Json) -> CliResult<Self::Result>;

    fn handle_result(&self, cli: &mut Cli, result: Self::Result) -> CliResult<()>;

    fn run(&self, cli: &mut Cli) -> CliResult<()> {
        let mut res = cli_try!(self.perform_request(cli), "HTTP request failed: {}");
        let status = res.status;
        let json = cli_try!(
            Json::from_reader(&mut res),
            "failed to parse HTTP response body as JSON (status was {}): {}",
            status);
        self.handle_result(cli, cli_try!(self.parse_json(status, json)))
    }
}

struct Check<'a> {
    path: &'a str
}

impl<'a> Task for Check<'a> {
    type Result = StdResult<models::Validation, models::Validation>;

    fn perform_request(&self, cli: &mut Cli) -> CliResult<Response> {
        let mut file = cli_try!(
            File::open(self.path),
            "failed to open input at `{}`: {}",
            self.path);
        let mut input = String::new();
        cli_try!(
            file.read_to_string(&mut input),
            "failed reading from file at `{}`: {}",
            self.path);
        let validations = cli.validations();
        Ok(cli_try!(validations.post(&input[..])))
    }

    fn parse_json(&self, status: hyper::status::StatusCode, json: Json) -> CliResult<<Check as Task>::Result> {
        let mut decoder = json::Decoder::new(json);
        let result = models::Validation::decode(&mut decoder);
        Ok(cli_try!(match status {
            hyper::Ok => result.map(|v| Ok(v)),
            _ => result.map(|v| Err(v))
        }))
    }

    fn handle_result(&self, cli: &mut Cli, result: <Check as Task>::Result) -> CliResult<()> {
        match result {
            Ok(_) => Ok(()),
            Err(validation) => {
                for err in validation.errors {
                    err!(cli, "validation error: {}", err);
                }
                Err(CliError { desc: "input invalid".to_string() })
            }
        }
    }
}

struct Generate<'a, 'b> {
    tag: &'a str,
    target: &'b str
}

impl<'a, 'b> Task for Generate<'a, 'b> {
    type Result = StdResult<models::Code, Vec<models::Error>>;

    fn perform_request(&self, cli: &mut Cli) -> CliResult<Response> {
        let Revision(Repo(org, app), version) = cli_try!(
            Revision::from_str(self.tag));
        let client = cli.code();
        Ok(cli_try!(
            client.get_by_organization_key_and_application_key_and_version_and_generator_key(
                org, app, version, self.target)))
    }

    fn parse_json(&self, status: StatusCode, json: Json) -> CliResult<<Generate as Task>::Result> {
        let mut decoder = json::Decoder::new(json);
        Ok(cli_try!(match status {
            hyper::Ok => {
                let result = models::Code::decode(&mut decoder);
                result.map(|c| Ok(c))
            },
            _ => {
                let result = Vec::<models::Error>::decode(&mut decoder);
                result.map(|e| Err(e))
            }
        }))
    }

    fn handle_result(&self, cli: &mut Cli, result: <Generate as Task>::Result) -> CliResult<()> {
        match result {
            Ok(code) => Ok(out!(cli, "{}", code.source)),
            Err(errors) => {
                for error in errors {
                    err!(cli, "error: {}", error.message);
                }
                Err(CliError { desc: "got error response from server".to_string() })
            }
        }
    }
}

struct Push<'a> {
    tag: &'a str,
    path: &'a str,
    visibility: &'a models::Visibility
}

impl<'a> Task for Push<'a> {
    type Result = StdResult<models::Version, Vec<models::Error>>;

    fn perform_request(&self, cli: &mut Cli) -> CliResult<Response> {
        cli_try!(self.visibility.valid(), "invalid visiblity: {}");
        let Revision(Repo(org, app), version) = try!(Revision::from_str(self.tag));
        let mut file = cli_try!(
            File::open(self.path), "failed to open {}: {}", self.path);
        let mut input = String::new();
        cli_try!(file.read_to_string(&mut input));
        let form = models::VersionForm {
            visibility: Some(self.visibility.clone()),
            original_form: models::OriginalForm {
                original_type: None,
                data: input
            }
        };
        out!(cli, "pushing to {}/{}:{}", org, app, version);
        Ok(cli_try!(cli.versions()
            .put_by_organization_key_and_application_key_and_version(
                org, app, version, form)))
    }

    fn parse_json(&self, status: StatusCode, json: Json) -> CliResult<<Push as Task>::Result> {
        let mut decoder = json::Decoder::new(json);
        Ok(cli_try!(match status {
            hyper::Ok => models::Version::decode(&mut decoder).map(|v| Ok(v)),
            _ => Vec::<models::Error>::decode(&mut decoder).map(|e| Err(e))
        }))
    }

    fn handle_result(&self, cli: &mut Cli, result: <Push as Task>::Result) -> CliResult<()> {
        match result {
            Ok(_) => Ok(()),
            Err(errors) => {
                for error in errors {
                    err!(cli, "error: {}", error.message);
                }
                Err(CliError { desc: "got error response from server".to_string() })
            }
        }
    }
}
