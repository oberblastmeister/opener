# opener

opener is a simple to use alternative to to xdg-open and xdg-mime written in rust

## Features

- uses a readable toml configuration file (comments are allowed)
- specify commands to open for patterns
- patterns can include mime types
- easy to use: `"image/*" = "sxiv` instead of `"image/pdf" = "sxiv" "image/jpeg" = "sxiv` like mimeapps.list
- open files based on regex
- no need to remember mime types, opener will convert extensions to mime types automatically
- fast

## Installation

Install ![rustup](https://www.rust-lang.org/tools/install)

Then run `cargo install opener`

## Usage
