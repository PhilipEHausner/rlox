mod char_stream;
mod token;

use crate::error_handling::{ErrorHandler, LineInformation};
use crate::frontend::scanner::char_stream::CharStream;
pub use crate::frontend::scanner::token::TokenType;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;
pub use token::Token;

pub fn scan(input: &str, error_handler: &ErrorHandler) -> Result<Vec<Token>, ScannerError> {
    let mut scanner = Scanner::new(input, error_handler);
    let result = scanner.scan()?;
    Ok(result)
}

#[derive(Error, Debug)]
#[error("{message:}")]
pub struct ScannerError {
    message: String,
}

impl ScannerError {
    fn new(message: &str) -> ScannerError {
        ScannerError {
            message: message.to_string(),
        }
    }
}

static KEYWORDS: Lazy<HashMap<&str, TokenType>> = Lazy::new(|| {
    HashMap::from([
        ("and", TokenType::And),
        ("bool", TokenType::Bool),
        ("class", TokenType::Class),
        ("else", TokenType::Else),
        ("false", TokenType::False),
        ("float", TokenType::Float),
        ("fun", TokenType::Fun),
        ("for", TokenType::For),
        ("if", TokenType::If),
        ("int", TokenType::Int),
        ("nil", TokenType::Nil),
        ("or", TokenType::Or),
        ("print", TokenType::Print),
        ("return", TokenType::Return),
        ("string", TokenType::String),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("true", TokenType::True),
        ("val", TokenType::Val),
        ("var", TokenType::Var),
        ("while", TokenType::While),
    ])
});

struct Scanner<'a> {
    error_handler: &'a ErrorHandler,
    char_stream: CharStream<'a>,
    token_start: usize,
    had_error: bool,
}

impl<'a> Scanner<'a> {
    fn new(input: &'a str, error_handler: &'a ErrorHandler) -> Scanner<'a> {
        let char_stream = CharStream::new(input);
        Scanner {
            error_handler,
            char_stream,
            token_start: 0,
            had_error: false,
        }
    }

    fn scan(&mut self) -> Result<Vec<Token>, ScannerError> {
        let mut result: Vec<Token> = vec![];

        self.had_error = false;
        self.char_stream.reset();

        while !self.char_stream.is_exhausted() {
            let token = self.next_token()?;
            match token {
                None => continue,
                Some(t) => result.push(t),
            }
        }

        result.push(Token::new(
            TokenType::EOF,
            LineInformation::new(self.char_stream.get_position(), 0),
        ));

        match self.had_error {
            true => Err(ScannerError::new("Error scanning file.")),
            false => Ok(result),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, ScannerError> {
        self.token_start = self.char_stream.get_position();

        match self.char_stream.next() {
            None => Err(ScannerError::new(&format!(
                "Critical: Scanner could not read next character at position {}.",
                self.char_stream.get_position()
            ))),
            Some(c) => match c {
                // Single char tokens.
                '(' => self.create_token(TokenType::LeftParenthesis),
                ')' => self.create_token(TokenType::RightParenthesis),
                '{' => self.create_token(TokenType::LeftBrace),
                '}' => self.create_token(TokenType::RightBrace),
                ':' => self.create_token(TokenType::Colon),
                ',' => self.create_token(TokenType::Comma),
                '.' => self.create_token(TokenType::Dot),
                '-' => self.create_token(TokenType::Minus),
                '+' => self.create_token(TokenType::Plus),
                ';' => self.create_token(TokenType::Semicolon),
                '*' => self.create_token(TokenType::Star),
                // Single or two character tokens.
                '/' => match self.char_stream.peek() {
                    None => self.create_token(TokenType::Slash),
                    Some(x) => match x {
                        '/' => {
                            self.process_comment();
                            Ok(None)
                        }
                        _ => self.create_token(TokenType::Slash),
                    },
                },
                '!' => match self.char_stream.matches('=') {
                    true => self.create_token(TokenType::BangEqual),
                    false => self.create_token(TokenType::Bang),
                },
                '=' => match self.char_stream.matches('=') {
                    true => self.create_token(TokenType::EqualEqual),
                    false => self.create_token(TokenType::Equal),
                },
                '>' => match self.char_stream.matches('=') {
                    true => self.create_token(TokenType::GreaterEqual),
                    false => self.create_token(TokenType::Greater),
                },
                '<' => match self.char_stream.matches('=') {
                    true => self.create_token(TokenType::LessEqual),
                    false => self.create_token(TokenType::Less),
                },
                // Whitespace is ignored.
                ' ' | '\r' | '\t' | '\n' => Ok(None),
                // Strings
                '"' => self.process_string(),
                // Character is invalid.
                _ => {
                    if !matches!(c, '0'..='9') {
                        self.process_number(c)
                    } else if self.is_valid_id_start(&c) {
                        self.process_identifier(c)
                    } else {
                        self.process_error(&format!("Unexpected character '{c}'."));
                        Ok(None)
                    }
                }
            },
        }
    }

    // Consume characters until end of line (or end of file, whichever is sooner).
    fn process_comment(&mut self) {
        while let Some(c) = self.char_stream.next() {
            if c == '\n' {
                break;
            }
        }
    }

    fn process_string(&mut self) -> Result<Option<Token>, ScannerError> {
        let mut s = "".to_string();
        let result;

        loop {
            let r = self.char_stream.next();
            match r {
                None => {
                    self.process_error("Unterminated string.");
                    result = Ok(None);
                    break;
                }
                Some(c) => match c {
                    '"' => {
                        result = self.create_token(TokenType::StringValue(s));
                        break;
                    }
                    _ => s.push(c),
                },
            }
        }

        result
    }

    fn process_number(&mut self, start: char) -> Result<Option<Token>, ScannerError> {
        let mut number = start.to_string();

        // Every number has to start with a flow of digits.
        while let Some(c) = self.char_stream.current_char() {
            if !(matches!(c, '0'..='9')) {
                break;
            };
            number.push(c);
            self.char_stream.next();
        }

        let is_float = self.number_is_float();
        if is_float {
            number.push(self.char_stream.next().unwrap());
            while let Some(n) = self.char_stream.current_char() {
                if !(matches!(n, '0'..='9')) {
                    break;
                }
                number.push(n);
                self.char_stream.next();
            }
        }

        match is_float {
            true => self.parse_float(&number),
            false => self.parse_int(&number),
        }
    }

    fn number_is_float(&self) -> bool {
        let r1 = self.char_stream.current_char();
        let r2 = self.char_stream.peek();
        match (r1, r2) {
            (Some(c1), Some(c2)) => c1 == '.' && matches!(c2, '0'..='9'),
            (_, _) => false,
        }
    }

    fn parse_int(&mut self, number: &str) -> Result<Option<Token>, ScannerError> {
        match number.parse::<i64>() {
            Ok(n) => self.create_token(TokenType::IntegerValue(n)),
            Err(_) => {
                self.process_error(&format!("Cannot parse integer {}", number));
                Ok(None)
            }
        }
    }

    fn parse_float(&mut self, number: &str) -> Result<Option<Token>, ScannerError> {
        match number.parse::<f64>() {
            Ok(n) => self.create_token(TokenType::FloatValue(n)),
            Err(_) => {
                self.process_error(&format!("Cannot parse float {}", number));
                Ok(None)
            }
        }
    }

    fn process_identifier(&mut self, start: char) -> Result<Option<Token>, ScannerError> {
        let mut identifier = start.to_string();

        while let Some(c) = self.char_stream.current_char() {
            if !self.is_valid_id_char(&c) {
                break;
            };
            identifier.push(c);
            self.char_stream.next();
        }

        match KEYWORDS.get(&identifier as &str) {
            None => self.create_token(TokenType::Identifier(identifier)),
            Some(t) => self.create_token(t.clone()),
        }
    }

    fn is_valid_id_start(&self, c: &char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }

    fn is_valid_id_char(&self, c: &char) -> bool {
        self.is_valid_id_start(c) || matches!(c, '0'..='9')
    }

    fn create_token(&mut self, token_type: TokenType) -> Result<Option<Token>, ScannerError> {
        let li = self.get_line_information();
        Ok(Some(Token::new(token_type, li)))
    }

    fn get_line_information(&self) -> LineInformation {
        LineInformation::new(
            self.token_start,
            self.char_stream.get_position() - self.token_start,
        )
    }

    fn process_error(&mut self, error_msg: &str) {
        self.error_handler
            .report_error(error_msg, &self.get_line_information());
        self.had_error = true;
    }
}
