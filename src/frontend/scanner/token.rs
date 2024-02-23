use crate::error_handling::LineInformation;

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    // Single-character tokens
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier(String),
    StringValue(String),
    FloatValue(f64),
    IntegerValue(i64),

    // Keywords
    And,
    Bool,
    Class,
    Else,
    False,
    Float,
    Fun,
    For,
    If,
    Int,
    Nil,
    Or,
    Print,
    Return,
    String,
    Super,
    This,
    True,
    Val,
    Var,
    While,

    // Special Character
    EOF,
}

pub struct Token {
    token_type: TokenType,
    line_information: LineInformation,
}

impl Token {
    pub fn new(token_type: TokenType, line_information: LineInformation) -> Token {
        Token {
            token_type,
            line_information,
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }

    pub fn line_information(&self) -> &LineInformation {
        &self.line_information
    }
}
