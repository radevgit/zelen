//! Array constraint mappers
//!
//! Maps FlatZinc array constraints (array_int_minimum, array_int_maximum)
//! to Selen constraint model.

use crate::ast::*;
use crate::error::{FlatZincError, FlatZincResult};
use crate::mapper::MappingContext;
use selen::runtime_api::{VarIdExt, ModelExt};

impl<'a> MappingContext<'a> {
    /// Map array_int_minimum: min = minimum(array)
    pub(in crate::mapper) fn map_array_int_minimum(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "array_int_minimum requires 2 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let min_var = self.get_var_or_const(&constraint.args[0])?;
        let arr_vars = self.extract_var_array(&constraint.args[1])?;
        let min_result = self.model.array_int_minimum(&arr_vars).map_err(|e| FlatZincError::MapError {
            message: format!("Failed to create array_int_minimum: {}", e),
            line: Some(constraint.location.line),
            column: Some(constraint.location.column),
        })?;
        self.model.new(min_var.eq(min_result));
        Ok(())
    }
    
    /// Map array_int_maximum: max = maximum(array)
    pub(in crate::mapper) fn map_array_int_maximum(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "array_int_maximum requires 2 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let max_var = self.get_var_or_const(&constraint.args[0])?;
        let arr_vars = self.extract_var_array(&constraint.args[1])?;
        let max_result = self.model.array_int_maximum(&arr_vars).map_err(|e| FlatZincError::MapError {
            message: format!("Failed to create array_int_maximum: {}", e),
            line: Some(constraint.location.line),
            column: Some(constraint.location.column),
        })?;
        self.model.new(max_var.eq(max_result));
        Ok(())
    }
    
    /// Map array_float_minimum: min = minimum(array)
    pub(in crate::mapper) fn map_array_float_minimum(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "array_float_minimum requires 2 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let min_var = self.get_var_or_const(&constraint.args[0])?;
        let arr_vars = self.extract_var_array(&constraint.args[1])?;
        let min_result = self.model.array_float_minimum(&arr_vars).map_err(|e| FlatZincError::MapError {
            message: format!("Failed to create float minimum: {}", e),
            line: Some(constraint.location.line),
            column: Some(constraint.location.column),
        })?;
        self.model.new(min_var.eq(min_result));
        Ok(())
    }
    
    /// Map array_float_maximum: max = maximum(array)
    pub(in crate::mapper) fn map_array_float_maximum(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "array_float_maximum requires 2 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let max_var = self.get_var_or_const(&constraint.args[0])?;
        let arr_vars = self.extract_var_array(&constraint.args[1])?;
        let max_result = self.model.array_float_maximum(&arr_vars).map_err(|e| FlatZincError::MapError {
            message: format!("Failed to create float maximum: {}", e),
            line: Some(constraint.location.line),
            column: Some(constraint.location.column),
        })?;
        self.model.new(max_var.eq(max_result));
        Ok(())
    }
}
