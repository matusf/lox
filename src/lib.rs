use std::io;
use thiserror::Error;

pub mod interpreter;
pub mod parser;
pub mod resolver;
pub mod tokenizer;

#[derive(Debug, Error)]
pub enum LoxError {
    #[error("Failed to read source file")]
    IoError(#[from] io::Error),
    #[error("Failed to tokenize")]
    TokenizerError(#[from] tokenizer::Error),
    #[error("Failed to parse")]
    ParserError(#[from] parser::Error),
    #[error("Runtime error")]
    RuntimeError(#[from] interpreter::Error),
    #[error("Resolver error")]
    ResolverError(#[from] resolver::Error),
}
