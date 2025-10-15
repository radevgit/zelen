//! Zelen - MiniZinc to Selen Compiler
//!
//! This crate implements a compiler that translates a subset of MiniZinc
//! directly to Selen code, bypassing FlatZinc.

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;

pub use ast::*;
pub use error::{Error, Result};
pub use lexer::Lexer;
pub use parser::Parser;

/// Parse a MiniZinc model from source text
pub fn parse(source: &str) -> Result<Model> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).with_source(source.to_string());
    parser.parse_model()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_model() {
        let source = r#"
            int: n = 5;
            var 1..n: x;
            constraint x > 2;
            solve satisfy;
        "#;
        
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        
        let model = result.unwrap();
        assert_eq!(model.items.len(), 4);
    }

    #[test]
    fn test_parse_nqueens() {
        let source = r#"
            int: n = 4;
            array[1..n] of var 1..n: queens;
            constraint alldifferent(queens);
            solve satisfy;
        "#;
        
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        
        let model = result.unwrap();
        assert_eq!(model.items.len(), 4);
    }

    #[test]
    fn test_parse_with_expressions() {
        let source = r#"
            int: n = 10;
            array[1..n] of var int: x;
            constraint sum(x) == 100;
            constraint forall(i in 1..n)(x[i] >= 0);
            solve minimize sum(i in 1..n)(x[i] * x[i]);
        "#;
        
        let result = parse(source);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[test]
    fn test_error_reporting() {
        let source = "int n = 5"; // Missing colon
        
        let result = parse(source);
        assert!(result.is_err());
        
        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(error_msg.contains("line 1"));
        }
    }
}
