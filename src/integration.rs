//! FlatZinc integration methods for Model

use selen::prelude::Model;
use crate::{parse_and_map, FlatZincResult, FlatZincError};
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
}
