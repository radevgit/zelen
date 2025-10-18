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
//! ```
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
//!     Ok(Ok(solution)) => { /* Found solution! */ },
//!     Ok(Err(_)) => { /* No solution exists */ },
//!     Err(e) => { /* Parse error */ },
//! }
//! ```
//!
//! ## With Variable Access
//!
//! ```
//! use zelen::Translator;
//!
//! let source = "var 1..10: x; solve satisfy;";
//! let ast = zelen::parse(source).unwrap();
//! let model_data = Translator::translate_with_vars(&ast).unwrap();
//!
//! // Access variables by name
//! for (name, var_id) in &model_data.int_vars {
//!     // name is available here
//!     let _ = (name, var_id);
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
pub mod error;
pub mod lexer;
pub mod parser;
pub mod translator;

pub use ast::*;
pub use error::{Error, Result};
pub use lexer::Lexer;
pub use parser::Parser;
pub use translator::{Translator, TranslatedModel, ObjectiveType};

// Re-export commonly used Selen types for convenience
pub use selen;
// Re-export specific selen types to avoid conflicts
pub use selen::prelude::{Model, Solution, VarId};

/// Configuration for the Selen solver backend
///
/// Allows customizing solver behavior like timeout, memory limits, and solution enumeration.
///
/// # Example
///
/// ```
/// use zelen::SolverConfig;
///
/// let config = SolverConfig::default()
///     .with_time_limit_ms(5000)
///     .with_memory_limit_mb(1024)
///     .with_all_solutions(true);
/// assert_eq!(config.time_limit_ms, Some(5000));
/// ```
#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Time limit in milliseconds (None = use Selen default)
    pub time_limit_ms: Option<u64>,
    /// Memory limit in MB (None = use Selen default)
    pub memory_limit_mb: Option<u64>,
    /// Whether to find all solutions (for satisfaction problems)
    pub all_solutions: bool,
    /// Maximum number of solutions to find (None = unlimited)
    pub max_solutions: Option<usize>,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            time_limit_ms: None,
            memory_limit_mb: None,
            all_solutions: false,
            max_solutions: None,
        }
    }
}

impl SolverConfig {
    /// Set the time limit in milliseconds
    pub fn with_time_limit_ms(mut self, ms: u64) -> Self {
        self.time_limit_ms = if ms > 0 { Some(ms) } else { None };
        self
    }

    /// Set the memory limit in MB
    pub fn with_memory_limit_mb(mut self, mb: u64) -> Self {
        self.memory_limit_mb = if mb > 0 { Some(mb) } else { None };
        self
    }

    /// Enable finding all solutions
    pub fn with_all_solutions(mut self, all: bool) -> Self {
        self.all_solutions = all;
        self
    }

    /// Set the maximum number of solutions to find
    pub fn with_max_solutions(mut self, n: usize) -> Self {
        self.max_solutions = if n > 0 { Some(n) } else { None };
        self
    }

    /// Convert to Selen's SolverConfig
    fn to_selen_config(&self) -> selen::utils::config::SolverConfig {
        let mut config = selen::utils::config::SolverConfig::default();
        if let Some(ms) = self.time_limit_ms {
            config.timeout_ms = Some(ms);
        }
        if let Some(mb) = self.memory_limit_mb {
            config.max_memory_mb = Some(mb);
        }
        config
    }
}

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
/// ```
/// let ast = zelen::parse("var 1..10: x; solve satisfy;");
/// assert!(ast.is_ok());
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
/// ```
/// let ast = zelen::parse("var 1..10: x; solve satisfy;").unwrap();
/// let model = zelen::translate(&ast);
/// assert!(model.is_ok());
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
/// ```
/// let model = zelen::build_model(r#"
///     var 1..10: x;
///     constraint x > 5;
///     solve satisfy;
/// "#);
/// assert!(model.is_ok());
/// ```
pub fn build_model(source: &str) -> Result<selen::prelude::Model> {
    let ast = parse(source)?;
    translate(&ast)
}

/// Parse and translate MiniZinc source directly to a Selen model with custom configuration
///
/// This version allows configuring solver parameters like timeouts and memory limits.
///
/// # Arguments
///
/// * `source` - MiniZinc source code as a string
/// * `config` - Solver configuration
///
/// # Returns
///
/// A Selen Model ready to solve, or an error (either parsing or translation)
///
/// # Example
///
/// ```
/// let config = zelen::SolverConfig::default()
///     .with_time_limit_ms(5000)
///     .with_memory_limit_mb(1024);
/// 
/// let model = zelen::build_model_with_config("var 1..10: x; solve satisfy;", config);
/// assert!(model.is_ok());
/// ```
pub fn build_model_with_config(source: &str, config: SolverConfig) -> Result<selen::prelude::Model> {
    let ast = parse(source)?;
    let selen_config = config.to_selen_config();
    Translator::translate_with_config(&ast, selen_config)
}

/// Solve a MiniZinc model with custom solver configuration and return solutions
///
/// This function combines parse, translate with config, and solve/enumerate.
/// It respects the `all_solutions` and `max_solutions` flags from the config.
///
/// # Arguments
///
/// * `source` - MiniZinc source code as a string
/// * `config` - Solver configuration including all_solutions and max_solutions settings
///
/// # Returns
///
/// A vector of solutions found. If `all_solutions` is false, returns at most one solution.
/// If `all_solutions` is true, returns multiple solutions up to `max_solutions` limit.
///
/// # Example
///
/// ```
/// let config = zelen::SolverConfig::default()
///     .with_all_solutions(false)
///     .with_time_limit_ms(5000);
///
/// let solutions = zelen::solve_with_config("var 1..10: x; solve satisfy;", config);
/// assert!(solutions.is_ok());
/// ```
pub fn solve_with_config(
    source: &str,
    config: SolverConfig,
) -> Result<Vec<selen::core::Solution>> {
    let model = build_model_with_config(source, config.clone())?;
    
    if config.all_solutions {
        // Enumerate all solutions up to max_solutions limit
        let max = config.max_solutions.unwrap_or(usize::MAX);
        Ok(model.enumerate().take(max).collect())
    } else {
        // Single solution
        match model.solve() {
            Ok(solution) => Ok(vec![solution]),
            Err(_) => Ok(Vec::new()),  // No solution found
        }
    }
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
/// ```
/// match zelen::solve("var 1..10: x; solve satisfy;") {
///     Ok(Ok(solution)) => assert!(true), // Solution found
///     Ok(Err(_)) => assert!(true), // Unsatisfiable
///     Err(e) => panic!("Parse error: {}", e),
/// }
/// ```
pub fn solve(source: &str) -> Result<std::result::Result<selen::core::Solution, selen::core::SolverError>> {
    let model = build_model(source)?;
    Ok(model.solve())
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
