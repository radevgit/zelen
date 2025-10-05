//! Reified constraint mappers
//!
//! Maps FlatZinc reified constraints (*_reif) to Selen constraint model.
//! Reified constraints have the form: b ⇔ (constraint)

use crate::ast::*;
use crate::error::{FlatZincError, FlatZincResult};
use crate::mapper::MappingContext;
use selen::runtime_api::ModelExt;

impl<'a> MappingContext<'a> {
    /// Map int_eq_reif: b ⇔ (x = y)
    pub(in crate::mapper) fn map_int_eq_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "int_eq_reif requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let b = self.get_var_or_const(&constraint.args[2])?;
        self.model.int_eq_reif(x, y, b);
        Ok(())
    }
    
    pub(in crate::mapper) fn map_int_ne_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "int_ne_reif requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let b = self.get_var_or_const(&constraint.args[2])?;
        self.model.int_ne_reif(x, y, b);
        Ok(())
    }
    
    pub(in crate::mapper) fn map_int_lt_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "int_lt_reif requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let b = self.get_var_or_const(&constraint.args[2])?;
        self.model.int_lt_reif(x, y, b);
        Ok(())
    }
    
    pub(in crate::mapper) fn map_int_le_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "int_le_reif requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let b = self.get_var_or_const(&constraint.args[2])?;
        self.model.int_le_reif(x, y, b);
        Ok(())
    }
    
    pub(in crate::mapper) fn map_int_gt_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "int_gt_reif requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let b = self.get_var_or_const(&constraint.args[2])?;
        self.model.int_gt_reif(x, y, b);
        Ok(())
    }
    
    pub(in crate::mapper) fn map_int_ge_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "int_ge_reif requires 3 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let b = self.get_var_or_const(&constraint.args[2])?;
        self.model.int_ge_reif(x, y, b);
        Ok(())
    }
    
    // Linear reified constraints
    
    /// Map int_lin_eq_reif: b ⇔ (a1*x1 + a2*x2 + ... + an*xn = c)
    pub(in crate::mapper) fn map_int_lin_eq_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 4 {
            return Err(FlatZincError::MapError {
                message: "int_lin_eq_reif requires 4 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        let b = self.get_var_or_const(&constraint.args[3])?;
        
        self.model.int_lin_eq_reif(&coeffs, &vars, constant, b);
        Ok(())
    }
    
    /// Map int_lin_ne_reif: b ⇔ (a1*x1 + a2*x2 + ... + an*xn != c)
    pub(in crate::mapper) fn map_int_lin_ne_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 4 {
            return Err(FlatZincError::MapError {
                message: "int_lin_ne_reif requires 4 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        let b = self.get_var_or_const(&constraint.args[3])?;
        
        self.model.int_lin_ne_reif(&coeffs, &vars, constant, b);
        Ok(())
    }
    
    /// Map int_lin_le_reif: b ⇔ (a1*x1 + a2*x2 + ... + an*xn <= c)
    pub(in crate::mapper) fn map_int_lin_le_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 4 {
            return Err(FlatZincError::MapError {
                message: "int_lin_le_reif requires 4 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_int_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_int(&constraint.args[2])?;
        let b = self.get_var_or_const(&constraint.args[3])?;
        
        self.model.int_lin_le_reif(&coeffs, &vars, constant, b);
        Ok(())
    }

    /// Map float_lin_eq_reif: b ⇔ (a1*x1 + a2*x2 + ... + an*xn = c)
    pub(in crate::mapper) fn map_float_lin_eq_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 4 {
            return Err(FlatZincError::MapError {
                message: "float_lin_eq_reif requires 4 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_float_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_float(&constraint.args[2])?;
        let b = self.get_var_or_const(&constraint.args[3])?;
        
        self.model.float_lin_eq_reif(&coeffs, &vars, constant, b);
        Ok(())
    }
    
    /// Map float_lin_ne_reif: b ⇔ (a1*x1 + a2*x2 + ... + an*xn != c)
    pub(in crate::mapper) fn map_float_lin_ne_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 4 {
            return Err(FlatZincError::MapError {
                message: "float_lin_ne_reif requires 4 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_float_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_float(&constraint.args[2])?;
        let b = self.get_var_or_const(&constraint.args[3])?;
        
        self.model.float_lin_ne_reif(&coeffs, &vars, constant, b);
        Ok(())
    }
    
    /// Map float_lin_le_reif: b ⇔ (a1*x1 + a2*x2 + ... + an*xn <= c)
    pub(in crate::mapper) fn map_float_lin_le_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 4 {
            return Err(FlatZincError::MapError {
                message: "float_lin_le_reif requires 4 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_float_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_float(&constraint.args[2])?;
        let b = self.get_var_or_const(&constraint.args[3])?;
        
        self.model.float_lin_le_reif(&coeffs, &vars, constant, b);
        Ok(())
    }
}
