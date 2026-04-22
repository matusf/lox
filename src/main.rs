use std::path::PathBuf;
use std::{fs, process};

use clap::{Parser, Subcommand};
use lox::{
    LoxError, parser,
    tokenizer::{Token, Tokenizer},
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Tokenize { filename: PathBuf },
    Parse { filename: PathBuf },
}

fn main() -> Result<(), LoxError> {
    let cli = Cli::parse();

    let mut is_err = false;
    match cli.command {
        Commands::Tokenize { filename } => {
            let source = fs::read_to_string(&filename)?;
            for token in Tokenizer::new(&source) {
                match token {
                    Ok(token) => println!("{token}"),
                    Err(e) => {
                        eprintln!("{e}");
                        is_err = true;
                    }
                }
            }
            println!("EOF  null");
        }
        Commands::Parse { filename } => {
            let source = fs::read_to_string(&filename)?;
            let tokens: Result<Vec<Token<'_>>, _> = Tokenizer::new(&source).collect();
            // TODO: this reports only first tokenization error
            let tokens = match tokens {
                Ok(ts) => ts,
                Err(_) => process::exit(65),
            };
            match parser::Parser::new(&source, tokens).parse() {
                Ok(expr) => println!("{expr}"),
                Err(e) => {
                    eprintln!("{:?}", e);
                    is_err = true;
                }
            }
        }
    };

    if is_err {
        process::exit(65);
    }
    Ok(())
}
