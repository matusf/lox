use std::fmt::Display;

use thiserror::Error;

pub struct Tokenizer<'a> {
    source: &'a str,
    current: usize,
    lexeme_start: usize,
    line: usize,
}

#[derive(Debug)]
pub struct Token<'a> {
    lexeme: &'a str,
    // offset: usize,
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

    Eof,
}

impl<'a> Display for Token<'a> {
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
            TokenType::String => write!(f, "STRING {lexeme} {lexeme}"),
            TokenType::Number => write!(f, "NUMBER {lexeme} {lexeme}"),
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
            TokenType::Eof => write!(f, "EOF {lexeme} null"),
        }
    }
}

#[derive(Debug, Error)]
#[error("[line {line}] Error: Unexpected character: {character}")]
pub struct Error {
    line: usize,
    character: char,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            current: 0,
            lexeme_start: 0,
            line: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.len()
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.is_at_end() {
                return None;
            }

            let mut chars = self.source[self.current..].chars().peekable();
            self.lexeme_start = self.current;

            let c = chars.next()?;
            let c_len = c.len_utf8();
            self.current += c_len;

            let single_char_token = |typ| -> Option<Result<Token<'a>, Error>> {
                Some(Ok(Token {
                    typ,
                    lexeme: &self.source[self.lexeme_start..self.current],
                }))
            };

            let multi_char_token = |typ, current| -> Option<Result<Token<'a>, Error>> {
                Some(Ok(Token {
                    typ,
                    lexeme: &self.source[self.lexeme_start..current],
                }))
            };
            match c {
                '(' => return single_char_token(TokenType::LeftParen),
                ')' => return single_char_token(TokenType::RightParen),
                '{' => return single_char_token(TokenType::LeftBrace),
                '}' => return single_char_token(TokenType::RightBrace),
                ',' => return single_char_token(TokenType::Comma),
                '.' => return single_char_token(TokenType::Dot),
                '-' => return single_char_token(TokenType::Minus),
                '+' => return single_char_token(TokenType::Plus),
                ';' => return single_char_token(TokenType::Semicolon),
                '/' => return single_char_token(TokenType::Slash),
                '*' => return single_char_token(TokenType::Star),
                '=' => {
                    if Some(&'=') == chars.peek() {
                        let c = chars.next()?;
                        self.current += c.len_utf8();
                        return multi_char_token(TokenType::EqualEqual, self.current);
                    }
                    return single_char_token(TokenType::Equal);
                }
                '!' => {
                    if Some(&'=') == chars.peek() {
                        let c = chars.next()?;
                        self.current += c.len_utf8();
                        return multi_char_token(TokenType::BangEqual, self.current);
                    }
                    return single_char_token(TokenType::Bang);
                }

                '\n' => self.line += 1,
                c if c.is_whitespace() => {
                    continue;
                }
                c => {
                    return Some(Err(Error {
                        line: self.line,
                        character: c,
                    }));
                }
            };
        }
    }
}
