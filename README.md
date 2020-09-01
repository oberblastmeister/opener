# opener

opener is a simple to use alternative to to xdg-open and xdg-mime written in rust

## Features

- uses a readable toml configuration file (comments are allowed)
- specify commands to open for rules
- rules can include mime types
- easy to use: `"image/*" = "sxiv` instead of `"image/pdf" = "sxiv" "image/jpeg" = "sxiv` like mimeapps.list
- open files based on regex
- no need to remember mime types, opener will convert extensions to mime types automatically
- fast

## Installation

Install [rustup](https://www.rust-lang.org/tools/install)

Then run `cargo install opener`

## Usage

```
USAGE:
    opener [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Pass many times for less log output
    -V, --version    Prints version information
    -v, --verbose    Pass many times for more log output

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    open     Open or preview a file with the correct program
    query    Query for mime types or extensions
    set      Set the correct command for an extension, mime, or path
```

### Open

running `opener open <path>` will open the file based on the rules in the config file. By default opener will go to the next command if one rule groups fails. For example, if the regex rule failed opener will try to use the mime rule. If the mime rule failed then opener will just try to use whatever program is the default on your system (xdg-open for linux). The order that opener runs in can be set in the configuration file. The -p flag will preview the file instead of opening if, relying on the preview rules in the config file.

### Query

`opener query <ext_mime_path>` can take different arguments. If the argument is prefixed with a dot, opener will interpret that as an extension and will find the corresponding mime type. If the argument is a mime type, opener will print all the extensions that match the mime type. If a path is given, opener will try to find the mime type of the path. The path must exist. Giving opener a path is not just a wrapper for giving it and extension. If the extension is not found for the path opener will use tree_magic.

### Set

You can set rule in the configuration file on the command line. `set` accepts the same argument types as query. If a mime type is forgotton, you can give it an extension and it will convert that to a mime type when adding it to the configuration file.

## Configuration

## Advanced

## Inspiration
