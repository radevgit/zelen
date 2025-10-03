//! Element constraint mappers
//!
//! Maps FlatZinc element constraints (array access constraints) to Selen constraint model.
//! Element constraints express: array[index] = value

use crate::ast::*;
use crate::error::{FlatZincError, FlatZincResult};
use crate::mapper::MappingContext;
use selen::runtime_api::ModelExt;
use selen::variables::VarId;

impl<'a> MappingContext<'a> {
    /// Map array_var_int_element: array[index] = value
    /// FlatZinc signature: array_var_int_element(index, array, value)
    /// Note: FlatZinc uses 1-based indexing, Selen uses 0-based
    pub(in crate::mapper) fn map_array_var_int_element(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "array_var_int_element requires 3 arguments (index, array, value)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Get index variable (1-based in FlatZinc)
        // Supports: variables, array access (x[i]), integer literals
        let index_1based = self.get_var_or_const(&constraint.args[0])?;
        
        // Convert to 0-based index for Selen
        // Create: index_0based = index_1based - 1
        let index_0based = self.model.sub(index_1based, selen::variables::Val::ValI(1));
        
        // Get array
        let array = self.extract_var_array(&constraint.args[1])?;
        
        // Get value (can be variable, array access, or constant)
        // Supports: variables, array access (y[j]), integer literals
        let value = self.get_var_or_const(&constraint.args[2])?;
        
        // Apply element constraint: array[index_0based] = value
        self.model.elem(&array, index_0based, value);
        Ok(())
    }
    
    /// Map array_int_element: array[index] = value (with constant array)
    /// FlatZinc signature: array_int_element(index, array, value)
    /// Note: FlatZinc uses 1-based indexing, Selen uses 0-based
    pub(in crate::mapper) fn map_array_int_element(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "array_int_element requires 3 arguments (index, array, value)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Get index variable (1-based in FlatZinc)
        // Supports: variables, array access (x[i]), integer literals
        let index_1based = self.get_var_or_const(&constraint.args[0])?;
        
        // Convert to 0-based index for Selen
        let index_0based = self.model.sub(index_1based, selen::variables::Val::ValI(1));
        
        // Get array of constants and convert to fixed variables
        let const_array = self.extract_int_array(&constraint.args[1])?;
        let array: Vec<VarId> = const_array.iter()
            .map(|&val| self.model.int(val, val))
            .collect();
        
        // Get value (can be variable, array access, or constant)
        // Supports: variables, array access (y[j]), integer literals
        let value = self.get_var_or_const(&constraint.args[2])?;
        
        // Apply element constraint: array[index_0based] = value
        self.model.elem(&array, index_0based, value);
        Ok(())
    }
    
    /// Map array_var_bool_element: array[index] = value (boolean version)
    /// FlatZinc signature: array_var_bool_element(index, array, value)
    /// Note: FlatZinc uses 1-based indexing, Selen uses 0-based
    pub(in crate::mapper) fn map_array_var_bool_element(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "array_var_bool_element requires 3 arguments (index, array, value)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Get index variable (1-based in FlatZinc)
        // Supports: variables, array access (x[i]), integer literals
        let index_1based = self.get_var_or_const(&constraint.args[0])?;
        
        // Convert to 0-based index for Selen
        let index_0based = self.model.sub(index_1based, selen::variables::Val::ValI(1));
        
        // Get array (booleans are represented as 0/1 variables)
        let array = self.extract_var_array(&constraint.args[1])?;
        
        // Get value (can be variable, array access, or constant)
        // Supports: variables, array access (y[j]), boolean literals
        let value = self.get_var_or_const(&constraint.args[2])?;
        
        // Apply element constraint: array[index_0based] = value
        self.model.elem(&array, index_0based, value);
        Ok(())
    }
    
    /// Map array_bool_element: array[index] = value (with constant boolean array)
    /// FlatZinc signature: array_bool_element(index, array, value)
    /// Note: FlatZinc uses 1-based indexing, Selen uses 0-based
    pub(in crate::mapper) fn map_array_bool_element(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "array_bool_element requires 3 arguments (index, array, value)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Get index variable (1-based in FlatZinc)
        // Supports: variables, array access (x[i]), integer literals
        let index_1based = self.get_var_or_const(&constraint.args[0])?;
        
        // Convert to 0-based index for Selen
        let index_0based = self.model.sub(index_1based, selen::variables::Val::ValI(1));
        
        // Get array of boolean constants and convert to 0/1 fixed variables
        let array: Vec<VarId> = if let Expr::ArrayLit(elements) = &constraint.args[1] {
            elements.iter()
                .map(|elem| {
                    if let Expr::BoolLit(b) = elem {
                        let val = if *b { 1 } else { 0 };
                        Ok(self.model.int(val, val))
                    } else {
                        Err(FlatZincError::MapError {
                            message: "Expected boolean literal in array_bool_element array".to_string(),
                            line: Some(constraint.location.line),
                            column: Some(constraint.location.column),
                        })
                    }
                })
                .collect::<FlatZincResult<Vec<VarId>>>()?
        } else {
            return Err(FlatZincError::MapError {
                message: "Expected array literal in array_bool_element".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        };
        
        // Get value (can be variable, array access, or constant)
        // Supports: variables, array access (y[j]), boolean literals
        let value = self.get_var_or_const(&constraint.args[2])?;
        
        // Apply element constraint: array[index_0based] = value
        self.model.elem(&array, index_0based, value);
        Ok(())
    }
}
