use std::collections::HashMap;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::rc::Rc;
use std::{fs, process};

use clap::{Parser, Subcommand};
use lox::{
    LoxError,
    interpreter::{Environment, Interpreter},
    parser,
    resolver::Resolver,
    tokenizer::{Token, Tokenizer},
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Tokenize {
        filename: PathBuf,
    },
    Parse {
        filename: PathBuf,
    },
    Evaluate {
        filename: PathBuf,
    },
    Run {
        filename: PathBuf,
        #[arg(long)]
        print_ast: bool,
    },
}

fn main() -> Result<(), LoxError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tokenize { filename } => {
            let source = fs::read_to_string(&filename)?;
            let (tokens, errors): (Vec<_>, Vec<_>) =
                Tokenizer::new(&source).partition(Result::is_ok);

            for token in tokens.into_iter().map(Result::unwrap) {
                println!("{token}");
            }

            let mut is_err = false;
            if !errors.is_empty() {
                is_err = true;
            }

            for error in errors.into_iter().map(Result::unwrap_err) {
                eprintln!("{error}");
            }

            println!("EOF  null");
            if is_err {
                process::exit(65);
            }
        }
        Commands::Parse { filename } => {
            let source = fs::read_to_string(&filename)?;
            let tokens: Result<Vec<_>, _> = Tokenizer::new(&source).collect();

            if tokens.is_err() {
                process::exit(65);
            }

            match parser::Parser::new(&source, tokens?).parse_expression() {
                Ok(expr) => println!("{expr}"),
                Err(error) => {
                    eprintln!("{error:?}");
                    process::exit(65);
                }
            }
        }
        Commands::Evaluate { filename } => {
            let source = fs::read_to_string(filename)?;
            let tokens: Result<Vec<Token<'_>>, _> = Tokenizer::new(&source).collect();
            let expr = parser::Parser::new(&source, tokens?).parse_expression()?;
            let interpreter = Interpreter::new(HashMap::new());
            match interpreter.eval(&expr, Rc::new(Environment::default())) {
                Ok(value) => println!("{value}"),
                Err(err) => {
                    eprintln!("{err}");
                    process::exit(70);
                }
            }
        }
        Commands::Run {
            filename,
            print_ast,
        } => {
            let source = fs::read_to_string(filename)?;
            let tokens: Result<Vec<Token<'_>>, _> = Tokenizer::new(&source).collect();
            let (statements, errors): (Vec<_>, Vec<_>) =
                parser::Parser::new(&source, tokens?).partition(Result::is_ok);

            let mut is_err = false;
            for error in errors.into_iter().map(Result::unwrap_err) {
                eprintln!("Parse error: {error}");
                is_err = true;
            }

            if is_err {
                process::exit(65);
            }

            let program: Vec<_> = statements.into_iter().map(Result::unwrap).collect();
            if print_ast {
                for statement in &program {
                    println!("{statement}");
                }
            }

            let resolver = Resolver::default();
            let locals = match resolver.run(program.iter()) {
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(65);
                }
                Ok(locals) => locals,
            };

            eprintln!("{:?}", &locals);
            let interpreter = Interpreter::new(locals);
            let globals = Environment::with_globals();
            match interpreter.execute(program.iter(), Rc::new(globals)) {
                Ok(ControlFlow::Break(_)) => unreachable!(),
                Ok(_) => (),
                Err(error) => {
                    eprintln!("{error}");
                    process::exit(70);
                }
            }
        }
    }
    Ok(())
}
