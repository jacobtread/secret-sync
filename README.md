# Secret-sync

![Tests Status](https://img.shields.io/github/actions/workflow/status/jacobtread/secret-sync/test.yml?style=for-the-badge&label=Tests)

**secret-sync** is a CLI tool for quickly and easily synchronizing local secrets file (`.env` and other configuration files) with remote secrets
manager such [AWS Secret Manager](https://docs.aws.amazon.com/secretsmanager/latest/userguide/intro.html) (or self-hosted alternatives like [Loker](https://github.com/jacobtread/loker))

**secret-sync** supports both pulling secrets out of secret managers and pushing
secrets into secret managers.

```sh
CLI tool for syncing local secret files with remote secret managers

Usage: secret-sync.exe [OPTIONS] <COMMAND>

Commands:
  pull  Pull the current secrets, storing the secret values in their respective files
  push  Push a secret file updating its value in the secret manage
  help  Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>  Optional custom path to the secret-sync.toml configuration file. By default secret-sync.toml (and secret-sync.json) is searched for in each parent directory until discovered
  -f, --format <FORMAT>  Output format to use when providing command output [default: human] [possible values: human, json]
  -h, --help             Print help (see more with '--help')
  -V, --version          Print version
```

## Installation

The recommended installation method when using **secret-sync** within a NPM based project is to use the `npm install` method

Manual binary downloads are available in [Releases](https://github.com/jacobtread/secret-sync/releases)

### Install prebuilt binaries into your npm project

```sh
npm install @jacobtread/secret-sync
```

### Install prebuilt binaries via shell script (Global)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jacobtread/secret-sync/releases/latest/download/secret-sync-installer.sh | sh
```

### Install prebuilt binaries via powershell script (Global)

```sh
powershell -ExecutionPolicy Bypass -c "irm https://github.com/jacobtread/secret-sync/releases/latest/download/secret-sync-installer.ps1 | iex"
```

## Configuration

**secret-sync** will search the current working directory for a `secret-sync.toml` (or `secret-sync.json`) file. If one is not found the parent
directories will be searched.

```toml
# Optional: Provider configuration (For future extension to other secrets managers)
[backend]
provider = "aws"

# Optional: AWS configuration
[aws]
# Optional: AWS profile override
profile = "example"
# OptionaL: AWS region override
region = "ap-southeast-2"
# Optional: AWS secrets endpoint override
endpoint = "https://secrets.example.com"

# Optional: Specify custom AWS access credentials
[aws.credentials]
access_key_id = "test"
access_key_secret = "secret"

[files.example]
# Path to the secret file relative to the secret-sync.toml or an absolute path
path = ".env"
# The secret manager secret to store/retrieve the data into/from
secret = "example"

# or the one line metadata = { description = "..etc" }
[files.example.metadata]
# Optional: Description that will be used for the secret on initial creation when pushing if not already existing
description = "Test description"
# Optional: AWS secret tags that will be attached on first push if the secret doesn't exist
tags = { "environment" = "production" }

# Specifying additional files
[files.example-2]
path = ".env.secondary"
secret = "example-2"
```

### Minimal Example

```toml
[files.example]
path = ".env"
secret = "example"
metadata = { description = "Example Secret" }
```
