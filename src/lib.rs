use std::io;
use thiserror::Error;

pub mod tokenizer;

#[derive(Debug, Error)]
pub enum LoxError {
    #[error("Failed to read source file")]
    IoError(#[from] io::Error),
    #[error("Failed to tokenize")]
    TokenizerError(#[from] tokenizer::Error),
}
