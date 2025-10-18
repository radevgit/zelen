//! Lexer for MiniZinc Core Subset
//!
//! Tokenizes MiniZinc source code into a stream of tokens.

use crate::ast::Span;
use crate::error::{Error, ErrorKind, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Array,
    Bool,
    Constraint,
    Float,
    Int,
    Maximize,
    Minimize,
    Of,
    Output,
    Par,
    Satisfy,
    Solve,
    Var,
    Where,
    In,
    
    // Operators
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Div,          // div
    Mod,          // mod
    
    Lt,           // <
    Le,           // <=
    Gt,           // >
    Ge,           // >=
    Eq,           // == or =
    Ne,           // !=
    
    And,          // /\
    Or,           // \/
    Impl,         // ->
    Iff,          // <->
    Not,          // not
    Xor,          // xor
    
    DotDot,       // ..
    
    // Delimiters
    LParen,       // (
    RParen,       // )
    LBracket,     // [
    RBracket,     // ]
    LBrace,       // {
    RBrace,       // }
    
    Comma,        // ,
    Colon,        // :
    ColonColon,   // ::
    Semicolon,    // ;
    Pipe,         // |
    
    // Literals and identifiers
    Ident(String),
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    
    // Special
    Eof,
}

#[derive(Clone)]
pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let current_char = chars.get(0).copied();
        Self {
            source: chars,
            pos: 0,
            current_char,
        }
    }
    
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace_and_comments();
        
        let start = self.pos;
        
        if self.current_char.is_none() {
            return Ok(Token {
                kind: TokenKind::Eof,
                span: Span::new(start, start),
            });
        }
        
        let ch = self.current_char.unwrap();
        
        // Numbers
        if ch.is_ascii_digit() {
            return self.lex_number(start);
        }
        
        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.lex_ident_or_keyword(start);
        }
        
        // String literals
        if ch == '"' {
            return self.lex_string(start);
        }
        
        // Single character tokens and operators
        let kind = match ch {
            '+' => {
                self.advance();
                TokenKind::Plus
            }
            '-' => {
                self.advance();
                if self.current_char == Some('>') {
                    self.advance();
                    TokenKind::Impl
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                self.advance();
                TokenKind::Star
            }
            '/' => {
                self.advance();
                if self.current_char == Some('\\') {
                    self.advance();
                    TokenKind::And
                } else {
                    TokenKind::Slash
                }
            }
            '\\' => {
                self.advance();
                if self.current_char == Some('/') {
                    self.advance();
                    TokenKind::Or
                } else {
                    return Err(Error::new(ErrorKind::UnexpectedChar(ch), Span::new(start, self.pos)));
                }
            }
            '<' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    TokenKind::Le
                } else if self.current_char == Some('-') {
                    self.advance();
                    if self.current_char == Some('>') {
                        self.advance();
                        TokenKind::Iff
                    } else {
                        return Err(Error::new(ErrorKind::UnexpectedChar('<'), Span::new(start, self.pos)));
                    }
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            '=' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                }
                TokenKind::Eq
            }
            '!' => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    TokenKind::Ne
                } else {
                    return Err(Error::new(ErrorKind::UnexpectedChar(ch), Span::new(start, self.pos)));
                }
            }
            '.' => {
                self.advance();
                if self.current_char == Some('.') {
                    self.advance();
                    TokenKind::DotDot
                } else {
                    return Err(Error::new(ErrorKind::UnexpectedChar(ch), Span::new(start, self.pos)));
                }
            }
            '(' => {
                self.advance();
                TokenKind::LParen
            }
            ')' => {
                self.advance();
                TokenKind::RParen
            }
            '[' => {
                self.advance();
                TokenKind::LBracket
            }
            ']' => {
                self.advance();
                TokenKind::RBracket
            }
            '{' => {
                self.advance();
                TokenKind::LBrace
            }
            '}' => {
                self.advance();
                TokenKind::RBrace
            }
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            ':' => {
                self.advance();
                if self.current_char == Some(':') {
                    self.advance();
                    TokenKind::ColonColon
                } else {
                    TokenKind::Colon
                }
            }
            ';' => {
                self.advance();
                TokenKind::Semicolon
            }
            '|' => {
                self.advance();
                TokenKind::Pipe
            }
            _ => {
                return Err(Error::new(ErrorKind::UnexpectedChar(ch), Span::new(start, self.pos)));
            }
        };
        
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
        })
    }
    
    fn advance(&mut self) {
        self.pos += 1;
        self.current_char = self.source.get(self.pos).copied();
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '%' {
                // Line comment
                while self.current_char.is_some() && self.current_char != Some('\n') {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }
    
    fn lex_number(&mut self, start: usize) -> Result<Token> {
        let mut has_dot = false;
        let mut num_str = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot {
                // Check if next char is a digit (to distinguish from ..)
                if let Some(next) = self.source.get(self.pos + 1) {
                    if next.is_ascii_digit() {
                        has_dot = true;
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        let kind = if has_dot {
            match num_str.parse::<f64>() {
                Ok(val) => TokenKind::FloatLit(val),
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::InvalidNumber(num_str),
                        Span::new(start, self.pos),
                    ));
                }
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(val) => TokenKind::IntLit(val),
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::InvalidNumber(num_str),
                        Span::new(start, self.pos),
                    ));
                }
            }
        };
        
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
        })
    }
    
    fn lex_ident_or_keyword(&mut self, start: usize) -> Result<Token> {
        let mut ident = String::new();
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        
        let kind = match ident.as_str() {
            "array" => TokenKind::Array,
            "bool" => TokenKind::Bool,
            "constraint" => TokenKind::Constraint,
            "div" => TokenKind::Div,
            "false" => TokenKind::BoolLit(false),
            "float" => TokenKind::Float,
            "in" => TokenKind::In,
            "int" => TokenKind::Int,
            "maximize" => TokenKind::Maximize,
            "minimize" => TokenKind::Minimize,
            "mod" => TokenKind::Mod,
            "not" => TokenKind::Not,
            "of" => TokenKind::Of,
            "output" => TokenKind::Output,
            "par" => TokenKind::Par,
            "satisfy" => TokenKind::Satisfy,
            "solve" => TokenKind::Solve,
            "true" => TokenKind::BoolLit(true),
            "var" => TokenKind::Var,
            "where" => TokenKind::Where,
            "xor" => TokenKind::Xor,
            _ => TokenKind::Ident(ident),
        };
        
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
        })
    }
    
    fn lex_string(&mut self, start: usize) -> Result<Token> {
        self.advance(); // Skip opening "
        let mut s = String::new();
        
        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance();
                return Ok(Token {
                    kind: TokenKind::StringLit(s),
                    span: Span::new(start, self.pos),
                });
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char {
                    match escaped {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        _ => {
                            s.push('\\');
                            s.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else {
                s.push(ch);
                self.advance();
            }
        }
        
        Err(Error::new(ErrorKind::UnterminatedString, Span::new(start, self.pos)))
    }
    
    pub fn peek_token(&mut self) -> Result<Token> {
        let saved_pos = self.pos;
        let saved_char = self.current_char;
        
        let token = self.next_token()?;
        
        self.pos = saved_pos;
        self.current_char = saved_char;
        
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_all(source: &str) -> Result<Vec<TokenKind>> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        
        loop {
            let token = lexer.next_token()?;
            if token.kind == TokenKind::Eof {
                break;
            }
            tokens.push(token.kind);
        }
        
        Ok(tokens)
    }

    #[test]
    fn test_keywords() {
        let tokens = lex_all("var int constraint solve satisfy").unwrap();
        assert_eq!(
            tokens,
            vec![
                TokenKind::Var,
                TokenKind::Int,
                TokenKind::Constraint,
                TokenKind::Solve,
                TokenKind::Satisfy,
            ]
        );
    }

    #[test]
    fn test_operators() {
        let tokens = lex_all("+ - * / div mod < <= > >= == != /\\ \\/ -> <-> ..").unwrap();
        assert_eq!(tokens.len(), 17);
    }

    #[test]
    fn test_numbers() {
        let tokens = lex_all("42 3.14 0 100").unwrap();
        assert_eq!(
            tokens,
            vec![
                TokenKind::IntLit(42),
                TokenKind::FloatLit(3.14),
                TokenKind::IntLit(0),
                TokenKind::IntLit(100),
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let tokens = lex_all("x queens my_var_123").unwrap();
        assert_eq!(
            tokens,
            vec![
                TokenKind::Ident("x".to_string()),
                TokenKind::Ident("queens".to_string()),
                TokenKind::Ident("my_var_123".to_string()),
            ]
        );
    }

    #[test]
    fn test_string() {
        let tokens = lex_all(r#""hello" "world\n""#).unwrap();
        assert_eq!(
            tokens,
            vec![
                TokenKind::StringLit("hello".to_string()),
                TokenKind::StringLit("world\n".to_string()),
            ]
        );
    }

    #[test]
    fn test_comments() {
        let tokens = lex_all("int % this is a comment\nvar").unwrap();
        assert_eq!(tokens, vec![TokenKind::Int, TokenKind::Var]);
    }
}
