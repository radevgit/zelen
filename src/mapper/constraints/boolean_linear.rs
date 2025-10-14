//! Boolean linear constraint mappers
//!
//! Maps FlatZinc boolean linear constraints (bool_lin_eq, bool_lin_le, bool_lin_ne)
//! to Selen constraint model. These handle weighted sums of boolean variables.

use crate::ast::*;
use crate::error::{FlatZincError, FlatZincResult};
use crate::mapper::MappingContext;
use selen::runtime_api::ModelExt;

impl<'a> MappingContext<'a> {
    /// Map bool_lin_eq: Σ(coeffs[i] * vars[i]) = constant
    pub(in crate::mapper) fn map_bool_lin_eq(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "bool_lin_eq requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        
        self.model.bool_lin_eq(&coeffs, &vars, constant);
        Ok(())
    }
    
    /// Map bool_lin_le: Σ(coeffs[i] * vars[i]) ≤ constant
    pub(in crate::mapper) fn map_bool_lin_le(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "bool_lin_le requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        
        self.model.bool_lin_le(&coeffs, &vars, constant);
        Ok(())
    }
    
    /// Map bool_lin_ne: Σ(coeffs[i] * vars[i]) ≠ constant
    pub(in crate::mapper) fn map_bool_lin_ne(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "bool_lin_ne requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        
        self.model.bool_lin_ne(&coeffs, &vars, constant);
        Ok(())
    }
    
    /// Map bool_lin_eq_reif: b ⇔ (Σ(coeffs[i] * vars[i]) = constant)
    pub(in crate::mapper) fn map_bool_lin_eq_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 4 {
            return Err(FlatZincError::MapError {
                message: "bool_lin_eq_reif requires 4 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        let b = self.get_var_or_const(&constraint.args[3])?;
        
        self.model.bool_lin_eq_reif(&coeffs, &vars, constant, b);
        Ok(())
    }
    
    /// Map bool_lin_le_reif: b ⇔ (Σ(coeffs[i] * vars[i]) ≤ constant)
    pub(in crate::mapper) fn map_bool_lin_le_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 4 {
            return Err(FlatZincError::MapError {
                message: "bool_lin_le_reif requires 4 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        let b = self.get_var_or_const(&constraint.args[3])?;
        
        self.model.bool_lin_le_reif(&coeffs, &vars, constant, b);
        Ok(())
    }
    
    /// Map bool_lin_ne_reif: b ⇔ (Σ(coeffs[i] * vars[i]) ≠ constant)
    pub(in crate::mapper) fn map_bool_lin_ne_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 4 {
            return Err(FlatZincError::MapError {
                message: "bool_lin_ne_reif requires 4 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        let b = self.get_var_or_const(&constraint.args[3])?;
        
        self.model.bool_lin_ne_reif(&coeffs, &vars, constant, b);
        Ok(())
    }
}
