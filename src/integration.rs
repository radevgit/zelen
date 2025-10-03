//! FlatZinc integration methods for Model

use selen::prelude::Model;
use crate::{parse_and_map, FlatZincResult, FlatZincError};
use crate::{tokenizer, parser, mapper};
use crate::solver::FlatZincContext;
use std::fs;
use std::path::Path;

/// Trait that extends `Model` with FlatZinc integration methods
pub trait FlatZincModel {
    /// Import a FlatZinc file into this model.
    ///
    /// This allows you to configure the model (memory limits, timeout, etc.) before
    /// importing the FlatZinc problem.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the `.fzn` file
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or a `FlatZincError` if parsing or mapping fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use zelen::prelude::*;
    ///
    /// // Configure model first
    /// let mut model = Model::default()
    ///     .with_timeout_seconds(30);
    ///
    /// model.from_flatzinc_file("problem.fzn")?;
    /// let solution = model.solve()?;
    /// ```
    fn from_flatzinc_file<P: AsRef<Path>>(&mut self, path: P) -> FlatZincResult<()>;

    /// Import FlatZinc source code into this model.
    ///
    /// This allows you to configure the model (memory limits, timeout, etc.) before
    /// importing the FlatZinc problem.
    ///
    /// # Arguments
    ///
    /// * `content` - FlatZinc source code as a string
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or a `FlatZincError` if parsing or mapping fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use zelen::prelude::*;
    ///
    /// let fzn = r#"
    ///     var 1..10: x;
    ///     var 1..10: y;
    ///     constraint int_eq(x, y);
    ///     solve satisfy;
    /// "#;
    ///
    /// let mut model = Model::default();
    /// model.from_flatzinc_str(fzn)?;
    /// let solution = model.solve()?;
    /// ```
    fn from_flatzinc_str(&mut self, content: &str) -> FlatZincResult<()>;

    /// Parse FlatZinc and return the context with variable mappings.
    ///
    /// This method parses the FlatZinc, maps it to the model, and returns
    /// the context needed to format solutions. The user then calls solve()
    /// separately and uses the context to format the output.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use zelen::prelude::*;
    ///
    /// let fzn = r#"
    ///     var 1..10: x;
    ///     constraint int_eq(x, 5);
    ///     solve satisfy;
    /// "#;
    ///
    /// let mut model = Model::default();
    /// let context = model.load_flatzinc_str(fzn)?;
    /// 
    /// match model.solve() {
    ///     Ok(solution) => context.print_solution(&solution),
    ///     Err(_) => context.print_unsatisfiable(),
    /// }
    /// ```
    fn load_flatzinc_str(&mut self, content: &str) -> FlatZincResult<FlatZincContext>;

    /// Parse a FlatZinc file and return the context with variable mappings.
    fn load_flatzinc_file<P: AsRef<Path>>(&mut self, path: P) -> FlatZincResult<FlatZincContext>;
}

impl FlatZincModel for Model {
    fn from_flatzinc_file<P: AsRef<Path>>(&mut self, path: P) -> FlatZincResult<()> {
        let content = fs::read_to_string(path)
            .map_err(|e| FlatZincError::IoError(e.to_string()))?;
        // Call parse_and_map directly to avoid calling selen's from_flatzinc_str method
        parse_and_map(&content, self)
    }

    fn from_flatzinc_str(&mut self, content: &str) -> FlatZincResult<()> {
        parse_and_map(content, self)
    }

    fn load_flatzinc_str(&mut self, content: &str) -> FlatZincResult<FlatZincContext> {
        // Parse to AST
        let tokens = tokenizer::tokenize(content)?;
        let ast = parser::parse(tokens)?;

        // Map to model with context
        mapper::map_to_model_with_context(ast, self)
    }

    fn load_flatzinc_file<P: AsRef<Path>>(&mut self, path: P) -> FlatZincResult<FlatZincContext> {
        let content = fs::read_to_string(path)
            .map_err(|e| FlatZincError::IoError(e.to_string()))?;
        self.load_flatzinc_str(&content)
    }
}
