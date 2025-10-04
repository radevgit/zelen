//! Float constraint mappers
//!
//! Maps FlatZinc float constraints to Selen constraint model.
//! Note: Selen handles floats through discretization internally.

use crate::ast::*;
use crate::error::{FlatZincError, FlatZincResult};
use crate::mapper::MappingContext;
use selen::runtime_api::{VarIdExt, ModelExt};

impl<'a> MappingContext<'a> {
    /// Map float_eq constraint: x = y
    pub(in crate::mapper) fn map_float_eq(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "float_eq requires 2 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        self.model.new(x.eq(y));
        
        Ok(())
    }
    
    /// Map float_ne constraint: x ≠ y
    pub(in crate::mapper) fn map_float_ne(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "float_ne requires 2 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        self.model.new(x.ne(y));
        Ok(())
    }
    
    /// Map float_lt constraint: x < y
    pub(in crate::mapper) fn map_float_lt(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "float_lt requires 2 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        self.model.new(x.lt(y));
        Ok(())
    }
    
    /// Map float_le constraint: x ≤ y
    pub(in crate::mapper) fn map_float_le(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "float_le requires 2 arguments".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        self.model.new(x.le(y));
        Ok(())
    }
    
    /// Map float_lin_eq constraint: sum(coeffs[i] * vars[i]) = constant
    pub(in crate::mapper) fn map_float_lin_eq(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_lin_eq requires 3 arguments (coeffs, vars, constant)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Extract coefficients (array of floats)
        let coeffs = self.extract_float_array(&constraint.args[0])?;
        
        // Extract variables
        let vars = self.extract_var_array(&constraint.args[1])?;
        
        // Extract constant
        let constant = self.extract_float(&constraint.args[2])?;
        
        if coeffs.len() != vars.len() {
            return Err(FlatZincError::MapError {
                message: format!("float_lin_eq: coefficient count ({}) != variable count ({})", 
                    coeffs.len(), vars.len()),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Use native Selen float_lin_eq constraint
        self.model.float_lin_eq(&coeffs, &vars, constant);
        
        Ok(())
    }
    
    /// Map float_lin_le constraint: sum(coeffs[i] * vars[i]) ≤ constant
    pub(in crate::mapper) fn map_float_lin_le(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_lin_le requires 3 arguments (coeffs, vars, constant)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_float_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_float(&constraint.args[2])?;
        
        if coeffs.len() != vars.len() {
            return Err(FlatZincError::MapError {
                message: format!("float_lin_le: coefficient count ({}) != variable count ({})", 
                    coeffs.len(), vars.len()),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Use native Selen float_lin_le constraint
        self.model.float_lin_le(&coeffs, &vars, constant);
        
        Ok(())
    }
    
    /// Map float_lin_ne constraint: sum(coeffs[i] * vars[i]) ≠ constant
    pub(in crate::mapper) fn map_float_lin_ne(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_lin_ne requires 3 arguments (coeffs, vars, constant)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let coeffs = self.extract_float_array(&constraint.args[0])?;
        let vars = self.extract_var_array(&constraint.args[1])?;
        let constant = self.extract_float(&constraint.args[2])?;
        
        if coeffs.len() != vars.len() {
            return Err(FlatZincError::MapError {
                message: format!("float_lin_ne: coefficient count ({}) != variable count ({})", 
                    coeffs.len(), vars.len()),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Use native Selen float_lin_ne constraint
        self.model.float_lin_ne(&coeffs, &vars, constant);
        
        Ok(())
    }
    
    /// Map float_plus constraint: c = a + b
    pub(in crate::mapper) fn map_float_plus(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_plus requires 3 arguments (a, b, c)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let a = self.get_var_or_const(&constraint.args[0])?;
        let b = self.get_var_or_const(&constraint.args[1])?;
        let c = self.get_var_or_const(&constraint.args[2])?;
        
        // Create add constraint: c = a + b
        let result = self.model.add(a, b);
        self.model.new(c.eq(result));
        
        Ok(())
    }
    
    /// Map float_minus constraint: c = a - b
    pub(in crate::mapper) fn map_float_minus(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_minus requires 3 arguments (a, b, c)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let a = self.get_var_or_const(&constraint.args[0])?;
        let b = self.get_var_or_const(&constraint.args[1])?;
        let c = self.get_var_or_const(&constraint.args[2])?;
        
        // Create sub constraint: c = a - b
        let result = self.model.sub(a, b);
        self.model.new(c.eq(result));
        
        Ok(())
    }
    
    /// Map float_times constraint: c = a * b
    pub(in crate::mapper) fn map_float_times(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_times requires 3 arguments (a, b, c)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let a = self.get_var_or_const(&constraint.args[0])?;
        let b = self.get_var_or_const(&constraint.args[1])?;
        let c = self.get_var_or_const(&constraint.args[2])?;
        
        // Create mul constraint: c = a * b
        let result = self.model.mul(a, b);
        self.model.new(c.eq(result));
        
        Ok(())
    }
    
    /// Map float_div constraint: c = a / b
    pub(in crate::mapper) fn map_float_div(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_div requires 3 arguments (a, b, c)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let a = self.get_var_or_const(&constraint.args[0])?;
        let b = self.get_var_or_const(&constraint.args[1])?;
        let c = self.get_var_or_const(&constraint.args[2])?;
        
        // Create div constraint: c = a / b
        let result = self.model.div(a, b);
        self.model.new(c.eq(result));
        
        Ok(())
    }
    
    /// Map float_abs constraint: b = |a|
    pub(in crate::mapper) fn map_float_abs(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "float_abs requires 2 arguments (a, b)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let a = self.get_var_or_const(&constraint.args[0])?;
        let b = self.get_var_or_const(&constraint.args[1])?;
        
        // Create abs constraint: b = |a|
        let result = self.model.abs(a);
        self.model.new(b.eq(result));
        
        Ok(())
    }
    
    /// Map float_max constraint: c = max(a, b)
    pub(in crate::mapper) fn map_float_max(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_max requires 3 arguments (a, b, c)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let a = self.get_var_or_const(&constraint.args[0])?;
        let b = self.get_var_or_const(&constraint.args[1])?;
        let c = self.get_var_or_const(&constraint.args[2])?;
        
        // Create max constraint: c = max(a, b)
        let result = self.model.max(&[a, b]).map_err(|e| FlatZincError::MapError {
            message: format!("Failed to create max constraint: {:?}", e),
            line: Some(constraint.location.line),
            column: Some(constraint.location.column),
        })?;
        self.model.new(c.eq(result));
        
        Ok(())
    }
    
    /// Map float_min constraint: c = min(a, b)
    pub(in crate::mapper) fn map_float_min(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_min requires 3 arguments (a, b, c)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let a = self.get_var_or_const(&constraint.args[0])?;
        let b = self.get_var_or_const(&constraint.args[1])?;
        let c = self.get_var_or_const(&constraint.args[2])?;
        
        // Create min constraint: c = min(a, b)
        let result = self.model.min(&[a, b]).map_err(|e| FlatZincError::MapError {
            message: format!("Failed to create min constraint: {:?}", e),
            line: Some(constraint.location.line),
            column: Some(constraint.location.column),
        })?;
        self.model.new(c.eq(result));
        
        Ok(())
    }
    
    // Reified Float Comparison Constraints
    
    /// Map float_eq_reif constraint: r ⟺ (x = y)
    pub(in crate::mapper) fn map_float_eq_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_eq_reif requires 3 arguments (x, y, r)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let r = self.get_var_or_const(&constraint.args[2])?;
        
        self.model.float_eq_reif(x, y, r);
        Ok(())
    }
    
    /// Map float_ne_reif constraint: r ⟺ (x ≠ y)
    pub(in crate::mapper) fn map_float_ne_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_ne_reif requires 3 arguments (x, y, r)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let r = self.get_var_or_const(&constraint.args[2])?;
        
        self.model.float_ne_reif(x, y, r);
        Ok(())
    }
    
    /// Map float_lt_reif constraint: r ⟺ (x < y)
    pub(in crate::mapper) fn map_float_lt_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_lt_reif requires 3 arguments (x, y, r)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let r = self.get_var_or_const(&constraint.args[2])?;
        
        self.model.float_lt_reif(x, y, r);
        Ok(())
    }
    
    /// Map float_le_reif constraint: r ⟺ (x ≤ y)
    pub(in crate::mapper) fn map_float_le_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_le_reif requires 3 arguments (x, y, r)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let r = self.get_var_or_const(&constraint.args[2])?;
        
        self.model.float_le_reif(x, y, r);
        Ok(())
    }
    
    /// Map float_gt_reif constraint: r ⟺ (x > y)
    pub(in crate::mapper) fn map_float_gt_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_gt_reif requires 3 arguments (x, y, r)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let r = self.get_var_or_const(&constraint.args[2])?;
        
        self.model.float_gt_reif(x, y, r);
        Ok(())
    }
    
    /// Map float_ge_reif constraint: r ⟺ (x ≥ y)
    pub(in crate::mapper) fn map_float_ge_reif(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "float_ge_reif requires 3 arguments (x, y, r)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        let r = self.get_var_or_const(&constraint.args[2])?;
        
        self.model.float_ge_reif(x, y, r);
        Ok(())
    }
    
    // Float/Int Conversion Constraints
    
    /// Map int2float constraint: y = float(x)
    pub(in crate::mapper) fn map_int2float(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "int2float requires 2 arguments (x, y)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        
        self.model.int2float(x, y);
        Ok(())
    }
    
    /// Map float2int constraint: y = floor(x)
    pub(in crate::mapper) fn map_float2int(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 2 {
            return Err(FlatZincError::MapError {
                message: "float2int requires 2 arguments (x, y)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        let x = self.get_var_or_const(&constraint.args[0])?;
        let y = self.get_var_or_const(&constraint.args[1])?;
        
        // float2int is typically floor conversion
        self.model.float2int_floor(x, y);
        Ok(())
    }
}
