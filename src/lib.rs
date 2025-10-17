//! Zelen - Direct MiniZinc Constraint Solver
//!
//! Zelen parses a subset of MiniZinc and translates it directly to the [Selen](https://github.com/radevgit/selen) 
//! constraint solver, bypassing FlatZinc compilation. This allows you to:
//!
//! - **Parse MiniZinc** from strings or files
//! - **Solve directly** using a single function call
//! - **Access variable information** with variable name to ID mappings
//! - **Use as a library** in your Rust projects
//!
//! # Quick Start
//!
//! ## Simple Usage
//!
//! ```ignore
//! use zelen;
//!
//! let source = r#"
//!     var 1..10: x;
//!     var 1..10: y;
//!     constraint x + y = 15;
//!     solve satisfy;
//! "#;
//!
//! // Parse and solve directly
//! match zelen::solve(source) {
//!     Ok(Ok(solution)) => println!("Found solution!"),
//!     Ok(Err(_)) => println!("No solution exists"),
//!     Err(e) => println!("Parse error: {}", e),
//! }
//! ```
//!
//! ## With Variable Access
//!
//! ```ignore
//! use zelen::Translator;
//!
//! let source = "var 1..10: x; solve satisfy;";
//! let ast = zelen::parse(source)?;
//! let model_data = Translator::translate_with_vars(&ast)?;
//!
//! // Access variables by name
//! for (name, var_id) in &model_data.int_vars {
//!     println!("Integer variable: {}", name);
//! }
//!
//! // Solve and get results
//! let solution = model_data.model.solve()?;
//! for (name, var_id) in &model_data.int_vars {
//!     println!("{} = {}", name, solution.get_int(*var_id));
//! }
//! ```
//!
//! # Supported Features
//!
//! - Integer, boolean, and float variables
//! - Variable arrays with initialization
//! - Arithmetic and comparison operators
//! - Boolean logic operators
//! - Global constraints: `all_different`, `element`
//! - Aggregation functions: `min`, `max`, `sum`, `forall`, `exists`
//! - Nested forall loops
//! - Satisfy, minimize, and maximize objectives

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
///
/// # Arguments
///
/// * `source` - MiniZinc source code as a string
///
/// # Returns
///
/// An AST (Abstract Syntax Tree) representing the model, or a parsing error
///
/// # Example
///
/// ```ignore
/// let ast = zelen::parse("var 1..10: x; solve satisfy;")?;
/// ```
pub fn parse(source: &str) -> Result<ast::Model> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer).with_source(source.to_string());
    parser.parse_model()
}

/// Translate a MiniZinc AST to a Selen model
///
/// # Arguments
///
/// * `ast` - The MiniZinc AST to translate
///
/// # Returns
///
/// A Selen Model ready to solve, or a translation error
///
/// # Example
///
/// ```ignore
/// let ast = zelen::parse("var 1..10: x; solve satisfy;")?;
/// let model = zelen::translate(&ast)?;
/// let solution = model.solve()?;
/// ```
pub fn translate(ast: &ast::Model) -> Result<selen::prelude::Model> {
    Translator::translate(ast)
}

/// Parse and translate MiniZinc source directly to a Selen model
///
/// This is a convenience function that combines `parse()` and `translate()`.
///
/// # Arguments
///
/// * `source` - MiniZinc source code as a string
///
/// # Returns
///
/// A Selen Model ready to solve, or an error (either parsing or translation)
///
/// # Example
///
/// ```ignore
/// let model = zelen::build_model(r#"
///     var 1..10: x;
///     constraint x > 5;
///     solve satisfy;
/// "#)?;
///
/// let solution = model.solve()?;
/// ```
pub fn build_model(source: &str) -> Result<selen::prelude::Model> {
    let ast = parse(source)?;
    translate(&ast)
}

/// Solve a MiniZinc model and return the solution
///
/// This is a convenience function that combines parse, translate, and solve.
///
/// # Arguments
///
/// * `source` - MiniZinc source code as a string
///
/// # Returns
///
/// Returns a nested Result:
/// - Outer `Result`: Parsing/translation errors
/// - Inner `Result`: Solver errors (satisfiability, resource limits, etc.)
///
/// # Example
///
/// ```ignore
/// match zelen::solve(source) {
///     Ok(Ok(solution)) => println!("Found solution!"),
///     Ok(Err(_)) => println!("Problem is unsatisfiable"),
///     Err(e) => println!("Parse error: {}", e),
/// }
///
/// // Or using the ? operator
/// let solution = zelen::solve(source)??;  // Note: double ? for both Results
/// println!("Found solution!");
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
