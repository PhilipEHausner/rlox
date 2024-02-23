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
                '/' => match self.char_stream.current_char() {
                    None => self.create_token(TokenType::Slash),
                    Some(x) => match x {
                        '/' => {
                            self.process_comment();
                            Ok(None)
                        }
                        '*' => {
                            self.char_stream.next();
                            self.process_multiline_comment();
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
                    if matches!(c, '0'..='9') {
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

    fn process_multiline_comment(&mut self) {
        while let Some(c1) = self.char_stream.next() {
            if c1 != '*' {
                continue;
            }
            match self.char_stream.current_char() {
                None => {}
                Some(c2) => {
                    if c2 == '/' {
                        self.char_stream.next();
                        return;
                    }
                }
            }
        }

        self.char_stream.revert();
        self.process_error("Unterminated multiline comment.");
    }

    fn process_string(&mut self) -> Result<Option<Token>, ScannerError> {
        let mut s = "".to_string();
        let result;

        loop {
            let r = self.char_stream.next();
            match r {
                None => {
                    self.char_stream.revert();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn simulate_scan_input(input: &str) -> Result<Vec<TokenType>, ScannerError> {
        let error_handler = ErrorHandler::new(input);
        let tokens = scan(&input, &error_handler)?;
        Ok(tokens
            .iter()
            .map(|token| token.token_type().clone())
            .collect())
    }

    #[test]
    fn test_function_block() {
        let input = "fun myFunction(a: int): string {\nreturn \"result\"\n}".to_string();

        let result = simulate_scan_input(&input).unwrap();

        assert_eq!(
            result,
            vec![
                TokenType::Fun,
                TokenType::Identifier("myFunction".to_string()),
                TokenType::LeftParenthesis,
                TokenType::Identifier("a".to_string()),
                TokenType::Colon,
                TokenType::Int,
                TokenType::RightParenthesis,
                TokenType::Colon,
                TokenType::String,
                TokenType::LeftBrace,
                TokenType::Return,
                TokenType::StringValue("result".to_string()),
                TokenType::RightBrace,
                TokenType::EOF,
            ]
        )
    }

    #[test]
    fn test_single_token_types() {
        let input = "+ - * / ( ) { } , ; : = ! == < <= > >=".to_string();
        let expected_tokens = vec![
            TokenType::Plus,
            TokenType::Minus,
            TokenType::Star,
            TokenType::Slash,
            TokenType::LeftParenthesis,
            TokenType::RightParenthesis,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Comma,
            TokenType::Semicolon,
            TokenType::Colon,
            TokenType::Equal,
            TokenType::Bang,
            TokenType::EqualEqual,
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::EOF,
        ];
        assert_eq!(simulate_scan_input(&input).unwrap(), expected_tokens);
    }

    #[test]
    fn test_two_character_tokens() {
        let input = "!= == <= >=".to_string();
        let expected_tokens = vec![
            TokenType::BangEqual,
            TokenType::EqualEqual,
            TokenType::LessEqual,
            TokenType::GreaterEqual,
            TokenType::EOF,
        ];
        assert_eq!(simulate_scan_input(&input).unwrap(), expected_tokens);
    }

    #[test]
    fn test_identifiers_and_keywords() {
        let input = "class MyClass fun myFunction if else true false var x".to_string();
        let expected_tokens = vec![
            TokenType::Class,
            TokenType::Identifier("MyClass".to_string()),
            TokenType::Fun,
            TokenType::Identifier("myFunction".to_string()),
            TokenType::If,
            TokenType::Else,
            TokenType::True,
            TokenType::False,
            TokenType::Var,
            TokenType::Identifier("x".to_string()),
            TokenType::EOF,
        ];
        assert_eq!(simulate_scan_input(&input).unwrap(), expected_tokens);
    }

    #[test]
    fn test_integers_and_floats() {
        let input = "123 45.67 0 -987.65".to_string();
        let expected_tokens = vec![
            TokenType::IntegerValue(123),
            TokenType::FloatValue(45.67),
            TokenType::IntegerValue(0),
            TokenType::Minus,
            TokenType::FloatValue(987.65),
            TokenType::EOF,
        ];
        assert_eq!(simulate_scan_input(&input).unwrap(), expected_tokens);
    }

    #[test]
    fn test_strings() {
        let input = "\"hello\" \"world\" \"123\"".to_string();
        let expected_tokens = vec![
            TokenType::StringValue("hello".to_string()),
            TokenType::StringValue("world".to_string()),
            TokenType::StringValue("123".to_string()),
            TokenType::EOF,
        ];
        assert_eq!(simulate_scan_input(&input).unwrap(), expected_tokens);
    }

    #[test]
    fn test_comments() {
        let input = "// This is a comment".to_string();
        let expected_tokens = vec![TokenType::EOF];
        assert_eq!(simulate_scan_input(&input).unwrap(), expected_tokens);
    }

    #[test]
    fn test_multiline_comment() {
        let input = "/* This is a comment\n This is the second line */".to_string();
        let expected_tokens = vec![TokenType::EOF];
        assert_eq!(simulate_scan_input(&input).unwrap(), expected_tokens);
    }

    #[test]
    fn test_error_handling_invalid_characters() {
        let input = "$ %".to_string();
        let error_handler = ErrorHandler::new(&input);
        let res = scan(&input, &error_handler);
        assert!(res.is_err());
    }

    #[test]
    fn test_error_handling_unterminated_strings() {
        let input = "\"unterminated string".to_string();
        assert!(simulate_scan_input(&input).is_err());
    }

    #[test]
    fn test_complex_scenarios() {
        let input =
            "if (x == 1) { print(\"x is 1\"); } else { print(\"x is not 1\"); }".to_string();
        let expected_tokens = vec![
            TokenType::If,
            TokenType::LeftParenthesis,
            TokenType::Identifier("x".to_string()),
            TokenType::EqualEqual,
            TokenType::IntegerValue(1),
            TokenType::RightParenthesis,
            TokenType::LeftBrace,
            TokenType::Print,
            TokenType::LeftParenthesis,
            TokenType::StringValue("x is 1".to_string()),
            TokenType::RightParenthesis,
            TokenType::Semicolon,
            TokenType::RightBrace,
            TokenType::Else,
            TokenType::LeftBrace,
            TokenType::Print,
            TokenType::LeftParenthesis,
            TokenType::StringValue("x is not 1".to_string()),
            TokenType::RightParenthesis,
            TokenType::Semicolon,
            TokenType::RightBrace,
            TokenType::EOF,
        ];
        assert_eq!(simulate_scan_input(&input).unwrap(), expected_tokens);
    }
}
