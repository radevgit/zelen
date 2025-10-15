//! Global cardinality constraint mappers
//!
//! Maps FlatZinc global cardinality constraint to Selen constraint model.

use crate::ast::*;
use crate::error::{FlatZincError, FlatZincResult};
use crate::mapper::MappingContext;
use selen::runtime_api::ModelExt;

impl<'a> MappingContext<'a> {
    /// Map global_cardinality: For each value[i], count occurrences in vars array
    /// FlatZinc signature: global_cardinality(vars, values, counts)
    /// 
    /// Where:
    /// - vars: array of variables to count in
    /// - values: array of values to count (must be constants)
    /// - counts: array of count variables (one per value)
    /// 
    /// Constraint: For each i, counts[i] = |{j : vars[j] = values[i]}|
    pub(in crate::mapper) fn map_global_cardinality(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 3 {
            return Err(FlatZincError::MapError {
                message: "global_cardinality requires 3 arguments (vars, values, counts)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Extract the variables array
        let vars = self.extract_var_array(&constraint.args[0])?;
        
        // Extract the values array (must be constants)
        let values = self.extract_int_array(&constraint.args[1])?;
        
        // Extract the counts array (variables or constants)
        let counts = self.extract_var_array(&constraint.args[2])?;
        
        // Verify arrays have compatible sizes
        if values.len() != counts.len() {
            return Err(FlatZincError::MapError {
                message: format!(
                    "global_cardinality: values array length ({}) must match counts array length ({})",
                    values.len(),
                    counts.len()
                ),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Use Selen's gcc (global cardinality constraint) method
        // gcc(&vars, values, counts) constrains that for each i,
        // counts[i] = |{j : vars[j] = values[i]}|
        self.model.gcc(&vars, &values, &counts);
        
        Ok(())
    }
    
    /// Map global_cardinality_low_up_closed: Count with bounds on counts
    /// FlatZinc signature: global_cardinality_low_up_closed(vars, values, low, up)
    /// 
    /// Where:
    /// - vars: array of variables to count in
    /// - values: array of values to count (must be constants)
    /// - low: array of lower bounds for counts
    /// - up: array of upper bounds for counts
    /// 
    /// Constraint: For each i, low[i] <= |{j : vars[j] = values[i]}| <= up[i]
    pub(in crate::mapper) fn map_global_cardinality_low_up_closed(&mut self, constraint: &Constraint) -> FlatZincResult<()> {
        if constraint.args.len() != 4 {
            return Err(FlatZincError::MapError {
                message: "global_cardinality_low_up_closed requires 4 arguments (vars, values, low, up)".to_string(),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // Extract the variables array
        let vars = self.extract_var_array(&constraint.args[0])?;
        
        // Extract the values array (must be constants)
        let values = self.extract_int_array(&constraint.args[1])?;
        
        // Extract the low bounds array
        let low = self.extract_int_array(&constraint.args[2])?;
        
        // Extract the up bounds array
        let up = self.extract_int_array(&constraint.args[3])?;
        
        // Verify arrays have compatible sizes
        if values.len() != low.len() || values.len() != up.len() {
            return Err(FlatZincError::MapError {
                message: format!(
                    "global_cardinality_low_up_closed: arrays must have same length (values: {}, low: {}, up: {})",
                    values.len(),
                    low.len(),
                    up.len()
                ),
                line: Some(constraint.location.line),
                column: Some(constraint.location.column),
            });
        }
        
        // For each value, constrain count with bounds using at_least and at_most
        for i in 0..values.len() {
            let value = values[i];
            let low_bound = low[i];
            let up_bound = up[i];
            
            // Use Selen's at_least and at_most constraints
            // at_least(&vars, value, n) constrains: at least n vars == value
            // at_most(&vars, value, n) constrains: at most n vars == value
            self.model.at_least(&vars, value, low_bound);
            self.model.at_most(&vars, value, up_bound);
        }
        
        Ok(())
    }
}
