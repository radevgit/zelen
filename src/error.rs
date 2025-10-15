//! Error types and reporting for MiniZinc parser

use std::fmt;
use crate::ast::Span;

pub type Result<T> = std::result::Result<T, Error>;

/// Parser and compiler errors
#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    // Lexer errors
    UnexpectedChar(char),
    UnterminatedString,
    InvalidNumber(String),
    
    // Parser errors
    UnexpectedToken {
        expected: String,
        found: String,
    },
    UnexpectedEof,
    InvalidExpression(String),
    InvalidTypeInst(String),
    
    // Semantic errors
    UnsupportedFeature {
        feature: String,
        phase: String,
        workaround: Option<String>,
    },
    TypeError {
        expected: String,
        found: String,
    },
    DuplicateDeclaration(String),
    UndefinedVariable(String),
    
    // General
    Message(String),
}

impl Error {
    pub fn new(kind: ErrorKind, span: Span) -> Self {
        Self {
            kind,
            span,
            source: None,
        }
    }
    
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }
    
    pub fn unexpected_token(expected: &str, found: &str, span: Span) -> Self {
        Self::new(
            ErrorKind::UnexpectedToken {
                expected: expected.to_string(),
                found: found.to_string(),
            },
            span,
        )
    }
    
    pub fn unexpected_eof(span: Span) -> Self {
        Self::new(ErrorKind::UnexpectedEof, span)
    }
    
    pub fn unsupported_feature(feature: &str, phase: &str, span: Span) -> Self {
        Self::new(
            ErrorKind::UnsupportedFeature {
                feature: feature.to_string(),
                phase: phase.to_string(),
                workaround: None,
            },
            span,
        )
    }
    
    pub fn with_workaround(mut self, workaround: &str) -> Self {
        if let ErrorKind::UnsupportedFeature { workaround: w, .. } = &mut self.kind {
            *w = Some(workaround.to_string());
        }
        self
    }
    
    /// Get the line and column of the error in the source
    pub fn location(&self) -> (usize, usize) {
        if let Some(source) = &self.source {
            let mut line = 1;
            let mut col = 1;
            for (i, c) in source.chars().enumerate() {
                if i >= self.span.start {
                    break;
                }
                if c == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
            }
            (line, col)
        } else {
            (0, 0)
        }
    }
    
    /// Get the line of source code where the error occurred
    pub fn source_line(&self) -> Option<String> {
        self.source.as_ref().map(|source| {
            let lines: Vec<&str> = source.lines().collect();
            let (line_num, _) = self.location();
            if line_num > 0 && line_num <= lines.len() {
                lines[line_num - 1].to_string()
            } else {
                String::new()
            }
        })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.location();
        
        write!(f, "Error")?;
        if line > 0 {
            write!(f, " at line {}, column {}", line, col)?;
        }
        write!(f, ": ")?;
        
        match &self.kind {
            ErrorKind::UnexpectedChar(c) => {
                write!(f, "Unexpected character '{}'", c)
            }
            ErrorKind::UnterminatedString => {
                write!(f, "Unterminated string literal")
            }
            ErrorKind::InvalidNumber(s) => {
                write!(f, "Invalid number: {}", s)
            }
            ErrorKind::UnexpectedToken { expected, found } => {
                write!(f, "Expected {}, found {}", expected, found)
            }
            ErrorKind::UnexpectedEof => {
                write!(f, "Unexpected end of file")
            }
            ErrorKind::InvalidExpression(msg) => {
                write!(f, "Invalid expression: {}", msg)
            }
            ErrorKind::InvalidTypeInst(msg) => {
                write!(f, "Invalid type-inst: {}", msg)
            }
            ErrorKind::UnsupportedFeature { feature, phase, workaround } => {
                write!(f, "Unsupported feature: {}", feature)?;
                write!(f, " (will be supported in {})", phase)?;
                if let Some(w) = workaround {
                    write!(f, "\nWorkaround: {}", w)?;
                }
                Ok(())
            }
            ErrorKind::TypeError { expected, found } => {
                write!(f, "Type error: expected {}, found {}", expected, found)
            }
            ErrorKind::DuplicateDeclaration(name) => {
                write!(f, "Duplicate declaration of '{}'", name)
            }
            ErrorKind::UndefinedVariable(name) => {
                write!(f, "Undefined variable '{}'", name)
            }
            ErrorKind::Message(msg) => {
                write!(f, "{}", msg)
            }
        }?;
        
        if let Some(source_line) = self.source_line() {
            write!(f, "\n  {}", source_line)?;
            let (_, col) = self.location();
            if col > 0 {
                write!(f, "\n  {}{}", " ".repeat(col - 1), "^")?;
            }
        }
        
        Ok(())
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self::new(ErrorKind::Message(msg), Span::dummy())
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self::new(ErrorKind::Message(msg.to_string()), Span::dummy())
    }
}
