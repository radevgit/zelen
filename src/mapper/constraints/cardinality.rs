//! At least, at most, exactly constraint mappers
//!
//! Maps FlatZinc counting constraints to Selen's native cardinality constraints.

use crate::ast::*;
use crate::error::{FlatZincError, FlatZincResult};
use crate::mapper::MappingContext;

impl<'a> MappingContext<'a> {
    /// Map at_least: at least n variables in array equal value
    /// FlatZinc signature: fzn_at_least_int(n, x, v)
    pub(in crate::mapper) fn map_at_least_int(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "at_least_int requires 3 arguments (n, x, v)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let n = self.extract_int(&constraint.args[0])?;
        let arr_vars = self.extract_var_array(&constraint.args[1])?;
        let value = self.extract_int(&constraint.args[2])?;
        
        // Use Selen's at_least constraint
        self.model.at_least(&arr_vars, value, n);
        Ok(())
    }
    
    /// Map at_most: at most n variables in array equal value
    /// FlatZinc signature: fzn_at_most_int(n, x, v)
    pub(in crate::mapper) fn map_at_most_int(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "at_most_int requires 3 arguments (n, x, v)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let n = self.extract_int(&constraint.args[0])?;
        let arr_vars = self.extract_var_array(&constraint.args[1])?;
        let value = self.extract_int(&constraint.args[2])?;
        
        // Use Selen's at_most constraint
        self.model.at_most(&arr_vars, value, n);
        Ok(())
    }
    
    /// Map exactly: exactly n variables in array equal value
    /// FlatZinc signature: fzn_exactly_int(n, x, v)
    pub(in crate::mapper) fn map_exactly_int(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "exactly_int requires 3 arguments (n, x, v)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let n = self.extract_int(&constraint.args[0])?;
        let arr_vars = self.extract_var_array(&constraint.args[1])?;
        let value = self.extract_int(&constraint.args[2])?;
        
        // Use Selen's exactly constraint
        self.model.exactly(&arr_vars, value, n);
        Ok(())
    }
}
