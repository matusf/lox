# Lox

A tree-walking interpreter of Lox programming language from [Crafting Interpreters](https://craftinginterpreters.com) book.

## Testing

Tests use codedrafters test harness. Either install it from [`interpreter-tester`](https://github.com/matusf/interpreter-tester) or run `nix develop` to setup everything via Nix.

```sh
nix develop .
cargo test -- --test-threads=1
```
