//! Linear constraint mappers
//!
//! Maps FlatZinc linear constraints (int_lin_eq, int_lin_le, int_lin_ne, float_lin_eq, float_lin_le, float_lin_ne)
//! to Selen constraint model using the new generic lin_eq/lin_le/lin_ne API.

use crate::ast::*;
use crate::error::{FlatZincError, FlatZincResult};
use crate::mapper::MappingContext;

impl<'a> MappingContext<'a> {
    /// Map int_lin_eq: Σ(coeffs[i] * vars[i]) = constant
    pub(in crate::mapper) fn map_int_lin_eq(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "int_lin_eq requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let var_ids = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        
        // Use the new generic lin_eq API
        self.model.lin_eq(&coeffs, &var_ids, constant);
        Ok(())
    }
    
    /// Map int_lin_le: Σ(coeffs[i] * vars[i]) ≤ constant
    pub(in crate::mapper) fn map_int_lin_le(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "int_lin_le requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let var_ids = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        
        // Use the new generic lin_le API
        self.model.lin_le(&coeffs, &var_ids, constant);
        Ok(())
    }
    
    /// Map int_lin_ne: Σ(coeffs[i] * vars[i]) ≠ constant
    pub(in crate::mapper) fn map_int_lin_ne(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "int_lin_ne requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let var_ids = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        
        // Use the new generic lin_ne API
        self.model.lin_ne(&coeffs, &var_ids, constant);
        Ok(())
    }
}
