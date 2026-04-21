use std::path::PathBuf;
use std::{fs, process::ExitCode};

use clap::{Parser, Subcommand};
use lox::{LoxError, tokenizer::Tokenizer};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Tokenize { filename: PathBuf },
}

fn run() -> Result<(), LoxError> {
    let cli = Cli::parse();

    let mut err = None;
    match cli.command {
        Commands::Tokenize { filename } => {
            let source = fs::read_to_string(&filename)?;
            for token in Tokenizer::new(&source) {
                match token {
                    Ok(token) => println!("{token}"),
                    Err(e) => {
                        eprintln!("{e}");
                        err = Some(e);
                    }
                }
            }
            println!("EOF  null");
        }
    };
    if let Some(err) = err {
        Err(err)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(65),
    }
}
