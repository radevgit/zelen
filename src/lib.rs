//! Zelen - MiniZinc Constraint Solver
//!
//! Zelen parses a subset of MiniZinc and translates it directly to Selen models,
//! bypassing FlatZinc. It can either solve models directly or export them to Rust code.

pub mod ast;
pub mod compiler;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod translator;

pub use ast::*;
pub use compiler::Compiler;
pub use error::{Error, Result};
pub use lexer::Lexer;
pub use parser::Parser;
pub use translator::{Translator, TranslatedModel, ObjectiveType};

// Re-export commonly used Selen types for convenience
pub use selen;
// Re-export specific selen types to avoid conflicts
pub use selen::prelude::{Model, Solution, VarId};

/// Parse a MiniZinc model from source text into an AST
pub fn parse(source: &str) -> Result<ast::Model> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).with_source(source.to_string());
    parser.parse_model()
}

/// Translate a MiniZinc AST to a Selen model
pub fn translate(ast: &ast::Model) -> Result<selen::prelude::Model> {
    Translator::translate(ast)
}

/// Parse and translate MiniZinc source directly to a Selen model
pub fn build_model(source: &str) -> Result<selen::prelude::Model> {
    let ast = parse(source)?;
    translate(&ast)
}

/// Solve a MiniZinc model and return the solution
/// 
/// Returns a `Result<Solution>` where the outer `Result` is from Zelen (parsing/translation errors)
/// and the inner `Result` is from Selen (solving errors).
/// 
/// # Example
/// ```ignore
/// let solution = zelen::solve(source)??;  // Note the double ? for both Results
/// println!("Found solution");
/// ```
pub fn solve(source: &str) -> Result<std::result::Result<selen::core::Solution, selen::core::SolverError>> {
    let model = build_model(source)?;
    Ok(model.solve())
}

/// Compile a MiniZinc model to Rust code (for code generation)
#[deprecated(note = "Use build_model() to create Selen models directly")]
pub fn compile(source: &str) -> Result<String> {
    let model = parse(source)?;
    let mut compiler = Compiler::new();
    compiler.compile(&model)
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
