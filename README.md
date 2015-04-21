# Apidoc CLI [![Build Status](https://travis-ci.org/haywood/apidoc-cli.svg)](https://travis-ci.org/haywood/apidoc-cli)

This is a CLI for working with [apidoc](api.apidoc.me).

## Installation

### OS X

    wget https://github.com/haywood/apidoc-cli/releases/download/<version>/apidoc-x86_64-apple-darwin
    chmod +x apidoc-x86_64-apple-darwin
    sudo mv apidoc-x86_64-apple-darwin /usr/local/bin/apidoc

e.g.

    wget https://github.com/haywood/apidoc-cli/releases/download/0.0.3/apidoc-x86_64-apple-darwin
    chmod +x apidoc-x86_64-apple-darwin
    sudo mv apidoc-x86_64-apple-darwin /usr/local/bin/apidoc

### Linux

    wget https://github.com/haywood/apidoc-cli/releases/download/<version>/apidoc-x86_64-unknown-linux-gnu
    chmod +x apidoc-x86_64-unknown-linux-gnu
    sudo mv apidoc-x86_64-unknown-linux-gnu /usr/local/bin/apidoc-x86_64-unknown-linux-gnu

e.g.

    wget https://github.com/haywood/apidoc-cli/releases/download/0.0.3/apidoc-x86_64-unknown-linux-gnu
    chmod +x apidoc-x86_64-unknown-linux-gnu
    sudo mv apidoc-x86_64-unknown-linux-gnu /usr/local/bin/apidoc-x86_64-unknown-linux-gnu

## Usage

- `apidoc check` - validate an api.json file using the API.
- `apidoc generate` - generate code from a given version of an application for a given target.
- `apidoc push` - push a new version of an application to api.apidoc.me.

For more detailed usage information, just run `apidoc --help`.

## Configuration

The API uses a simple [TOML](https://github.com/toml-lang/toml)
configuration located at `~/.apidoc/config` by default.
Configuration is done using profiles similar to those used by
[Amazon Web Services](http://docs.aws.amazon.com/cli/latest/userguide/cli-chap-getting-started.html#cli-multiple-profiles).
A minimal default config looks like this

    [default]
    token = "394530a861f89e4fed8536f6c90e74189cd2eed40bf3f234c08ef105586ca8b3"

To generate a token, go to [http://www.apidoc.me/tokens/create](http://www.apidoc.me/tokens/create).
It is recommended to enter something like *CLI* or *Home PC* in the description field.
