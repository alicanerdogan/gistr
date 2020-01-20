# gistr

[![License](https://img.shields.io/github/license/alicanerdogan/subtle.svg)](https://github.com/alicanerdogan/gistr/blob/master/LICENSE)
[![Contributions Welcome](https://img.shields.io/badge/contributions-welcome-brightgreen.svg?style=flat)](https://github.com/alicanerdogan/gistr/issues)

gistr is a CLI tool written in rust to download, display and create gists from terminal.

![CLI Animation](./docs/cli.gif)

## Installation

Binaries for various platforms can be downloaded from releases section. gistr is available for linux, windows and mac.

[![Releases](./docs/download.svg)](https://github.com/alicanerdogan/gistr/releases) [Releases](https://github.com/alicanerdogan/gistr/releases)

## Quick Start

To be able to access your github account, you need to login using the login subcommand or creating a token file named `.gistr` at the home directory of your machine with a valid token in it. You can manually get the token from https://github.com/settings/tokens/new with only `gist` scope is selected.

Basic functionality of the cli tool is given below:

```sh
USAGE:
    gistr [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    create      create gist(s)
    display     display gists for the user
    download    download gists for the user
    help        Prints this message or the help of the given subcommand(s)
    login       gets necessary token from github to read/write as a user and stores it in the filesystem
                it requires credentials to complete operation
```

## Subcommands

### login

```sh
gets necessary token from github to read/write as a user and stores it in the filesystem
it requires credentials to complete operation

USAGE:
    gistr login

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
```

### display

```sh
display gists for the user

USAGE:
    gistr display [FLAGS]

FLAGS:
    -h, --help       Prints help information
    -p, --public     display only public gists
    -V, --version    Prints version information
```

### download

```sh
download gists for the user

USAGE:
    gistr download [FLAGS]

FLAGS:
    -h, --help       Prints help information
    -p, --public     download only public gists
    -V, --version    Prints version information
```

### create

```sh
create gist(s)

USAGE:
    gistr create [FLAGS] <FILE(S)>... --description <DESCRIPTION>

FLAGS:
    -h, --help       Prints help information
    -p, --public     makes gist public
    -V, --version    Prints version information

OPTIONS:
    -d, --description <DESCRIPTION>    gist description

ARGS:
    <FILE(S)>...    gist file
```

### help

## License

[MIT](LICENSE).
