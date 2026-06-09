use std::fmt::{Debug, Display};
use std::fs;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;

use clap::Parser;
use lox::{
    LoxError,
    interpreter::{Interpreter, environment::Environment},
    parser,
    resolver::Resolver,
    tokenizer::Tokenizer,
};

/// Lox interpreter
#[derive(Parser)]
struct Cli {
    /// Program read from script file
    filename: PathBuf,
}

fn report<T: Debug, E: Display + Debug>(
    items: Vec<Result<T, E>>,
    errors: Vec<Result<T, E>>,
) -> Vec<T> {
    if !errors.is_empty() {
        for error in errors.into_iter().map(Result::unwrap_err) {
            eprintln!("{error}");
        }
        process::exit(1);
    }
    items.into_iter().map(Result::unwrap).collect()
}

fn main() -> Result<(), LoxError> {
    let cli = Cli::parse();

    let source = fs::read_to_string(cli.filename)?;

    let (tokens, errors) = Tokenizer::new(&source).partition(Result::is_ok);
    let tokens = report(tokens, errors);

    let (statements, errors) = parser::Parser::new(&source, tokens).partition(Result::is_ok);
    let statements = report(statements, errors);

    let _ = Interpreter::new(Resolver::default().run(statements.iter())?)
        .execute(statements.iter(), &Rc::new(Environment::with_globals()))?;

    Ok(())
}
