//! # Zelen - FlatZinc Frontend for Selen
//! 
//! Zelen provides FlatZinc parsing and integration with the Selen constraint solver.
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use zelen::prelude::*;
//! 
//! let mut model = Model::default();
//! model.from_flatzinc_file("problem.fzn")?;
//! let solution = model.solve()?;
//! ```

pub mod ast;
pub mod error;
pub mod tokenizer;
pub mod parser;
pub mod mapper;
pub mod output;
pub mod solver;
pub mod integration;

pub use error::{FlatZincError, FlatZincResult};

// Re-export selen for convenience
pub use selen;

/// Prelude module for common imports
pub mod prelude {
    pub use crate::error::{FlatZincError, FlatZincResult};
    pub use crate::integration::*;
    pub use crate::output::{OutputFormatter, SearchType, SolveStatistics};
    pub use crate::solver::{FlatZincContext, FlatZincSolver, SolverOptions};
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