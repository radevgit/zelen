//! # Zelen - FlatZinc Frontend for Selen
//! 
//! Zelen provides FlatZinc parsing and integration with the Selen constraint solver.
//! 
//! ## Quick Start
//! 
//! ```rust
//! use zelen::prelude::*;
//! 
//! // Define a simple FlatZinc problem
//! let fzn = r#"
//!     var 1..10: x;
//!     var 1..10: y;
//!     constraint int_eq(x, 5);
//!     constraint int_plus(x, y, 12);
//!     solve satisfy;
//! "#;
//! 
//! // Create solver and solve
//! let mut solver = FlatZincSolver::new();
//! solver.load_str(fzn).unwrap();
//! 
//! if solver.solve().is_ok() {
//!     // Get FlatZinc-formatted output
//!     let output = solver.to_flatzinc();
//!     assert!(output.contains("x = 5"));
//!     assert!(output.contains("y = 7"));
//! }
//! ```
//! 
//! ## Main API
//! 
//! The primary way to use Zelen is through the [`FlatZincSolver`] which provides
//! automatic FlatZinc parsing and spec-compliant output formatting.
//! 
//! For more control, you can use the lower-level [`FlatZincModel`] trait or work
//! with individual modules directly.
//! 
//! See the [`prelude`] module for commonly used types and traits.

// Internal implementation modules - hidden from docs by default
#[doc(hidden)]
pub mod ast;
pub mod error;
#[doc(hidden)]
pub mod tokenizer;
#[doc(hidden)]
pub mod parser;
#[doc(hidden)]
pub mod mapper;

// Public API modules
pub mod output;
pub mod solver;
pub mod integration;
#[doc(hidden)]
pub mod exporter;

pub use error::{FlatZincError, FlatZincResult};
pub use solver::{FlatZincSolver, FlatZincContext, SolverOptions};
pub use integration::FlatZincModel;

// Re-export selen for convenience, but hide its docs since users should refer to selen's own docs
#[doc(no_inline)]
pub use selen;

/// Prelude module for convenient imports.
///
/// This module re-exports the most commonly used types and traits.
/// 
/// # Example
/// 
/// ```rust
/// use zelen::prelude::*;
/// 
/// let mut solver = FlatZincSolver::new();
/// // ... use solver
/// ```
pub mod prelude {
    //! Commonly used types and traits for working with FlatZinc.
    
    pub use crate::error::{FlatZincError, FlatZincResult};
    pub use crate::integration::FlatZincModel;
    pub use crate::output::{OutputFormatter, SearchType, SolveStatistics};
    pub use crate::solver::{FlatZincContext, FlatZincSolver, SolverOptions};
    
    // Re-export Selen's prelude, but don't inline the docs
    #[doc(no_inline)]
    pub use selen::prelude::*;
}

/// Parse FlatZinc tokens into AST and map to an existing Model.
///
/// This is an internal function used by Model::from_flatzinc_* methods.
pub(crate) fn parse_and_map(content: &str, model: &mut selen::prelude::Model) -> FlatZincResult<()> {
    // Step 1: Tokenize
    let tokens = tokenizer::tokenize(content)?;
    
    // Step 2: Parse into AST
    let ast = parser::parse(tokens)?;
    
    // Step 3: Map AST to the provided Model
    mapper::map_to_model_mut(ast, model)?;
    
    Ok(())
}