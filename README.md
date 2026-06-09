# Lox

A tree-walking interpreter of Lox programming language from [Crafting Interpreters](https://craftinginterpreters.com) book.

## Building

To build the interpreter use `nix`:

```sh
nix build .
```

Or call `cargo` directly:

```sh
cargo build --release
```

## Usage

```console
$ lox --help
Lox interpreter

Usage: lox <FILENAME>

Arguments:
  <FILENAME>  Program read from script file

Options:
  -h, --help  Print help

$ lox test.lox
```

## Testing

Tests use codedrafters test harness. Either install it from [`interpreter-tester`](https://github.com/matusf/interpreter-tester) or run `nix develop` to setup everything via Nix.

```sh
nix develop .
cargo test -- --test-threads=1
```
