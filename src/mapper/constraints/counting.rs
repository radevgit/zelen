//! Counting constraint mappers
//!
//! Maps FlatZinc counting constraints (count_eq) to Selen constraint model.

use crate::ast::*;
use crate::error::{FlatZincError, FlatZincResult};
use crate::mapper::MappingContext;
use selen::runtime_api::ModelExt;

impl<'a> MappingContext<'a> {
    /// Map count_eq: count = |{i : array[i] = value}|
    /// Also used for count/3 which has the same signature
    pub(in crate::mapper) fn map_count_eq(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "count_eq requires 3 arguments (array, value, count)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let arr_vars = self.extract_var_array(&constraint.args[0])?;
        let value = self.extract_int(&constraint.args[1])?;
        let count_var = self.get_var_or_const(&constraint.args[2])?;
        
        // Use Selen's count constraint: count(&vars, value, count_var)
        // This constrains: count_var = |{i : vars[i] = value}|
        self.model.count(&arr_vars, selen::variables::Val::ValI(value), count_var);
        Ok(())
    }
}
