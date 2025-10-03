//! Helper functions for extracting values from FlatZinc AST expressions

use crate::ast::*;
use crate::error::{FlatZincError, FlatZincResult};
use crate::mapper::MappingContext;
use selen::variables::VarId;

impl<'a> MappingContext<'a> {
    /// Evaluate array access expression: array[index]
    /// 
    /// Resolves expressions like `x[1]` by:
    /// 1. Looking up the array variable `x` in array_map
    /// 2. Converting the FlatZinc 1-based index to 0-based
    /// 3. Returning the VarId at that position
    pub(super) fn evaluate_array_access(
        &self,
        array_expr: &Expr,
        index_expr: &Expr,
    ) -> FlatZincResult<VarId> {
        // Get the array name
        let array_name = match array_expr {
            Expr::Ident(name) => name,
            _ => {
                return Err(FlatZincError::MapError {
                    message: format!("Array access requires identifier, got: {:?}", array_expr),
                    line: None,
                    column: None,
                });
            }
        };

        // Get the array
        let array = self.array_map.get(array_name).ok_or_else(|| {
            FlatZincError::MapError {
                message: format!("Unknown array: {}", array_name),
                line: None,
                column: None,
            }
        })?;

        // Get the index (1-based in FlatZinc)
        let index_1based = match index_expr {
            Expr::IntLit(val) => *val as usize,
            _ => {
                return Err(FlatZincError::MapError {
                    message: format!("Array index must be integer literal, got: {:?}", index_expr),
                    line: None,
                    column: None,
                });
            }
        };

        // Convert to 0-based and bounds check
        if index_1based < 1 {
            return Err(FlatZincError::MapError {
                message: format!("Array index must be >= 1, got: {}", index_1based),
                line: None,
                column: None,
            });
        }
        let index_0based = index_1based - 1;

        if index_0based >= array.len() {
            return Err(FlatZincError::MapError {
                message: format!(
                    "Array index {} out of bounds for array '{}' of length {}",
                    index_1based,
                    array_name,
                    array.len()
                ),
                line: None,
                column: None,
            });
        }

        Ok(array[index_0based])
    }

    /// Get a variable by identifier or array access
    pub(super) fn get_var(&self, expr: &Expr) -> FlatZincResult<VarId> {
        match expr {
            Expr::Ident(name) => {
                self.var_map.get(name).copied().ok_or_else(|| {
                    FlatZincError::MapError {
                        message: format!("Unknown variable: {}", name),
                        line: None,
                        column: None,
                    }
                })
            }
            Expr::ArrayAccess { array, index } => {
                // Handle array access like x[1]
                self.evaluate_array_access(array, index)
            }
            _ => Err(FlatZincError::MapError {
                message: format!("Expected variable identifier or array access, got: {:?}", expr),
                line: None,
                column: None,
            }),
        }
    }
    
    /// Get a variable or convert a constant to a fixed variable
    /// Handles: variables, array access, integer literals, boolean literals
    pub(super) fn get_var_or_const(&mut self, expr: &Expr) -> FlatZincResult<VarId> {
        match expr {
            Expr::Ident(name) => {
                self.var_map.get(name).copied().ok_or_else(|| {
                    FlatZincError::MapError {
                        message: format!("Unknown variable: {}", name),
                        line: None,
                        column: None,
                    }
                })
            }
            Expr::ArrayAccess { array, index } => {
                // Handle array access like x[1]
                self.evaluate_array_access(array, index)
            }
            Expr::IntLit(val) => {
                // Convert constant to fixed variable
                Ok(self.model.int(*val as i32, *val as i32))
            }
            Expr::BoolLit(b) => {
                // Convert boolean to 0/1 fixed variable
                let val = if *b { 1 } else { 0 };
                Ok(self.model.int(val, val))
            }
            _ => Err(FlatZincError::MapError {
                message: format!("Unsupported expression type: {:?}", expr),
                line: None,
                column: None,
            }),
        }
    }
    
    /// Extract an integer value from an expression
    pub(super) fn extract_int(&self, expr: &Expr) -> FlatZincResult<i32> {
        match expr {
            Expr::IntLit(val) => Ok(*val as i32),
            Expr::Ident(name) => {
                // Could be a parameter - for now, just error
                Err(FlatZincError::MapError {
                    message: format!("Expected integer literal, got identifier: {}", name),
                    line: None,
                    column: None,
                })
            }
            _ => Err(FlatZincError::MapError {
                message: "Expected integer literal".to_string(),
                line: None,
                column: None,
            }),
        }
    }
    
    /// Extract an array of integers from an expression
    /// 
    /// Handles:
    /// - Inline array literals: [1, 2, 3]
    /// - Parameter array identifiers: col_left (references previously declared parameter array)
    pub(super) fn extract_int_array(&self, expr: &Expr) -> FlatZincResult<Vec<i32>> {
        match expr {
            Expr::ArrayLit(elements) => {
                elements.iter().map(|e| self.extract_int(e)).collect()
            }
            Expr::Ident(name) => {
                // Look up parameter array by name
                self.param_int_arrays.get(name)
                    .cloned()
                    .ok_or_else(|| FlatZincError::MapError {
                        message: format!("Parameter array '{}' not found (expected array of integers)", name),
                        line: None,
                        column: None,
                    })
            }
            _ => Err(FlatZincError::MapError {
                message: "Expected array of integers or array identifier".to_string(),
                line: None,
                column: None,
            }),
        }
    }
    
    /// Extract a boolean value from an expression
    pub(super) fn extract_bool(&self, expr: &Expr) -> FlatZincResult<bool> {
        match expr {
            Expr::BoolLit(val) => Ok(*val),
            Expr::IntLit(val) => Ok(*val != 0), // Treat 0 as false, non-zero as true
            Expr::Ident(name) => {
                // Could be a parameter - for now, just error
                Err(FlatZincError::MapError {
                    message: format!("Expected boolean literal, got identifier: {}", name),
                    line: None,
                    column: None,
                })
            }
            _ => Err(FlatZincError::MapError {
                message: "Expected boolean literal".to_string(),
                line: None,
                column: None,
            }),
        }
    }
    
    /// Extract an array of booleans from an expression
    /// 
    /// Handles:
    /// - Inline array literals: [true, false, true]
    /// - Parameter array identifiers: flags (references previously declared parameter array)
    pub(super) fn extract_bool_array(&self, expr: &Expr) -> FlatZincResult<Vec<bool>> {
        match expr {
            Expr::ArrayLit(elements) => {
                elements.iter().map(|e| self.extract_bool(e)).collect()
            }
            Expr::Ident(name) => {
                // Look up parameter array by name
                self.param_bool_arrays.get(name)
                    .cloned()
                    .ok_or_else(|| FlatZincError::MapError {
                        message: format!("Parameter array '{}' not found (expected array of booleans)", name),
                        line: None,
                        column: None,
                    })
            }
            _ => Err(FlatZincError::MapError {
                message: "Expected array of booleans or array identifier".to_string(),
                line: None,
                column: None,
            }),
        }
    }
    
    /// Extract an array of variables from an expression
    /// 
    /// Handles:
    /// - Array literals like `[x, y, z]` (may contain variables, array access, or integer constants)
    /// - Array identifiers that reference previously declared arrays
    /// - Single variable identifiers (treated as single-element array)
    pub(super) fn extract_var_array(&mut self, expr: &Expr) -> FlatZincResult<Vec<VarId>> {
        match expr {
            Expr::ArrayLit(elements) => {
                // Handle array literals that may contain variables, array access, or integer constants
                let mut var_ids = Vec::new();
                for elem in elements {
                    match elem {
                        Expr::Ident(name) => {
                            // Variable reference
                            let var_id = self.var_map.get(name).copied().ok_or_else(|| {
                                FlatZincError::MapError {
                                    message: format!("Unknown variable: {}", name),
                                    line: None,
                                    column: None,
                                }
                            })?;
                            var_ids.push(var_id);
                        }
                        Expr::IntLit(val) => {
                            // Constant integer - create a fixed variable
                            let const_var = self.model.int(*val as i32, *val as i32);
                            var_ids.push(const_var);
                        }
                        Expr::BoolLit(b) => {
                            // Constant boolean - create a fixed variable (0 or 1)
                            let val = if *b { 1 } else { 0 };
                            let const_var = self.model.int(val, val);
                            var_ids.push(const_var);
                        }
                        Expr::ArrayAccess { array, index } => {
                            // Array element access like x[1]
                            let var_id = self.evaluate_array_access(array, index)?;
                            var_ids.push(var_id);
                        }
                        _ => {
                            return Err(FlatZincError::MapError {
                                message: format!("Unsupported array element: {:?}", elem),
                                line: None,
                                column: None,
                            });
                        }
                    }
                }
                Ok(var_ids)
            }
            Expr::Ident(name) => {
                // First check if it's an array variable
                if let Some(arr) = self.array_map.get(name) {
                    return Ok(arr.clone());
                }
                
                // Check if it's a parameter int array - create constant VarIds
                if let Some(int_values) = self.param_int_arrays.get(name) {
                    let var_ids: Vec<VarId> = int_values.iter()
                        .map(|&val| self.model.int(val, val))
                        .collect();
                    return Ok(var_ids);
                }
                
                // Check if it's a parameter bool array - create constant VarIds
                if let Some(bool_values) = self.param_bool_arrays.get(name) {
                    let var_ids: Vec<VarId> = bool_values.iter()
                        .map(|&b| {
                            let val = if b { 1 } else { 0 };
                            self.model.int(val, val)
                        })
                        .collect();
                    return Ok(var_ids);
                }
                
                // Otherwise treat as single variable
                Ok(vec![self.var_map.get(name).copied().ok_or_else(|| {
                    FlatZincError::MapError {
                        message: format!("Unknown variable or array: {}", name),
                        line: None,
                        column: None,
                    }
                })?])
            }
            _ => Err(FlatZincError::MapError {
                message: "Expected array of variables".to_string(),
                line: None,
                column: None,
            }),
        }
    }
}
