use std::{fmt::Display, iter::Peekable, str::Chars};

use thiserror::Error;

pub struct Tokenizer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    current: usize,
    line: usize,
}

#[derive(Debug)]
pub struct Token<'a> {
    lexeme: &'a str,
    typ: TokenType,
}

#[derive(Debug)]
enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One Or Two Character Tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lexeme = self.lexeme;
        match self.typ {
            TokenType::LeftParen => write!(f, "LEFT_PAREN {lexeme} null"),
            TokenType::RightParen => write!(f, "RIGHT_PAREN {lexeme} null"),
            TokenType::LeftBrace => write!(f, "LEFT_BRACE {lexeme} null"),
            TokenType::RightBrace => write!(f, "RIGHT_BRACE {lexeme} null"),
            TokenType::Comma => write!(f, "COMMA {lexeme} null"),
            TokenType::Dot => write!(f, "DOT {lexeme} null"),
            TokenType::Minus => write!(f, "MINUS {lexeme} null"),
            TokenType::Plus => write!(f, "PLUS {lexeme} null"),
            TokenType::Semicolon => write!(f, "SEMICOLON {lexeme} null"),
            TokenType::Slash => write!(f, "SLASH {lexeme} null"),
            TokenType::Star => write!(f, "STAR {lexeme} null"),
            TokenType::Bang => write!(f, "BANG {lexeme} null"),
            TokenType::BangEqual => write!(f, "BANG_EQUAL {lexeme} null"),
            TokenType::Equal => write!(f, "EQUAL {lexeme} null"),
            TokenType::EqualEqual => write!(f, "EQUAL_EQUAL {lexeme} null"),
            TokenType::Greater => write!(f, "GREATER {lexeme} null"),
            TokenType::GreaterEqual => write!(f, "GREATER_EQUAL {lexeme} null"),
            TokenType::Less => write!(f, "LESS {lexeme} null"),
            TokenType::LessEqual => write!(f, "LESS_EQUAL {lexeme} null"),
            TokenType::Identifier => write!(f, "IDENTIFIER {lexeme} null"),
            TokenType::String => write!(f, "STRING {lexeme} {}", lexeme.trim_matches('"')),
            TokenType::Number => {
                let n = lexeme.parse::<f64>().expect("parse num");
                if n.trunc() == n {
                    write!(f, "NUMBER {lexeme} {n}.0")
                } else {
                    write!(f, "NUMBER {lexeme} {lexeme}",)
                }
            }
            TokenType::And => write!(f, "AND {lexeme} null"),
            TokenType::Class => write!(f, "CLASS {lexeme} null"),
            TokenType::Else => write!(f, "ELSE {lexeme} null"),
            TokenType::False => write!(f, "FALSE {lexeme} null"),
            TokenType::Fun => write!(f, "FUN {lexeme} null"),
            TokenType::For => write!(f, "FOR {lexeme} null"),
            TokenType::If => write!(f, "IF {lexeme} null"),
            TokenType::Nil => write!(f, "NIL {lexeme} null"),
            TokenType::Or => write!(f, "OR {lexeme} null"),
            TokenType::Print => write!(f, "PRINT {lexeme} null"),
            TokenType::Return => write!(f, "RETURN {lexeme} null"),
            TokenType::Super => write!(f, "SUPER {lexeme} null"),
            TokenType::This => write!(f, "THIS {lexeme} null"),
            TokenType::True => write!(f, "TRUE {lexeme} null"),
            TokenType::Var => write!(f, "VAR {lexeme} null"),
            TokenType::While => write!(f, "WHILE {lexeme} null"),
        }
    }
}

#[derive(Debug, Error)]
#[error("Tokenization error")]
pub enum Error {
    #[error("[line {line}] Error: Unexpected character: {character}")]
    UnexpectedCharacter { line: usize, character: char },
    #[error("[line {line}] Error: Unterminated string.")]
    UnterminatedString { line: usize },
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            current: 0,
            line: 1,
        }
    }
}

#[derive(Debug)]
enum Started {
    IfEqualElse(TokenType, TokenType),
    Identifier,
    String,
    Number,
    Slash,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let lexeme_start = self.current;

            let current_char = self.chars.next()?;
            self.current += current_char.len_utf8();

            let single_char_token = |typ| -> Option<Result<Token<'a>, Error>> {
                Some(Ok(Token {
                    typ,
                    lexeme: &self.source[lexeme_start..self.current],
                }))
            };

            let multi_char_token = |typ, lexeme_end| -> Option<Result<Token<'a>, Error>> {
                Some(Ok(Token {
                    typ,
                    lexeme: &self.source[lexeme_start..lexeme_end],
                }))
            };
            let started = match current_char {
                '(' => return single_char_token(TokenType::LeftParen),
                ')' => return single_char_token(TokenType::RightParen),
                '{' => return single_char_token(TokenType::LeftBrace),
                '}' => return single_char_token(TokenType::RightBrace),
                ',' => return single_char_token(TokenType::Comma),
                '.' => return single_char_token(TokenType::Dot),
                '-' => return single_char_token(TokenType::Minus),
                '+' => return single_char_token(TokenType::Plus),
                ';' => return single_char_token(TokenType::Semicolon),
                '*' => return single_char_token(TokenType::Star),
                '=' => Started::IfEqualElse(TokenType::EqualEqual, TokenType::Equal),
                '!' => Started::IfEqualElse(TokenType::BangEqual, TokenType::Bang),
                '>' => Started::IfEqualElse(TokenType::GreaterEqual, TokenType::Greater),
                '<' => Started::IfEqualElse(TokenType::LessEqual, TokenType::Less),
                '/' => Started::Slash,
                '0'..='9' => Started::Number,
                'a'..='z' | 'A'..='Z' | '_' => Started::Identifier,
                '"' => Started::String,
                '\n' => {
                    self.line += 1;
                    continue;
                }
                c if c.is_whitespace() => continue,
                c => {
                    return Some(Err(Error::UnexpectedCharacter {
                        line: self.line,
                        character: c,
                    }));
                }
            };

            match started {
                Started::IfEqualElse(yes, no) => {
                    if self.chars.peek() == Some(&'=') {
                        let c = self.chars.next()?;
                        self.current += c.len_utf8();
                        return multi_char_token(yes, self.current);
                    }
                    return single_char_token(no);
                }
                Started::Identifier => {
                    while let Some(c) = self.chars.peek()
                        && matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
                    {
                        self.current += c.len_utf8();
                        self.chars.next();
                    }
                    return match &self.source[lexeme_start..self.current] {
                        "and" => multi_char_token(TokenType::And, self.current),
                        "class" => multi_char_token(TokenType::Class, self.current),
                        "else" => multi_char_token(TokenType::Else, self.current),
                        "false" => multi_char_token(TokenType::False, self.current),
                        "fun" => multi_char_token(TokenType::Fun, self.current),
                        "for" => multi_char_token(TokenType::For, self.current),
                        "if" => multi_char_token(TokenType::If, self.current),
                        "nil" => multi_char_token(TokenType::Nil, self.current),
                        "or" => multi_char_token(TokenType::Or, self.current),
                        "print" => multi_char_token(TokenType::Print, self.current),
                        "return" => multi_char_token(TokenType::Return, self.current),
                        "super" => multi_char_token(TokenType::Super, self.current),
                        "this" => multi_char_token(TokenType::This, self.current),
                        "true" => multi_char_token(TokenType::True, self.current),
                        "var" => multi_char_token(TokenType::Var, self.current),
                        "while" => multi_char_token(TokenType::While, self.current),
                        _ => multi_char_token(TokenType::Identifier, self.current),
                    };
                }
                Started::String => {
                    loop {
                        match self.chars.next() {
                            Some('"') => {
                                self.current += 1;
                                break;
                            }
                            Some(_) => self.current += 1,
                            None => {
                                return Some(Err(Error::UnterminatedString { line: self.line }));
                            }
                        }
                    }
                    return multi_char_token(TokenType::String, self.current);
                }
                Started::Number => {
                    while let Some(c) = self.chars.peek()
                        && c.is_ascii_digit()
                    {
                        self.current += c.len_utf8();
                        self.chars.next();
                    }
                    if self.chars.peek() == Some(&'.') {
                        // Construct new peekable iterator to look 2 ahead
                        let mut p = self.source[self.current..].chars().peekable();
                        // Move past the dot
                        p.next()?;
                        // Check if it has floating point part
                        if let Some(c) = p.peek()
                            && c.is_ascii_digit()
                        {
                            // Move pass the dot in chars
                            let dot = self.chars.next()?;
                            self.current += dot.len_utf8();

                            while let Some(c) = self.chars.peek()
                                && c.is_ascii_digit()
                            {
                                self.current += c.len_utf8();
                                self.chars.next();
                            }
                        }
                    }
                    return multi_char_token(TokenType::Number, self.current);
                }
                Started::Slash => {
                    // Comment tokenization
                    if self.chars.peek() == Some(&'/') {
                        while let Some(c) = self.chars.next()
                            && c != '\n'
                        {
                            self.current += c.len_utf8();
                        }
                        // Move pass the end line
                        self.current += 1;
                        self.line += 1;
                    } else {
                        return single_char_token(TokenType::Slash);
                    }
                }
            }
        }
    }
}
