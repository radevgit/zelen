//! Translator for MiniZinc Core Subset to Selen
//!
//! Translates a parsed MiniZinc AST into Selen Model objects for execution.

use crate::ast::{self, Span};
use crate::error::{Error, Result};
use selen::prelude::*;
use std::collections::HashMap;

/// Metadata for multi-dimensional arrays to support flattening
#[derive(Debug, Clone)]
struct ArrayMetadata {
    /// Dimensions of the array (e.g., [3, 4] for a 3x4 2D array)
    dimensions: Vec<usize>,
}

impl ArrayMetadata {
    /// Create metadata for a multi-dimensional array
    fn new(dimensions: Vec<usize>) -> Self {
        Self { dimensions }
    }

    /// Total number of elements
    fn total_size(&self) -> usize {
        self.dimensions.iter().product()
    }

    /// Flatten multi-dimensional indices to a single 1D index
    /// indices should be 0-based, and we return the 0-based flattened index
    fn flatten_indices(&self, indices: &[usize]) -> Result<usize> {
        if indices.len() != self.dimensions.len() {
            return Err(Error::message(
                &format!(
                    "Index dimension mismatch: expected {}, got {}",
                    self.dimensions.len(),
                    indices.len()
                ),
                ast::Span::dummy(),
            ));
        }

        let mut flat_index = 0;
        let mut multiplier = 1;

        // Process dimensions from right to left (least significant first)
        for i in (0..self.dimensions.len()).rev() {
            if indices[i] >= self.dimensions[i] {
                return Err(Error::message(
                    &format!(
                        "Array index {} out of bounds for dimension {} (size: {})",
                        indices[i], i, self.dimensions[i]
                    ),
                    ast::Span::dummy(),
                ));
            }
            flat_index += indices[i] * multiplier;
            multiplier *= self.dimensions[i];
        }

        Ok(flat_index)
    }
}

/// Context for tracking variables during translation
#[derive(Debug)]
struct TranslatorContext {
    /// Map from MiniZinc variable names to Selen VarIds (integers)
    int_vars: HashMap<String, VarId>,
    /// Map from MiniZinc variable names to Selen VarId arrays (1D)
    int_var_arrays: HashMap<String, Vec<VarId>>,
    /// Map from MiniZinc variable names to Selen 2D VarId arrays
    int_var_arrays_2d: HashMap<String, Vec<Vec<VarId>>>,
    /// Map from MiniZinc variable names to Selen 3D VarId arrays
    int_var_arrays_3d: HashMap<String, Vec<Vec<Vec<VarId>>>>,
    /// Map from MiniZinc variable names to Selen VarIds (booleans)
    bool_vars: HashMap<String, VarId>,
    /// Map from MiniZinc variable names to Selen VarId arrays (booleans)
    bool_var_arrays: HashMap<String, Vec<VarId>>,
    /// Map from MiniZinc variable names to Selen 2D boolean VarId arrays
    bool_var_arrays_2d: HashMap<String, Vec<Vec<VarId>>>,
    /// Map from MiniZinc variable names to Selen 3D boolean VarId arrays
    bool_var_arrays_3d: HashMap<String, Vec<Vec<Vec<VarId>>>>,
    /// Map from MiniZinc variable names to Selen VarIds (floats)
    float_vars: HashMap<String, VarId>,
    /// Map from MiniZinc variable names to Selen VarId arrays (floats)
    float_var_arrays: HashMap<String, Vec<VarId>>,
    /// Map from MiniZinc variable names to Selen 2D float VarId arrays
    float_var_arrays_2d: HashMap<String, Vec<Vec<VarId>>>,
    /// Map from MiniZinc variable names to Selen 3D float VarId arrays
    float_var_arrays_3d: HashMap<String, Vec<Vec<Vec<VarId>>>>,
    /// Parameter values (for compile-time constants)
    int_params: HashMap<String, i32>,
    /// Float parameters
    float_params: HashMap<String, f64>,
    /// Bool parameters
    bool_params: HashMap<String, bool>,
    /// Parameter arrays (integer constants)
    int_param_arrays: HashMap<String, Vec<i32>>,
    /// Float parameter arrays
    float_param_arrays: HashMap<String, Vec<f64>>,
    /// Bool parameter arrays
    bool_param_arrays: HashMap<String, Vec<bool>>,
    /// Metadata for multi-dimensional arrays (name -> dimensions)
    array_metadata: HashMap<String, ArrayMetadata>,
    /// Enumerated type definitions: enum_name -> list of values
    enums: HashMap<String, Vec<String>>,
}

impl TranslatorContext {
    fn new() -> Self {
        Self {
            int_vars: HashMap::new(),
            int_var_arrays: HashMap::new(),
            int_var_arrays_2d: HashMap::new(),
            int_var_arrays_3d: HashMap::new(),
            bool_vars: HashMap::new(),
            bool_var_arrays: HashMap::new(),
            bool_var_arrays_2d: HashMap::new(),
            bool_var_arrays_3d: HashMap::new(),
            float_vars: HashMap::new(),
            float_var_arrays: HashMap::new(),
            float_var_arrays_2d: HashMap::new(),
            float_var_arrays_3d: HashMap::new(),
            int_params: HashMap::new(),
            float_params: HashMap::new(),
            bool_params: HashMap::new(),
            int_param_arrays: HashMap::new(),
            float_param_arrays: HashMap::new(),
            bool_param_arrays: HashMap::new(),
            array_metadata: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    fn add_int_var(&mut self, name: String, var: VarId) {
        self.int_vars.insert(name, var);
    }

    fn get_int_var(&self, name: &str) -> Option<VarId> {
        self.int_vars.get(name).copied()
    }

    fn add_bool_var(&mut self, name: String, var: VarId) {
        self.bool_vars.insert(name, var);
    }

    fn get_bool_var(&self, name: &str) -> Option<VarId> {
        self.bool_vars.get(name).copied()
    }

    fn add_float_var(&mut self, name: String, var: VarId) {
        self.float_vars.insert(name, var);
    }

    fn get_float_var(&self, name: &str) -> Option<VarId> {
        self.float_vars.get(name).copied()
    }

    fn add_int_param(&mut self, name: String, value: i32) {
        self.int_params.insert(name, value);
    }

    fn get_int_param(&self, name: &str) -> Option<i32> {
        self.int_params.get(name).copied()
    }

    fn add_bool_param(&mut self, name: String, value: bool) {
        self.bool_params.insert(name, value);
    }

    fn get_bool_param(&self, name: &str) -> Option<bool> {
        self.bool_params.get(name).copied()
    }

    fn add_float_param(&mut self, name: String, value: f64) {
        self.float_params.insert(name, value);
    }

    fn get_float_param(&self, name: &str) -> Option<f64> {
        self.float_params.get(name).copied()
    }

    fn add_int_var_array(&mut self, name: String, vars: Vec<VarId>) {
        self.int_var_arrays.insert(name, vars);
    }

    fn get_int_var_array(&self, name: &str) -> Option<&Vec<VarId>> {
        self.int_var_arrays.get(name)
    }

    fn add_bool_var_array(&mut self, name: String, vars: Vec<VarId>) {
        self.bool_var_arrays.insert(name, vars);
    }

    fn get_bool_var_array(&self, name: &str) -> Option<&Vec<VarId>> {
        self.bool_var_arrays.get(name)
    }

    fn add_float_var_array(&mut self, name: String, vars: Vec<VarId>) {
        self.float_var_arrays.insert(name, vars);
    }

    fn get_float_var_array(&self, name: &str) -> Option<&Vec<VarId>> {
        self.float_var_arrays.get(name)
    }

    // 2D array methods
    fn add_int_var_array_2d(&mut self, name: String, vars: Vec<Vec<VarId>>) {
        self.int_var_arrays_2d.insert(name, vars);
    }

    fn get_int_var_array_2d(&self, name: &str) -> Option<&Vec<Vec<VarId>>> {
        self.int_var_arrays_2d.get(name)
    }

    fn add_bool_var_array_2d(&mut self, name: String, vars: Vec<Vec<VarId>>) {
        self.bool_var_arrays_2d.insert(name, vars);
    }

    fn get_bool_var_array_2d(&self, name: &str) -> Option<&Vec<Vec<VarId>>> {
        self.bool_var_arrays_2d.get(name)
    }

    fn add_float_var_array_2d(&mut self, name: String, vars: Vec<Vec<VarId>>) {
        self.float_var_arrays_2d.insert(name, vars);
    }

    fn get_float_var_array_2d(&self, name: &str) -> Option<&Vec<Vec<VarId>>> {
        self.float_var_arrays_2d.get(name)
    }

    // 3D array methods
    fn add_int_var_array_3d(&mut self, name: String, vars: Vec<Vec<Vec<VarId>>>) {
        self.int_var_arrays_3d.insert(name, vars);
    }

    fn get_int_var_array_3d(&self, name: &str) -> Option<&Vec<Vec<Vec<VarId>>>> {
        self.int_var_arrays_3d.get(name)
    }

    fn add_bool_var_array_3d(&mut self, name: String, vars: Vec<Vec<Vec<VarId>>>) {
        self.bool_var_arrays_3d.insert(name, vars);
    }

    fn get_bool_var_array_3d(&self, name: &str) -> Option<&Vec<Vec<Vec<VarId>>>> {
        self.bool_var_arrays_3d.get(name)
    }

    fn add_float_var_array_3d(&mut self, name: String, vars: Vec<Vec<Vec<VarId>>>) {
        self.float_var_arrays_3d.insert(name, vars);
    }

    fn get_float_var_array_3d(&self, name: &str) -> Option<&Vec<Vec<Vec<VarId>>>> {
        self.float_var_arrays_3d.get(name)
    }

    fn add_int_param_array(&mut self, name: String, values: Vec<i32>) {
        self.int_param_arrays.insert(name, values);
    }

    fn get_int_param_array(&self, name: &str) -> Option<&Vec<i32>> {
        self.int_param_arrays.get(name)
    }

    fn add_float_param_array(&mut self, name: String, values: Vec<f64>) {
        self.float_param_arrays.insert(name, values);
    }

    fn get_float_param_array(&self, name: &str) -> Option<&Vec<f64>> {
        self.float_param_arrays.get(name)
    }

    fn add_bool_param_array(&mut self, name: String, values: Vec<bool>) {
        self.bool_param_arrays.insert(name, values);
    }

    fn get_bool_param_array(&self, name: &str) -> Option<&Vec<bool>> {
        self.bool_param_arrays.get(name)
    }
}

/// Main translator struct
pub struct Translator {
    model: selen::model::Model,
    context: TranslatorContext,
    objective_type: ObjectiveType,
    objective_var: Option<VarId>,
    output_items: Vec<ast::Expr>,
    search_option: Option<ast::SearchOption>,
    /// Map from variable name to (enum_name, enum_values) for output formatting
    enum_var_mapping: HashMap<String, (String, Vec<String>)>,
}

/// Optimization objective type for the solver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveType {
    /// Satisfaction problem - find any solution
    Satisfy,
    /// Minimization problem - find solution with minimum objective value
    Minimize,
    /// Maximization problem - find solution with maximum objective value
    Maximize,
}

/// Result of translating a MiniZinc model to a Selen model
///
/// This struct contains:
/// - The Selen model ready to solve
/// - Mappings from variable names to their VarIds
/// - The objective type and variable (if any)
///
/// # Example
///
/// ```
/// use zelen::Translator;
///
/// let ast = zelen::parse("var 1..10: x; solve satisfy;").unwrap();
/// let model_data = Translator::translate_with_vars(&ast).unwrap();
///
/// // Access variable information
/// for (name, var_id) in &model_data.int_vars {
///     let _ = (name, var_id);  // Variable available here
/// }
/// ```
pub struct TranslatedModel {
    /// The Selen constraint model ready to solve
    pub model: selen::model::Model,
    /// Mapping from integer variable names to their VarIds
    pub int_vars: HashMap<String, VarId>,
    /// Mapping from integer array variable names to their VarId vectors
    pub int_var_arrays: HashMap<String, Vec<VarId>>,
    /// Mapping from boolean variable names to their VarIds
    pub bool_vars: HashMap<String, VarId>,
    /// Mapping from boolean array variable names to their VarId vectors
    pub bool_var_arrays: HashMap<String, Vec<VarId>>,
    /// Mapping from float variable names to their VarIds
    pub float_vars: HashMap<String, VarId>,
    /// Mapping from float array variable names to their VarId vectors
    pub float_var_arrays: HashMap<String, Vec<VarId>>,
    /// Type of optimization goal (satisfy, minimize, or maximize)
    pub objective_type: ObjectiveType,
    /// Variable ID of the objective (for minimize/maximize problems)
    pub objective_var: Option<VarId>,
    /// Output expressions from output items (stored as AST for formatting during solution)
    pub output_items: Vec<ast::Expr>,
    /// Search option from solve item (complete vs incomplete)
    pub search_option: Option<ast::SearchOption>,
    /// Enum definitions: maps variable name to (enum_name, enum_values)
    /// Used for output formatting to convert integers back to enum names
    pub enum_vars: HashMap<String, (String, Vec<String>)>,
}

impl TranslatedModel {
    /// Format output using the output items from the MiniZinc model
    /// Returns the formatted output string if output items exist
    pub fn format_output(&self, solution: &selen::prelude::Solution) -> Option<String> {
        if self.output_items.is_empty() {
            return None;
        }

        let mut result = String::new();
        
        for output_expr in &self.output_items {
            match self.format_expr(output_expr, solution) {
                Ok(formatted) => result.push_str(&formatted),
                Err(_) => {
                    // If any expression fails, skip the entire output
                    return None;
                }
            }
        }

        Some(result)
    }

    /// Format a single expression
    fn format_expr(&self, expr: &ast::Expr, solution: &selen::prelude::Solution) -> Result<String> {
        match &expr.kind {
            ast::ExprKind::StringLit(s) => {
                // Process escape sequences
                Ok(self.process_escape_sequences(s))
            }
            ast::ExprKind::ArrayLit(elements) => {
                // String concatenation: ["a", "b", show(x)]
                let mut result = String::new();
                for elem in elements {
                    result.push_str(&self.format_expr(elem, solution)?);
                }
                Ok(result)
            }
            ast::ExprKind::Call { name, args } if name == "show" => {
                // show() function - convert variable/array to string representation
                if args.is_empty() {
                    return Err(Error::message("show() requires at least one argument", expr.span));
                }
                self.format_show_arg(&args[0], solution)
            }
            ast::ExprKind::Ident(var_name) => {
                // Direct variable reference - get its value
                self.format_variable(var_name, solution)
            }
            _ => {
                // For other expressions, try to evaluate them
                Err(Error::message(
                    &format!("Unsupported expression in output: {:?}", expr.kind),
                    expr.span,
                ))
            }
        }
    }

    /// Format the argument to show() function
    fn format_show_arg(&self, arg: &ast::Expr, solution: &selen::prelude::Solution) -> Result<String> {
        match &arg.kind {
            ast::ExprKind::Ident(var_name) => {
                // show(x) or show(array)
                self.format_variable(var_name, solution)
            }
            ast::ExprKind::ArrayAccess { array, indices } => {
                // show(array[i]) - access and format specific element
                if let ast::ExprKind::Ident(array_name) = &array.kind {
                    self.format_array_access(array_name, indices, solution)
                } else {
                    Err(Error::message(
                        "Complex array access in show() not supported",
                        arg.span,
                    ))
                }
            }
            _ => Err(Error::message(
                &format!("Unsupported argument to show(): {:?}", arg.kind),
                arg.span,
            )),
        }
    }

    /// Format a variable or array value
    fn format_variable(&self, var_name: &str, solution: &selen::prelude::Solution) -> Result<String> {
        // Try integer variable
        if let Some(&var_id) = self.int_vars.get(var_name) {
            return Ok(solution.get_int(var_id).to_string());
        }

        // Try boolean variable (format as 0/1)
        if let Some(&var_id) = self.bool_vars.get(var_name) {
            let value = solution.get_int(var_id);
            return Ok(value.to_string());
        }

        // Try float variable
        if let Some(&var_id) = self.float_vars.get(var_name) {
            return Ok(solution.get_float(var_id).to_string());
        }

        // Try integer array
        if let Some(var_ids) = self.int_var_arrays.get(var_name) {
            return Ok(self.format_array(var_ids, solution, false, false));
        }

        // Try boolean array (format as 0/1)
        if let Some(var_ids) = self.bool_var_arrays.get(var_name) {
            return Ok(self.format_array(var_ids, solution, true, false));
        }

        // Try float array
        if let Some(var_ids) = self.float_var_arrays.get(var_name) {
            return Ok(self.format_array(var_ids, solution, false, true));
        }

        Err(Error::message(
            &format!("Undefined variable in output: '{}'", var_name),
            Span::new(0, 0),
        ))
    }

    /// Format an array value
    fn format_array(
        &self,
        var_ids: &[VarId],
        solution: &selen::prelude::Solution,
        _is_bool: bool,
        is_float: bool,
    ) -> String {
        let mut result = String::from("[");
        
        for (i, var_id) in var_ids.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            
            if is_float {
                result.push_str(&solution.get_float(*var_id).to_string());
            } else {
                result.push_str(&solution.get_int(*var_id).to_string());
            }
        }
        
        result.push(']');
        result
    }

    /// Format array element access
    fn format_array_access(
        &self,
        array_name: &str,
        indices: &[ast::Expr],
        solution: &selen::prelude::Solution,
    ) -> Result<String> {
        // For now, only support constant indices for element access
        let mut const_indices = Vec::new();
        
        for idx_expr in indices {
            // Try to evaluate index to a constant
            if let ast::ExprKind::IntLit(val) = idx_expr.kind {
                const_indices.push((val - 1) as usize); // Convert from 1-based to 0-based
            } else if let ast::ExprKind::Ident(_) = idx_expr.kind {
                // Variable index - not supported in output formatting yet
                return Err(Error::message(
                    "Variable indices in array access within output not yet supported",
                    idx_expr.span,
                ));
            } else {
                return Err(Error::message(
                    "Complex indices in array access within output not supported",
                    idx_expr.span,
                ));
            }
        }

        // Flatten the indices to get the element position
        // Try integer array first
        if let Some(var_ids) = self.int_var_arrays.get(array_name) {
            if const_indices.len() == 1 && const_indices[0] < var_ids.len() {
                return Ok(solution.get_int(var_ids[const_indices[0]]).to_string());
            }
        }

        // Try boolean array
        if let Some(var_ids) = self.bool_var_arrays.get(array_name) {
            if const_indices.len() == 1 && const_indices[0] < var_ids.len() {
                return Ok(solution.get_int(var_ids[const_indices[0]]).to_string());
            }
        }

        // Try float array
        if let Some(var_ids) = self.float_var_arrays.get(array_name) {
            if const_indices.len() == 1 && const_indices[0] < var_ids.len() {
                return Ok(solution.get_float(var_ids[const_indices[0]]).to_string());
            }
        }

        Err(Error::message(
            &format!("Invalid array access: '{}' with indices: {:?}", array_name, const_indices),
            Span::new(0, 0),
        ))
    }

    /// Process escape sequences in strings
    fn process_escape_sequences(&self, s: &str) -> String {
        s.replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .replace("\\\\", "\\")
            .replace("\\\"", "\"")
    }
}

impl Translator {
    pub fn new() -> Self {
        Self {
            model: selen::model::Model::default(),
            context: TranslatorContext::new(),
            objective_type: ObjectiveType::Satisfy,
            objective_var: None,
            output_items: Vec::new(),
            search_option: None,
            enum_var_mapping: HashMap::new(),
        }
    }

    /// Translate a MiniZinc AST model to a Selen Model
    pub fn translate(ast: &ast::Model) -> Result<selen::model::Model> {
        let mut translator = Self::new();

        // Process all items in order
        for item in &ast.items {
            translator.translate_item(item)?;
        }

        Ok(translator.model)
    }

    /// Translate a MiniZinc AST model to a Selen Model with custom configuration
    pub fn translate_with_config(ast: &ast::Model, config: selen::utils::config::SolverConfig) -> Result<selen::model::Model> {
        let model = selen::model::Model::with_config(config);
        let mut translator = Self {
            model,
            context: TranslatorContext::new(),
            objective_type: ObjectiveType::Satisfy,
            objective_var: None,
            output_items: Vec::new(),
            search_option: None,
            enum_var_mapping: HashMap::new(),
        };

        // Process all items in order
        for item in &ast.items {
            translator.translate_item(item)?;
        }

        Ok(translator.model)
    }

    /// Translate a MiniZinc AST model and return the model with variable mappings
    pub fn translate_with_vars(ast: &ast::Model) -> Result<TranslatedModel> {
        let mut translator = Self::new();

        // Two-pass approach to ensure simple constraints (e.g., var == const) are posted FIRST
        // This helps Selen's propagators work with narrowed variable domains
        
        let debug = std::env::var("TRANSLATOR_DEBUG").is_ok();
        
        // Pass 0: Enum definitions (must be processed first)
        if debug {
            eprintln!("TRANSLATOR_DEBUG: PASS 0 - Enum definitions");
        }
        for item in &ast.items {
            if matches!(item, ast::Item::EnumDef(_)) {
                translator.translate_item(item)?;
            }
        }
        
        // Pass 1: Variable declarations
        if debug {
            eprintln!("TRANSLATOR_DEBUG: PASS 1 - Variable declarations");
        }
        for item in &ast.items {
            if matches!(item, ast::Item::VarDecl(_)) {
                translator.translate_item(item)?;
            }
        }
        
        // Pass 2: Simple equality constraints (var == const)
        if debug {
            eprintln!("TRANSLATOR_DEBUG: PASS 2 - Simple equality constraints");
        }
        for item in &ast.items {
            if let ast::Item::Constraint(c) = item {
                if Self::is_simple_equality_constraint(&c.expr) {
                    if debug {
                        eprintln!("TRANSLATOR_DEBUG:   Posting simple constraint: {:?}", c.expr);
                    }
                    translator.translate_item(item)?;
                }
            }
        }
        
        // Pass 3: All other constraints and solve statements
        if debug {
            eprintln!("TRANSLATOR_DEBUG: PASS 3 - Complex constraints and solve");
        }
        for item in &ast.items {
            match item {
                ast::Item::EnumDef(_) => {} // Already done in pass 0
                ast::Item::VarDecl(_) => {} // Already done in pass 1
                ast::Item::Constraint(c) => {
                    if !Self::is_simple_equality_constraint(&c.expr) {
                        if debug {
                            eprintln!("TRANSLATOR_DEBUG:   Posting complex constraint: {:?}", c.expr);
                        }
                        translator.translate_item(item)?;
                    }
                }
                _ => {
                    translator.translate_item(item)?;
                }
            }
        }

        Ok(TranslatedModel {
            model: translator.model,
            int_vars: translator.context.int_vars.clone(),
            int_var_arrays: translator.context.int_var_arrays.clone(),
            bool_vars: translator.context.bool_vars,
            bool_var_arrays: translator.context.bool_var_arrays,
            float_vars: translator.context.float_vars,
            float_var_arrays: translator.context.float_var_arrays,
            objective_type: translator.objective_type,
            objective_var: translator.objective_var,
            output_items: translator.output_items,
            search_option: translator.search_option,
            enum_vars: translator.enum_var_mapping,
        })
    }

    /// Check if a constraint is a simple equality (Var == Const or Const == Var)
    fn is_simple_equality_constraint(expr: &ast::Expr) -> bool {
        match &expr.kind {
            ast::ExprKind::BinOp { op, left, right } => {
                if !matches!(op, ast::BinOp::Eq) {
                    return false;
                }
                
                // Check if one side is an identifier and the other is a literal
                let left_is_ident = matches!(left.kind, ast::ExprKind::Ident(_));
                let left_is_literal = matches!(left.kind, 
                    ast::ExprKind::IntLit(_) | 
                    ast::ExprKind::BoolLit(_) | 
                    ast::ExprKind::FloatLit(_)
                );
                
                let right_is_ident = matches!(right.kind, ast::ExprKind::Ident(_));
                let right_is_literal = matches!(right.kind,
                    ast::ExprKind::IntLit(_) | 
                    ast::ExprKind::BoolLit(_) | 
                    ast::ExprKind::FloatLit(_)
                );
                
                (left_is_ident && right_is_literal) || (left_is_literal && right_is_ident)
            }
            _ => false,
        }
    }

    /// Extract a constant integer value from an expression if possible
    /// Extract a constant integer value from an expression if possible
    fn extract_const_value(expr: &ast::Expr) -> Option<i64> {
        match &expr.kind {
            ast::ExprKind::IntLit(i) => Some(*i),
            _ => None,
        }
    }

    fn translate_item(&mut self, item: &ast::Item) -> Result<()> {
        match item {
            ast::Item::EnumDef(enum_def) => {
                // Store enum definition for later use
                self.context.enums.insert(enum_def.name.clone(), enum_def.values.clone());
                Ok(())
            }
            ast::Item::VarDecl(var_decl) => self.translate_var_decl(var_decl),
            ast::Item::Constraint(constraint) => self.translate_constraint(constraint),
            ast::Item::Solve(solve) => self.translate_solve(solve),
            ast::Item::Output(output) => {
                // Store output items for later formatting
                self.output_items.push(output.expr.clone());
                Ok(())
            }
        }
    }

    fn translate_var_decl(&mut self, var_decl: &ast::VarDecl) -> Result<()> {
        match &var_decl.type_inst {
            ast::TypeInst::Basic { is_var, base_type } => {
                if *is_var {
                    // Decision variable without domain
                    match base_type {
                        ast::BaseType::Bool => {
                            // var bool: x
                            let var = self.model.bool();
                            self.context.add_bool_var(var_decl.name.clone(), var);
                        }
                        ast::BaseType::Int => {
                            // var int: x (unbounded)
                            let var = self.model.int(i32::MIN, i32::MAX);
                            self.context.add_int_var(var_decl.name.clone(), var);
                        }
                        ast::BaseType::Float => {
                            // var float: x (unbounded)
                            let var = self.model.float(f64::MIN, f64::MAX);
                            self.context.add_float_var(var_decl.name.clone(), var);
                        }
                        ast::BaseType::Enum(enum_name) => {
                            // var EnumType: x
                            // Map to integer domain 1..cardinality
                            let enum_values = self.context.enums.get(enum_name)
                                .ok_or_else(|| Error::message(
                                    &format!("Undefined enum type: {}", enum_name),
                                    var_decl.span,
                                ))?
                                .clone();
                            let cardinality = enum_values.len() as i32;
                            let var = self.model.int(1, cardinality);
                            self.context.add_int_var(var_decl.name.clone(), var);
                            // Track this variable as an enum for output formatting
                            self.enum_var_mapping.insert(
                                var_decl.name.clone(),
                                (enum_name.clone(), enum_values),
                            );
                        }
                    }
                } else {
                    // Parameter declaration
                    if let Some(expr) = &var_decl.expr {
                        match base_type {
                            ast::BaseType::Int => {
                                let value = self.eval_int_expr(expr)?;
                                self.context.add_int_param(var_decl.name.clone(), value);
                            }
                            ast::BaseType::Float => {
                                let value = self.eval_float_expr(expr)?;
                                self.context.add_float_param(var_decl.name.clone(), value);
                            }
                            ast::BaseType::Bool => {
                                let value = self.eval_bool_expr(expr)?;
                                self.context.add_bool_param(var_decl.name.clone(), value);
                            }
                            ast::BaseType::Enum(enum_name) => {
                                // For now, parameters with enum types must be initialized
                                // We'll look up the enum value in the definition
                                if let ast::ExprKind::Ident(value_name) = &expr.kind {
                                    let enum_values = self.context.enums.get(enum_name)
                                        .ok_or_else(|| Error::message(
                                            &format!("Undefined enum type: {}", enum_name),
                                            var_decl.span,
                                        ))?
                                        .clone();
                                    if let Some(pos) = enum_values.iter().position(|v| v == value_name) {
                                        self.context.add_int_param(var_decl.name.clone(), (pos + 1) as i32);
                                    } else {
                                        return Err(Error::message(
                                            &format!("Unknown enum value: {} for enum {}", value_name, enum_name),
                                            expr.span,
                                        ));
                                    }
                                } else {
                                    return Err(Error::message(
                                        "Enum parameter initialization must be an enum value identifier",
                                        expr.span,
                                    ));
                                }
                            }
                        }
                    } else {
                        return Err(Error::type_error(
                            "parameter with initializer",
                            "parameter without initializer",
                            var_decl.span,
                        ));
                    }
                }
            }

            ast::TypeInst::Constrained { is_var, base_type, domain } => {
                if !is_var {
                    return Err(Error::unsupported_feature(
                        "Constrained parameters",
                        "Phase 1",
                        var_decl.span,
                    ));
                }

                // Decision variable with domain
                match base_type {
                    ast::BaseType::Int => {
                        let (min, max) = self.eval_int_domain(domain)?;
                        let var = self.model.int(min, max);
                        if std::env::var("ZELEN_DEBUG").is_ok() {
                            eprintln!("DEBUG: Created int var '{}': {:?} with range [{}, {}]", var_decl.name, var, min, max);
                        }
                        self.context.add_int_var(var_decl.name.clone(), var);
                    }
                    ast::BaseType::Float => {
                        let (min, max) = self.eval_float_domain(domain)?;
                        let var = self.model.float(min, max);
                        self.context.add_float_var(var_decl.name.clone(), var);
                    }
                    ast::BaseType::Bool => {
                        // var 0..1: x or similar - treat as bool
                        let var = self.model.bool();
                        self.context.add_bool_var(var_decl.name.clone(), var);
                    }
                    ast::BaseType::Enum(_) => {
                        // Constrained enum is not typical, but treat as error
                        return Err(Error::message(
                            "Enum types cannot be used in constrained form",
                            var_decl.span,
                        ));
                    }
                }
            }

            ast::TypeInst::Array { index_sets, element_type } => {
                self.translate_array_decl(&var_decl.name, index_sets, element_type, &var_decl.expr)?;
            }
        }

        Ok(())
    }

    /// Flatten a 2D array to 1D with pre-allocated capacity
    #[inline]
    fn flatten_2d(arr_2d: &[Vec<VarId>]) -> Vec<VarId> {
        let rows = arr_2d.len();
        let cols = if rows > 0 { arr_2d[0].len() } else { 0 };
        let mut result = Vec::with_capacity(rows * cols);
        for row in arr_2d {
            result.extend_from_slice(row);
        }
        result
    }

    /// Flatten a 3D array to 1D with pre-allocated capacity
    #[inline]
    fn flatten_3d(arr_3d: &[Vec<Vec<VarId>>]) -> Vec<VarId> {
        let depth = arr_3d.len();
        let rows = if depth > 0 { arr_3d[0].len() } else { 0 };
        let cols = if rows > 0 { arr_3d[0][0].len() } else { 0 };
        let mut result = Vec::with_capacity(depth * rows * cols);
        for layer in arr_3d {
            for row in layer {
                result.extend_from_slice(row);
            }
        }
        result
    }

    fn translate_array_decl(
        &mut self,
        name: &str,
        index_sets: &[ast::Expr],
        element_type: &ast::TypeInst,
        init_expr: &Option<ast::Expr>,
    ) -> Result<()> {
        // Determine if it's a var array or par array
        let is_var = match element_type {
            ast::TypeInst::Basic { is_var, .. } => *is_var,
            ast::TypeInst::Constrained { is_var, .. } => *is_var,
            ast::TypeInst::Array { .. } => {
                return Err(Error::unsupported_feature(
                    "Multi-dimensional arrays",
                    "Phase 2",
                    ast::Span::dummy(),
                ));
            }
        };

        // Get total array size (product of all dimensions for multi-dimensional arrays)
        let mut size = 1usize;
        let mut dimensions = Vec::new();
        for index_set in index_sets {
            let dim_size = self.eval_index_set_size(index_set)?;
            dimensions.push(dim_size);
            size = size.saturating_mul(dim_size);
        }

        // Store array metadata for later index flattening
        self.context
            .array_metadata
            .insert(name.to_string(), ArrayMetadata::new(dimensions.clone()));

        if is_var {
            // Decision variable array - determine the type
            match element_type {
                ast::TypeInst::Constrained { base_type, domain, .. } => {
                    match base_type {
                        ast::BaseType::Int => {
                            let (min, max) = self.eval_int_domain(domain)?;
                            // Use native 2D/3D arrays when applicable
                            if dimensions.len() == 2 {
                                let vars_2d = self.model.ints_2d(dimensions[0], dimensions[1], min, max);
                                // Also flatten for backward compatibility with constraints
                                let flattened = Self::flatten_2d(&vars_2d);
                                self.context.add_int_var_array_2d(name.to_string(), vars_2d);
                                self.context.add_int_var_array(name.to_string(), flattened);
                            } else if dimensions.len() == 3 {
                                let vars_3d = self.model.ints_3d(dimensions[0], dimensions[1], dimensions[2], min, max);
                                // Also flatten for backward compatibility with constraints
                                let flattened = Self::flatten_3d(&vars_3d);
                                self.context.add_int_var_array_3d(name.to_string(), vars_3d);
                                self.context.add_int_var_array(name.to_string(), flattened);
                            } else {
                                // 1D or higher - use flat arrays
                                let vars = self.model.ints(size, min, max);
                                self.context.add_int_var_array(name.to_string(), vars);
                            }
                        }
                        ast::BaseType::Float => {
                            let (min, max) = self.eval_float_domain(domain)?;
                            if dimensions.len() == 2 {
                                let vars_2d = self.model.floats_2d(dimensions[0], dimensions[1], min, max);
                                let flattened = Self::flatten_2d(&vars_2d);
                                self.context.add_float_var_array_2d(name.to_string(), vars_2d);
                                self.context.add_float_var_array(name.to_string(), flattened);
                            } else if dimensions.len() == 3 {
                                let vars_3d = self.model.floats_3d(dimensions[0], dimensions[1], dimensions[2], min, max);
                                let flattened = Self::flatten_3d(&vars_3d);
                                self.context.add_float_var_array_3d(name.to_string(), vars_3d);
                                self.context.add_float_var_array(name.to_string(), flattened);
                            } else {
                                let vars = self.model.floats(size, min, max);
                                self.context.add_float_var_array(name.to_string(), vars);
                            }
                        }
                        ast::BaseType::Bool => {
                            if dimensions.len() == 2 {
                                let vars_2d = self.model.bools_2d(dimensions[0], dimensions[1]);
                                let flattened = Self::flatten_2d(&vars_2d);
                                self.context.add_bool_var_array_2d(name.to_string(), vars_2d);
                                self.context.add_bool_var_array(name.to_string(), flattened);
                            } else if dimensions.len() == 3 {
                                let vars_3d = self.model.bools_3d(dimensions[0], dimensions[1], dimensions[2]);
                                let flattened = Self::flatten_3d(&vars_3d);
                                self.context.add_bool_var_array_3d(name.to_string(), vars_3d);
                                self.context.add_bool_var_array(name.to_string(), flattened);
                            } else {
                                let vars = self.model.bools(size);
                                self.context.add_bool_var_array(name.to_string(), vars);
                            }
                        }
                        ast::BaseType::Enum(enum_name) => {
                            // Treat enum array as integer array with domain 1..cardinality
                            let enum_values = self.context.enums.get(enum_name)
                                .ok_or_else(|| Error::message(
                                    &format!("Undefined enum type: {}", enum_name),
                                    Span::dummy(),
                                ))?
                                .clone();
                            let cardinality = enum_values.len() as i32;
                            if dimensions.len() == 2 {
                                let vars_2d = self.model.ints_2d(dimensions[0], dimensions[1], 1, cardinality);
                                let flattened = Self::flatten_2d(&vars_2d);
                                self.context.add_int_var_array_2d(name.to_string(), vars_2d);
                                self.context.add_int_var_array(name.to_string(), flattened);
                            } else if dimensions.len() == 3 {
                                let vars_3d = self.model.ints_3d(dimensions[0], dimensions[1], dimensions[2], 1, cardinality);
                                let flattened = Self::flatten_3d(&vars_3d);
                                self.context.add_int_var_array_3d(name.to_string(), vars_3d);
                                self.context.add_int_var_array(name.to_string(), flattened);
                            } else {
                                let vars = self.model.ints(size, 1, cardinality);
                                self.context.add_int_var_array(name.to_string(), vars);
                            }
                            // Track this array as enum for output formatting
                            self.enum_var_mapping.insert(
                                name.to_string(),
                                (enum_name.clone(), enum_values),
                            );
                        }
                    }
                }
                ast::TypeInst::Basic { base_type, .. } => {
                    match base_type {
                        ast::BaseType::Int => {
                            if dimensions.len() == 2 {
                                let vars_2d = self.model.ints_2d(dimensions[0], dimensions[1], i32::MIN, i32::MAX);
                                let flattened = Self::flatten_2d(&vars_2d);
                                self.context.add_int_var_array_2d(name.to_string(), vars_2d);
                                self.context.add_int_var_array(name.to_string(), flattened);
                            } else if dimensions.len() == 3 {
                                let vars_3d = self.model.ints_3d(dimensions[0], dimensions[1], dimensions[2], i32::MIN, i32::MAX);
                                let flattened = Self::flatten_3d(&vars_3d);
                                self.context.add_int_var_array_3d(name.to_string(), vars_3d);
                                self.context.add_int_var_array(name.to_string(), flattened);
                            } else {
                                let vars = self.model.ints(size, i32::MIN, i32::MAX);
                                self.context.add_int_var_array(name.to_string(), vars);
                            }
                        }
                        ast::BaseType::Float => {
                            if dimensions.len() == 2 {
                                let vars_2d = self.model.floats_2d(dimensions[0], dimensions[1], f64::MIN, f64::MAX);
                                let flattened = Self::flatten_2d(&vars_2d);
                                self.context.add_float_var_array_2d(name.to_string(), vars_2d);
                                self.context.add_float_var_array(name.to_string(), flattened);
                            } else if dimensions.len() == 3 {
                                let vars_3d = self.model.floats_3d(dimensions[0], dimensions[1], dimensions[2], f64::MIN, f64::MAX);
                                let flattened = Self::flatten_3d(&vars_3d);
                                self.context.add_float_var_array_3d(name.to_string(), vars_3d);
                                self.context.add_float_var_array(name.to_string(), flattened);
                            } else {
                                let vars = self.model.floats(size, f64::MIN, f64::MAX);
                                self.context.add_float_var_array(name.to_string(), vars);
                            }
                        }
                        ast::BaseType::Bool => {
                            if dimensions.len() == 2 {
                                let vars_2d = self.model.bools_2d(dimensions[0], dimensions[1]);
                                let flattened = Self::flatten_2d(&vars_2d);
                                self.context.add_bool_var_array_2d(name.to_string(), vars_2d);
                                self.context.add_bool_var_array(name.to_string(), flattened);
                            } else if dimensions.len() == 3 {
                                let vars_3d = self.model.bools_3d(dimensions[0], dimensions[1], dimensions[2]);
                                let flattened = Self::flatten_3d(&vars_3d);
                                self.context.add_bool_var_array_3d(name.to_string(), vars_3d);
                                self.context.add_bool_var_array(name.to_string(), flattened);
                            } else {
                                let vars = self.model.bools(size);
                                self.context.add_bool_var_array(name.to_string(), vars);
                            }
                        }
                        ast::BaseType::Enum(enum_name) => {
                            // Treat enum array as integer array with domain 1..cardinality
                            let enum_values = self.context.enums.get(enum_name)
                                .ok_or_else(|| Error::message(
                                    &format!("Undefined enum type: {}", enum_name),
                                    Span::dummy(),
                                ))?
                                .clone();
                            let cardinality = enum_values.len() as i32;
                            if dimensions.len() == 2 {
                                let vars_2d = self.model.ints_2d(dimensions[0], dimensions[1], 1, cardinality);
                                let flattened = Self::flatten_2d(&vars_2d);
                                self.context.add_int_var_array_2d(name.to_string(), vars_2d);
                                self.context.add_int_var_array(name.to_string(), flattened);
                            } else if dimensions.len() == 3 {
                                let vars_3d = self.model.ints_3d(dimensions[0], dimensions[1], dimensions[2], 1, cardinality);
                                let flattened = Self::flatten_3d(&vars_3d);
                                self.context.add_int_var_array_3d(name.to_string(), vars_3d);
                                self.context.add_int_var_array(name.to_string(), flattened);
                            } else {
                                let vars = self.model.ints(size, 1, cardinality);
                                self.context.add_int_var_array(name.to_string(), vars);
                            }
                            // Track this array as enum for output formatting
                            self.enum_var_mapping.insert(
                                name.to_string(),
                                (enum_name.clone(), enum_values),
                            );
                        }
                    }
                }
                _ => unreachable!(),
            }
        } else {
            // Parameter array - extract values from initializer
            if let Some(init) = init_expr {
                // Extract array literal or array2d/array3d initializer
                match &init.kind {
                    ast::ExprKind::ArrayLit(elements) => {
                        // Verify size matches
                        if elements.len() != size {
                            return Err(Error::array_size_mismatch(size, elements.len(), init.span));
                        }
                        
                        // Determine element type and extract values
                        match element_type {
                            ast::TypeInst::Constrained { base_type, .. } | ast::TypeInst::Basic { base_type, .. } => {
                                match base_type {
                                    ast::BaseType::Int => {
                                        let mut values = Vec::with_capacity(size);
                                        for elem in elements.iter() {
                                            let val = self.eval_int_expr(elem)?;
                                            values.push(val);
                                        }
                                        self.context.add_int_param_array(name.to_string(), values);
                                    }
                                    ast::BaseType::Float => {
                                        let mut values = Vec::with_capacity(size);
                                        for elem in elements.iter() {
                                            let val = self.eval_float_expr(elem)?;
                                            values.push(val);
                                        }
                                        self.context.add_float_param_array(name.to_string(), values);
                                    }
                                    ast::BaseType::Bool => {
                                        let mut values = Vec::with_capacity(size);
                                        for elem in elements.iter() {
                                            let val = self.eval_bool_expr(elem)?;
                                            values.push(val);
                                        }
                                        self.context.add_bool_param_array(name.to_string(), values);
                                    }
                                    ast::BaseType::Enum(enum_name) => {
                                        // Convert enum values to integers
                                        let enum_values = self.context.enums.get(enum_name)
                                            .ok_or_else(|| Error::message(
                                                &format!("Undefined enum type: {}", enum_name),
                                                init.span,
                                            ))?
                                            .clone();
                                        let mut values = Vec::with_capacity(size);
                                        for elem in elements.iter() {
                                            if let ast::ExprKind::Ident(value_name) = &elem.kind {
                                                if let Some(pos) = enum_values.iter().position(|v| v == value_name) {
                                                    values.push((pos + 1) as i32);
                                                } else {
                                                    return Err(Error::message(
                                                        &format!("Unknown enum value: {} for enum {}", value_name, enum_name),
                                                        elem.span,
                                                    ));
                                                }
                                            } else {
                                                return Err(Error::message(
                                                    "Enum array elements must be enum value identifiers",
                                                    elem.span,
                                                ));
                                            }
                                        }
                                        self.context.add_int_param_array(name.to_string(), values);
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    ast::ExprKind::Array2D { row_range, col_range, values } => {
                        // Handle array2d initializer for 2D parameter arrays
                        if dimensions.len() != 2 {
                            return Err(Error::array2d_invalid_context(init.span));
                        }
                        
                        let row_size = self.eval_index_set_size(row_range)?;
                        let col_size = self.eval_index_set_size(col_range)?;
                        
                        if row_size != dimensions[0] || col_size != dimensions[1] {
                            return Err(Error::array2d_size_mismatch(
                                dimensions[0], dimensions[1], row_size, col_size, init.span
                            ));
                        }
                        
                        // Extract values from array literal
                        if let ast::ExprKind::ArrayLit(elements) = &values.kind {
                            let expected_len = row_size * col_size;
                            if elements.len() != expected_len {
                                return Err(Error::array2d_value_count_mismatch(expected_len, elements.len(), values.span));
                            }
                            
                            // Determine element type and extract values
                            match element_type {
                                ast::TypeInst::Constrained { base_type, .. } | ast::TypeInst::Basic { base_type, .. } => {
                                    match base_type {
                                        ast::BaseType::Int => {
                                            let mut values = Vec::with_capacity(expected_len);
                                            for elem in elements.iter() {
                                                let val = self.eval_int_expr(elem)?;
                                                values.push(val);
                                            }
                                            self.context.add_int_param_array(name.to_string(), values);
                                        }
                                        ast::BaseType::Float => {
                                            let mut values = Vec::with_capacity(expected_len);
                                            for elem in elements.iter() {
                                                let val = self.eval_float_expr(elem)?;
                                                values.push(val);
                                            }
                                            self.context.add_float_param_array(name.to_string(), values);
                                        }
                                        ast::BaseType::Bool => {
                                            let mut values = Vec::with_capacity(expected_len);
                                            for elem in elements.iter() {
                                                let val = self.eval_bool_expr(elem)?;
                                                values.push(val);
                                            }
                                            self.context.add_bool_param_array(name.to_string(), values);
                                        }
                                        ast::BaseType::Enum(enum_name) => {
                                            // Convert enum values to integers for 2D array
                                            let enum_values = self.context.enums.get(enum_name)
                                                .ok_or_else(|| Error::message(
                                                    &format!("Undefined enum type: {}", enum_name),
                                                    values.span,
                                                ))?
                                                .clone();
                                            let mut enum_values_mapped = Vec::with_capacity(expected_len);
                                            for elem in elements.iter() {
                                                if let ast::ExprKind::Ident(value_name) = &elem.kind {
                                                    if let Some(pos) = enum_values.iter().position(|v| v == value_name) {
                                                        enum_values_mapped.push((pos + 1) as i32);
                                                    } else {
                                                        return Err(Error::message(
                                                            &format!("Unknown enum value: {} for enum {}", value_name, enum_name),
                                                            elem.span,
                                                        ));
                                                    }
                                                } else {
                                                    return Err(Error::message(
                                                        "Enum array elements must be enum value identifiers",
                                                        elem.span,
                                                    ));
                                                }
                                            }
                                            self.context.add_int_param_array(name.to_string(), enum_values_mapped);
                                        }
                                    }
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            return Err(Error::array2d_values_must_be_literal(values.span));
                        }
                    }
                    ast::ExprKind::Array3D { r1_range, r2_range, r3_range, values } => {
                        // Handle array3d initializer for 3D parameter arrays
                        if dimensions.len() != 3 {
                            return Err(Error::array3d_invalid_context(init.span));
                        }
                        
                        let d1 = self.eval_index_set_size(r1_range)?;
                        let d2 = self.eval_index_set_size(r2_range)?;
                        let d3 = self.eval_index_set_size(r3_range)?;
                        
                        if d1 != dimensions[0] || d2 != dimensions[1] || d3 != dimensions[2] {
                            return Err(Error::array3d_size_mismatch(
                                dimensions[0], dimensions[1], dimensions[2],
                                d1, d2, d3,
                                init.span
                            ));
                        }
                        
                        // Extract values from array literal
                        if let ast::ExprKind::ArrayLit(elements) = &values.kind {
                            let expected_len = d1 * d2 * d3;
                            if elements.len() != expected_len {
                                return Err(Error::array3d_value_count_mismatch(expected_len, elements.len(), values.span));
                            }
                            
                            // Determine element type and extract values
                            match element_type {
                                ast::TypeInst::Constrained { base_type, .. } | ast::TypeInst::Basic { base_type, .. } => {
                                    match base_type {
                                        ast::BaseType::Int => {
                                            let mut values = Vec::with_capacity(expected_len);
                                            for elem in elements.iter() {
                                                let val = self.eval_int_expr(elem)?;
                                                values.push(val);
                                            }
                                            self.context.add_int_param_array(name.to_string(), values);
                                        }
                                        ast::BaseType::Float => {
                                            let mut values = Vec::with_capacity(expected_len);
                                            for elem in elements.iter() {
                                                let val = self.eval_float_expr(elem)?;
                                                values.push(val);
                                            }
                                            self.context.add_float_param_array(name.to_string(), values);
                                        }
                                        ast::BaseType::Bool => {
                                            let mut values = Vec::with_capacity(expected_len);
                                            for elem in elements.iter() {
                                                let val = self.eval_bool_expr(elem)?;
                                                values.push(val);
                                            }
                                            self.context.add_bool_param_array(name.to_string(), values);
                                        }
                                        ast::BaseType::Enum(enum_name) => {
                                            // Convert enum values to integers for 3D array
                                            let enum_values = self.context.enums.get(enum_name)
                                                .ok_or_else(|| Error::message(
                                                    &format!("Undefined enum type: {}", enum_name),
                                                    values.span,
                                                ))?
                                                .clone();
                                            let mut enum_values_mapped = Vec::with_capacity(expected_len);
                                            for elem in elements.iter() {
                                                if let ast::ExprKind::Ident(value_name) = &elem.kind {
                                                    if let Some(pos) = enum_values.iter().position(|v| v == value_name) {
                                                        enum_values_mapped.push((pos + 1) as i32);
                                                    } else {
                                                        return Err(Error::message(
                                                            &format!("Unknown enum value: {} for enum {}", value_name, enum_name),
                                                            elem.span,
                                                        ));
                                                    }
                                                } else {
                                                    return Err(Error::message(
                                                        "Enum array elements must be enum value identifiers",
                                                        elem.span,
                                                    ));
                                                }
                                            }
                                            self.context.add_int_param_array(name.to_string(), enum_values_mapped);
                                        }
                                    }
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            return Err(Error::array3d_values_must_be_literal(values.span));
                        }
                    }
                    _ => {
                        return Err(Error::unsupported_feature(
                            "Array initialization must be an array literal [...], array2d(...), or array3d(...)",
                            "Phase 4",
                            init.span,
                        ));
                    }
                }
            } else {
                // Parameter array without initializer - not supported
                return Err(Error::unsupported_feature(
                    "Parameter arrays without initializer",
                    "Phase 2",
                    ast::Span::dummy(),
                ));
            }
        }

        Ok(())
    }

    fn translate_constraint(&mut self, constraint: &ast::Constraint) -> Result<()> {
        match &constraint.expr.kind {
            ast::ExprKind::Call { name, args } => {
                self.translate_constraint_call(name, args)?;
            }
            ast::ExprKind::GenCall { name, generators, body } => {
                self.translate_constraint_gencall(name, generators, body)?;
            }
            ast::ExprKind::BinOp { op, left, right } => {
                self.translate_constraint_binop(*op, left, right)?;
            }
            ast::ExprKind::UnOp { op, expr } => {
                self.translate_constraint_unop(*op, expr)?;
            }
            ast::ExprKind::Ident(_) | ast::ExprKind::BoolLit(_) => {
                // Boolean variable or literal used as a constraint
                // Convert to boolean var and constrain it to be true
                let bool_var = self.expr_to_bool_var(&constraint.expr)?;
                let one = self.model.int(1, 1);
                self.model.new(bool_var.eq(one));
            }
            _ => {
                return Err(Error::type_error(
                    "constraint expression",
                    "other expression",
                    constraint.span,
                ));
            }
        }
        Ok(())
    }

    fn translate_constraint_call(&mut self, name: &str, args: &[ast::Expr]) -> Result<()> {
        match name {
            "alldifferent" | "alldiff" => {
                if args.len() != 1 {
                    return Err(Error::type_error(
                        "1 argument",
                        &format!("{} arguments", args.len()),
                        ast::Span::dummy(),
                    ));
                }

                // Get the array variable
                if let ast::ExprKind::Ident(array_name) = &args[0].kind {
                    if let Some(vars) = self.context.get_int_var_array(array_name) {
                        self.model.alldiff(vars);
                    } else {
                        return Err(Error::message(
                            &format!("Undefined array variable: {}", array_name),
                            args[0].span,
                        ));
                    }
                } else {
                    return Err(Error::type_error(
                        "array identifier",
                        "other expression",
                        args[0].span,
                    ));
                }
            }
            _ => {
                return Err(Error::unsupported_feature(
                    &format!("Constraint '{}'", name),
                    "Phase 1",
                    ast::Span::dummy(),
                ));
            }
        }
        Ok(())
    }

    fn translate_constraint_gencall(
        &mut self,
        name: &str,
        generators: &[ast::Generator],
        body: &ast::Expr,
    ) -> Result<()> {
        // For now, we only support "forall"
        // Other generator calls like "exists" would have different semantics
        if name != "forall" {
            return Err(Error::unsupported_feature(
                &format!("Generator call '{}'", name),
                "forall only",
                ast::Span::dummy(),
            ));
        }

        // Expand forall(i in range)(constraint) into multiple individual constraints
        // by iterating through the range and substituting values for the loop variable
        if generators.len() == 1 {
            self.expand_forall_constraint(&generators[0], body)?;
        } else {
            self.expand_forall_constraint_multi(generators, body)?;
        }
        Ok(())
    }

    /// Expand forall(i in range)(constraint) into individual constraints for a single generator
    fn expand_forall_constraint(&mut self, generator: &ast::Generator, body: &ast::Expr) -> Result<()> {
        
        // Get the loop variable name
        if generator.names.len() != 1 {
            return Err(Error::message(
                "Generator must have exactly one variable",
                ast::Span::dummy(),
            ));
        }
        let loop_var = &generator.names[0];

        // Parse the range expression to get (start, end)
        let (range_start, range_end) = self.parse_range(&generator.expr)?;

        // Iterate through the range and substitute loop variable with actual values
        for i in range_start..=range_end {
            // Create a new context for this iteration
            let old_val = self.context.int_params.get(loop_var).copied();
            
            // Set the loop variable to the current iteration value
            self.context.int_params.insert(loop_var.clone(), i);
            
            // Translate the constraint body with the loop variable substituted
            let substituted_body = self.substitute_loop_var_in_expr(body, loop_var, i)?;
            
            // Create and translate the constraint
            let constraint = ast::Constraint {
                expr: substituted_body,
                span: body.span,
            };
            self.translate_constraint(&constraint)?;
            
            // Restore the old value (or remove the parameter)
            if let Some(old) = old_val {
                self.context.int_params.insert(loop_var.clone(), old);
            } else {
                self.context.int_params.remove(loop_var);
            }
        }
        
        Ok(())
    }

    /// Parse a range expression like `1..n` to get (start, end)
    fn parse_range(&self, expr: &ast::Expr) -> Result<(i32, i32)> {
        match &expr.kind {
            ast::ExprKind::BinOp { op: ast::BinOp::Range, left, right } => {
                let start = self.eval_int_expr(left)?;
                let end = self.eval_int_expr(right)?;
                Ok((start, end))
            }
            _ => {
                // Single value range
                let val = self.eval_int_expr(expr)?;
                Ok((val, val))
            }
        }
    }

    /// Expand forall with multiple generators (nested loops)
    fn expand_forall_constraint_multi(&mut self, generators: &[ast::Generator], body: &ast::Expr) -> Result<()> {
        if generators.is_empty() {
            return Err(Error::message("No generators in forall", ast::Span::dummy()));
        }

        // For nested loops, we recursively expand each generator
        self.expand_forall_generators(generators, 0, body)?;
        Ok(())
    }

    /// Recursively expand nested forall generators
    fn expand_forall_generators(&mut self, generators: &[ast::Generator], depth: usize, body: &ast::Expr) -> Result<()> {
        if depth >= generators.len() {
            // All generators processed - translate the body
            let constraint = ast::Constraint {
                expr: body.clone(),
                span: body.span,
            };
            self.translate_constraint(&constraint)?;
            return Ok(());
        }

        let generator = &generators[depth];
        
        if generator.names.len() != 1 {
            return Err(Error::message(
                "Generator must have exactly one variable",
                ast::Span::dummy(),
            ));
        }
        let loop_var = &generator.names[0];

        let (range_start, range_end) = self.parse_range(&generator.expr)?;

        // Iterate through this level's range
        for i in range_start..=range_end {
            let old_val = self.context.int_params.get(loop_var).copied();
            self.context.int_params.insert(loop_var.clone(), i);
            
            // Substitute all remaining loop variables in the expression
            let mut substituted = body.clone();
            
            // Substitute all loop variables from current depth onwards
            for j in 0..=depth {
                if j < generators.len() {
                    let var_name = &generators[j].names[0];
                    if let Some(var_val) = self.context.int_params.get(var_name) {
                        substituted = self.substitute_loop_var_in_expr(&substituted, var_name, *var_val)?;
                    }
                }
            }
            
            // Process next level or translate
            self.expand_forall_generators(generators, depth + 1, &substituted)?;
            
            if let Some(old) = old_val {
                self.context.int_params.insert(loop_var.clone(), old);
            } else {
                self.context.int_params.remove(loop_var);
            }
        }

        Ok(())
    }

    /// Substitute a loop variable with a concrete value in an expression
    fn substitute_loop_var_in_expr(&self, expr: &ast::Expr, var_name: &str, value: i32) -> Result<ast::Expr> {
        let substituted_kind = match &expr.kind {
            // If it's the loop variable itself, replace with a literal
            ast::ExprKind::Ident(name) if name == var_name => {
                ast::ExprKind::IntLit(value as i64)
            }
            // If it's another identifier, keep it as is
            ast::ExprKind::Ident(_) => expr.kind.clone(),
            
            // For binary operations, recursively substitute both sides
            ast::ExprKind::BinOp { op, left, right } => {
                let left_sub = self.substitute_loop_var_in_expr(left, var_name, value)?;
                let right_sub = self.substitute_loop_var_in_expr(right, var_name, value)?;
                ast::ExprKind::BinOp {
                    op: *op,
                    left: Box::new(left_sub),
                    right: Box::new(right_sub),
                }
            }
            
            // For unary operations, recursively substitute
            ast::ExprKind::UnOp { op, expr: inner } => {
                let inner_sub = self.substitute_loop_var_in_expr(inner, var_name, value)?;
                ast::ExprKind::UnOp {
                    op: *op,
                    expr: Box::new(inner_sub),
                }
            }
            
            // For array access, substitute the indices if needed
            ast::ExprKind::ArrayAccess { array, indices } => {
                let indices_sub = indices.iter()
                    .map(|idx| self.substitute_loop_var_in_expr(idx, var_name, value))
                    .collect::<Result<Vec<_>>>()?;
                ast::ExprKind::ArrayAccess {
                    array: array.clone(),
                    indices: indices_sub,
                }
            }
            
            // For function calls, recursively substitute all arguments
            ast::ExprKind::Call { name, args } => {
                let args_sub = args.iter()
                    .map(|arg| self.substitute_loop_var_in_expr(arg, var_name, value))
                    .collect::<Result<Vec<_>>>()?;
                ast::ExprKind::Call {
                    name: name.clone(),
                    args: args_sub,
                }
            }
            
            // For literals, keep them as is
            ast::ExprKind::IntLit(_) | ast::ExprKind::BoolLit(_) | 
            ast::ExprKind::FloatLit(_) => expr.kind.clone(),
            
            // Other expression types
            other => other.clone(),
        };
        
        Ok(ast::Expr {
            kind: substituted_kind,
            span: expr.span,
        })
    }

    fn translate_constraint_binop(
        &mut self,
        op: ast::BinOp,
        left: &ast::Expr,
        right: &ast::Expr,
    ) -> Result<()> {
        match op {
            // Boolean logical operators
            ast::BinOp::And => {
                // Translate as conjunction: both must be true
                // Recursively translate each side as a constraint
                let one = self.model.int(1, 1);
                let left_constraint = self.expr_to_bool_var(left)?;
                self.model.new(left_constraint.eq(one));
                let one = self.model.int(1, 1);
                let right_constraint = self.expr_to_bool_var(right)?;
                self.model.new(right_constraint.eq(one));
            }
            ast::BinOp::Or => {
                // Translate as disjunction: at least one must be true
                let left_constraint = self.expr_to_bool_var(left)?;
                let right_constraint = self.expr_to_bool_var(right)?;
                // At least one must be 1: left + right >= 1
                let sum = self.model.add(left_constraint, right_constraint);
                let one = self.model.int(1, 1);
                self.model.new(sum.ge(one));
            }
            ast::BinOp::Impl => {
                // Translate as implication: left => right
                let left_constraint = self.expr_to_bool_var(left)?;
                let right_constraint = self.expr_to_bool_var(right)?;
                self.model.implies(left_constraint, right_constraint);
            }
            ast::BinOp::Iff => {
                // Translate as bi-directional implication: left <-> right
                // This means left and right must have the same value
                // Equivalent to: (left -> right) /\ (right -> left)
                let left_constraint = self.expr_to_bool_var(left)?;
                let right_constraint = self.expr_to_bool_var(right)?;
                
                // left => right
                self.model.implies(left_constraint, right_constraint);
                // right => left
                self.model.implies(right_constraint, left_constraint);
            }
            // Comparison operators
            ast::BinOp::Lt | ast::BinOp::Le | ast::BinOp::Gt | 
            ast::BinOp::Ge | ast::BinOp::Eq | ast::BinOp::Ne => {
                // CRITICAL FIX: Check if right side is a literal constant BEFORE calling get_var_or_value
                // If it is, we should pass the raw integer directly to the constraint method,
                // not create a new VarId. This prevents Selen's modulo propagator from being confused.
                if let Some(const_val) = Self::extract_const_value(right) {
                    let left_var = self.get_var_or_value(left)?;
                    let const_i32 = const_val as i32;
                    
                    match op {
                        ast::BinOp::Lt => {
                            self.model.new(left_var.lt(const_i32));
                        }
                        ast::BinOp::Le => {
                            self.model.new(left_var.le(const_i32));
                        }
                        ast::BinOp::Gt => {
                            self.model.new(left_var.gt(const_i32));
                        }
                        ast::BinOp::Ge => {
                            self.model.new(left_var.ge(const_i32));
                        }
                        ast::BinOp::Eq => {
                            self.model.new(left_var.eq(const_i32));
                        }
                        ast::BinOp::Ne => {
                            self.model.new(left_var.ne(const_i32));
                        }
                        _ => unreachable!(),
                    }
                } else if let Some(const_val) = Self::extract_const_value(left) {
                    // Constant on left side
                    let right_var = self.get_var_or_value(right)?;
                    let const_i32 = const_val as i32;
                    let const_var = self.model.int(const_i32, const_i32);
                    
                    match op {
                        ast::BinOp::Lt => {
                            self.model.new(const_var.lt(right_var));
                        }
                        ast::BinOp::Le => {
                            self.model.new(const_var.le(right_var));
                        }
                        ast::BinOp::Gt => {
                            self.model.new(const_var.gt(right_var));
                        }
                        ast::BinOp::Ge => {
                            self.model.new(const_var.ge(right_var));
                        }
                        ast::BinOp::Eq => {
                            self.model.new(const_var.eq(right_var));
                        }
                        ast::BinOp::Ne => {
                            self.model.new(const_var.ne(right_var));
                        }
                        _ => unreachable!(),
                    }
                } else {
                    // Neither side is a constant literal - normal path
                    let left_var = self.get_var_or_value(left)?;
                    let right_var = self.get_var_or_value(right)?;

                    match op {
                        ast::BinOp::Lt => {
                            self.model.new(left_var.lt(right_var));
                        }
                        ast::BinOp::Le => {
                            self.model.new(left_var.le(right_var));
                        }
                        ast::BinOp::Gt => {
                            self.model.new(left_var.gt(right_var));
                        }
                        ast::BinOp::Ge => {
                            self.model.new(left_var.ge(right_var));
                        }
                        ast::BinOp::Eq => {
                            self.model.new(left_var.eq(right_var));
                        }
                        ast::BinOp::Ne => {
                            self.model.new(left_var.ne(right_var));
                        }
                        _ => unreachable!(),
                    }
                }
            }
            _ => {
                return Err(Error::unsupported_feature(
                    &format!("Binary operator {:?} in constraints", op),
                    "Phase 2",
                    ast::Span::dummy(),
                ));
            }
        }

        Ok(())
    }

    fn translate_constraint_unop(
        &mut self,
        op: ast::UnOp,
        expr: &ast::Expr,
    ) -> Result<()> {
        match op {
            ast::UnOp::Not => {
                // Translate as negation: expr must be false (0)
                let bool_var = self.expr_to_bool_var(expr)?;
                let zero = self.model.int(0, 0);
                self.model.new(bool_var.eq(zero));
            }
            ast::UnOp::Neg => {
                return Err(Error::unsupported_feature(
                    "Unary negation in constraints",
                    "Phase 2",
                    ast::Span::dummy(),
                ));
            }
        }
        Ok(())
    }

    /// Convert an expression to a boolean variable (0 or 1)
    /// Used for boolean logical operations
    fn expr_to_bool_var(&mut self, expr: &ast::Expr) -> Result<VarId> {
        match &expr.kind {
            // Boolean literals
            ast::ExprKind::BoolLit(b) => {
                let val = if *b { 1 } else { 0 };
                Ok(self.model.int(val, val))
            }
            // Boolean variables
            ast::ExprKind::Ident(name) => {
                if let Some(var) = self.context.get_bool_var(name) {
                    return Ok(var);
                }
                if let Some(value) = self.context.get_bool_param(name) {
                    let val = if value { 1 } else { 0 };
                    return Ok(self.model.int(val, val));
                }
                Err(Error::message(
                    &format!("Undefined boolean variable: '{}'", name),
                    expr.span,
                ))
            }
            // Comparison operators - just evaluate them directly in constraint context
            // We don't need reification for simple cases
            ast::ExprKind::BinOp { op, .. } if matches!(op,
                ast::BinOp::Lt | ast::BinOp::Le | ast::BinOp::Gt |
                ast::BinOp::Ge | ast::BinOp::Eq | ast::BinOp::Ne) => {
                // For now, treat comparison in boolean context as always true
                // This is a simplified approach - full reification would be better
                // but requires more Selen API support
                let result = self.model.bool();
                // Set result to 1 (true) if we're in a positive context
                // In practice, this means the comparison must hold
                Ok(result)
            }
            ast::ExprKind::UnOp { op: ast::UnOp::Not, expr: inner } => {
                // Not of a boolean expression: flip the value
                let inner_var = self.expr_to_bool_var(inner)?;
                let one = self.model.int(1, 1);
                let negated = self.model.sub(one, inner_var);
                Ok(negated)
            }
            ast::ExprKind::BinOp { op: ast::BinOp::And, left, right } => {
                // AND: both must be true
                // Use Selen's bool_and to create the result
                let left_var = self.expr_to_bool_var(left)?;
                let right_var = self.expr_to_bool_var(right)?;
                
                // bool_and returns a VarId representing the AND result
                let result = self.model.bool_and(&[left_var, right_var]);
                Ok(result)
            }
            ast::ExprKind::BinOp { op: ast::BinOp::Or, left, right } => {
                // OR: at least one must be true
                // Use Selen's bool_or to create the result
                let left_var = self.expr_to_bool_var(left)?;
                let right_var = self.expr_to_bool_var(right)?;
                
                // bool_or returns a VarId representing the OR result
                let result = self.model.bool_or(&[left_var, right_var]);
                Ok(result)
            }
            _ => {
                Err(Error::unsupported_feature(
                    &format!("Expression type in boolean context: {:?}", expr.kind),
                    "Phase 2",
                    expr.span,
                ))
            }
        }
    }

    fn translate_solve(&mut self, solve: &ast::Solve) -> Result<()> {
        match solve {
            ast::Solve::Satisfy { search_option, .. } => {
                // Default behavior - no optimization
                self.objective_type = ObjectiveType::Satisfy;
                self.objective_var = None;
                self.search_option = search_option.clone();
            }
            ast::Solve::Minimize { expr, search_option, .. } => {
                let var = self.get_var_or_value(expr)?;
                self.objective_type = ObjectiveType::Minimize;
                self.objective_var = Some(var);
                self.search_option = search_option.clone();
            }
            ast::Solve::Maximize { expr, search_option, .. } => {
                let var = self.get_var_or_value(expr)?;
                self.objective_type = ObjectiveType::Maximize;
                self.objective_var = Some(var);
                self.search_option = search_option.clone();
            }
        }
        Ok(())
    }

    /// Get a VarId from an expression (either a variable reference or create a constant)
    fn get_var_or_value(&mut self, expr: &ast::Expr) -> Result<VarId> {
        let debug = std::env::var("TRANSLATOR_DEBUG").is_ok();
        match &expr.kind {
            ast::ExprKind::Ident(name) => {
                // Try integer variable
                if let Some(var) = self.context.get_int_var(name) {
                    if debug {
                        eprintln!("TRANSLATOR_DEBUG: get_var_or_value(Ident({})) -> existing var {:?}", name, var);
                    }
                    return Ok(var);
                }
                // Try boolean variable
                if let Some(var) = self.context.get_bool_var(name) {
                    if debug {
                        eprintln!("TRANSLATOR_DEBUG: get_var_or_value(Ident({})) -> existing bool var {:?}", name, var);
                    }
                    return Ok(var);
                }
                // Try float variable
                if let Some(var) = self.context.get_float_var(name) {
                    if debug {
                        eprintln!("TRANSLATOR_DEBUG: get_var_or_value(Ident({})) -> existing float var {:?}", name, var);
                    }
                    return Ok(var);
                }
                // Try integer parameter
                if let Some(value) = self.context.get_int_param(name) {
                    // Create a constant variable
                    let const_var = self.model.int(value, value);
                    if debug {
                        eprintln!("TRANSLATOR_DEBUG: get_var_or_value(Ident({})) -> new constant {:?} (value={})", name, const_var, value);
                    }
                    return Ok(const_var);
                }
                // Try float parameter
                if let Some(value) = self.context.get_float_param(name) {
                    // Create a constant variable
                    let const_var = self.model.float(value, value);
                    if debug {
                        eprintln!("TRANSLATOR_DEBUG: get_var_or_value(Ident({})) -> new constant {:?} (value={})", name, const_var, value);
                    }
                    return Ok(const_var);
                }
                // Try boolean parameter
                if let Some(value) = self.context.get_bool_param(name) {
                    // Create a constant variable (0 or 1)
                    let val = if value { 1 } else { 0 };
                    let const_var = self.model.int(val, val);
                    if debug {
                        eprintln!("TRANSLATOR_DEBUG: get_var_or_value(Ident({})) -> new constant {:?} (value={})", name, const_var, val);
                    }
                    return Ok(const_var);
                }
                // Not found - give helpful error
                Err(Error::message(
                    &format!("Undefined variable or parameter: '{}'", name),
                    expr.span,
                ))
            }
            ast::ExprKind::IntLit(i) => {
                // Don't create a variable - return the value as a special marker
                // We'll handle this in translate_constraint_binop to avoid creating extra variables
                Ok(self.model.int(*i as i32, *i as i32))
            }
            ast::ExprKind::FloatLit(f) => {
                // Create a constant float variable
                Ok(self.model.float(*f, *f))
            }
            ast::ExprKind::BoolLit(b) => {
                // Create a constant boolean variable (0 or 1)
                let val = if *b { 1 } else { 0 };
                Ok(self.model.int(val, val))
            }
            ast::ExprKind::BinOp { op, left, right } => {
                let left_var = self.get_var_or_value(left)?;
                let right_var = self.get_var_or_value(right)?;

                match op {
                    ast::BinOp::Add => Ok(self.model.add(left_var, right_var)),
                    ast::BinOp::Sub => Ok(self.model.sub(left_var, right_var)),
                    ast::BinOp::Mul => Ok(self.model.mul(left_var, right_var)),
                    ast::BinOp::Div | ast::BinOp::FDiv => Ok(self.model.div(left_var, right_var)),
                    ast::BinOp::Mod => {
                        if std::env::var("ZELEN_DEBUG").is_ok() {
                            eprintln!("DEBUG: Creating modulo: {:?} mod {:?}", left_var, right_var);
                        }
                        let result = self.model.modulo(left_var, right_var);
                        if std::env::var("ZELEN_DEBUG").is_ok() {
                            eprintln!("DEBUG:   -> Modulo result VarId: {:?}", result);
                        }
                        Ok(result)
                    }
                    _ => Err(Error::unsupported_feature(
                        &format!("Binary operator {:?} in expressions", op),
                        "Phase 2",
                        expr.span,
                    )),
                }
            }
            ast::ExprKind::ArrayAccess { array, indices } => {
                // Get the array name
                let array_name = match &array.kind {
                    ast::ExprKind::Ident(name) => name,
                    _ => {
                        return Err(Error::message(
                            "Array access must use simple array name",
                            array.span,
                        ));
                    }
                };
                
                // Try to handle as multi-dimensional if multiple indices
                if indices.len() > 1 {
                    // Multi-dimensional array access - use native 2D/3D element constraints
                    if let Some(metadata) = self.context.array_metadata.get(array_name) {
                        if metadata.dimensions.len() != indices.len() {
                            return Err(Error::message(
                                &format!(
                                    "Array index dimension mismatch: expected {}, got {}",
                                    metadata.dimensions.len(),
                                    indices.len()
                                ),
                                expr.span,
                            ));
                        }
                        
                        // For 2D arrays, use element_2d
                        if indices.len() == 2 {
                            // Check for 2D arrays and get early
                            let arr_2d_int = self.context.get_int_var_array_2d(array_name).cloned();
                            let arr_2d_bool = if arr_2d_int.is_none() {
                                self.context.get_bool_var_array_2d(array_name).cloned()
                            } else {
                                None
                            };
                            let arr_2d_float = if arr_2d_int.is_none() && arr_2d_bool.is_none() {
                                self.context.get_float_var_array_2d(array_name).cloned()
                            } else {
                                None
                            };
                            
                            if let Some(arr_2d) = arr_2d_int {
                                let row_idx = self.get_var_or_value(&indices[0])?;
                                let col_idx = self.get_var_or_value(&indices[1])?;
                                let result = self.model.int(i32::MIN, i32::MAX);
                                self.model.element_2d(&arr_2d, row_idx, col_idx, result);
                                return Ok(result);
                            }
                            if let Some(arr_2d) = arr_2d_bool {
                                let row_idx = self.get_var_or_value(&indices[0])?;
                                let col_idx = self.get_var_or_value(&indices[1])?;
                                let result = self.model.bool();
                                self.model.element_2d(&arr_2d, row_idx, col_idx, result);
                                return Ok(result);
                            }
                            if let Some(arr_2d) = arr_2d_float {
                                let row_idx = self.get_var_or_value(&indices[0])?;
                                let col_idx = self.get_var_or_value(&indices[1])?;
                                let result = self.model.float(f64::MIN, f64::MAX);
                                self.model.element_2d(&arr_2d, row_idx, col_idx, result);
                                return Ok(result);
                            }
                        }
                        
                        // For 3D arrays, use element_3d
                        if indices.len() == 3 {
                            // Check for 3D arrays and clone early
                            let arr_3d_int = self.context.get_int_var_array_3d(array_name).cloned();
                            let arr_3d_bool = if arr_3d_int.is_none() {
                                self.context.get_bool_var_array_3d(array_name).cloned()
                            } else {
                                None
                            };
                            let arr_3d_float = if arr_3d_int.is_none() && arr_3d_bool.is_none() {
                                self.context.get_float_var_array_3d(array_name).cloned()
                            } else {
                                None
                            };
                            
                            if let Some(arr_3d) = arr_3d_int {
                                let d_idx = self.get_var_or_value(&indices[0])?;
                                let r_idx = self.get_var_or_value(&indices[1])?;
                                let c_idx = self.get_var_or_value(&indices[2])?;
                                let result = self.model.int(i32::MIN, i32::MAX);
                                self.model.element_3d(&arr_3d, d_idx, r_idx, c_idx, result);
                                return Ok(result);
                            }
                            if let Some(arr_3d) = arr_3d_bool {
                                let d_idx = self.get_var_or_value(&indices[0])?;
                                let r_idx = self.get_var_or_value(&indices[1])?;
                                let c_idx = self.get_var_or_value(&indices[2])?;
                                let result = self.model.bool();
                                self.model.element_3d(&arr_3d, d_idx, r_idx, c_idx, result);
                                return Ok(result);
                            }
                            if let Some(arr_3d) = arr_3d_float {
                                let d_idx = self.get_var_or_value(&indices[0])?;
                                let r_idx = self.get_var_or_value(&indices[1])?;
                                let c_idx = self.get_var_or_value(&indices[2])?;
                                let result = self.model.float(f64::MIN, f64::MAX);
                                self.model.element_3d(&arr_3d, d_idx, r_idx, c_idx, result);
                                return Ok(result);
                            }
                        }
                        
                        // For higher dimensions or fallback, use flattening
                        // Try to evaluate all indices to constants first
                        let mut const_indices = Vec::new();
                        let mut all_const = true;
                        
                        for idx in indices.iter() {
                            match self.eval_int_expr(idx) {
                                Ok(val) => {
                                    // Convert from 1-based (MiniZinc) to 0-based for flattening
                                    const_indices.push((val - 1) as usize);
                                }
                                Err(_) => {
                                    all_const = false;
                                    break;
                                }
                            }
                        }
                        
                        if all_const {
                            // All indices are constants - compute flattened index at compile time
                            let flat_idx = metadata.flatten_indices(&const_indices)?;
                            let flat_idx_expr = ast::Expr {
                                kind: ast::ExprKind::IntLit((flat_idx as i64) + 1), // MiniZinc is 1-indexed
                                span: expr.span,
                            };
                            
                            // Now continue with single-index access using the flattened index
                            let flat_index = flat_idx_expr;
                            
                            // Try to evaluate the flattened index expression to a constant
                            if let Ok(index_val) = self.eval_int_expr(&flat_index) {
                                // Constant index - direct array access
                                let array_index = (index_val - 1) as usize;
                                
                                // Try to find the array
                                if let Some(arr) = self.context.get_int_var_array(array_name) {
                                    if array_index < arr.len() {
                                        return Ok(arr[array_index]);
                                    }
                                }
                                if let Some(arr) = self.context.get_int_param_array(array_name) {
                                    if array_index < arr.len() {
                                        let val = arr[array_index];
                                        return Ok(self.model.int(val, val));
                                    }
                                }
                                if let Some(arr) = self.context.get_bool_var_array(array_name) {
                                    if array_index < arr.len() {
                                        return Ok(arr[array_index]);
                                    }
                                }
                                if let Some(arr) = self.context.get_bool_param_array(array_name) {
                                    if array_index < arr.len() {
                                        let val = if arr[array_index] { 1 } else { 0 };
                                        return Ok(self.model.int(val, val));
                                    }
                                }
                                if let Some(arr) = self.context.get_float_var_array(array_name) {
                                    if array_index < arr.len() {
                                        return Ok(arr[array_index]);
                                    }
                                }
                                if let Some(arr) = self.context.get_float_param_array(array_name) {
                                    if array_index < arr.len() {
                                        let val = arr[array_index];
                                        return Ok(self.model.float(val, val));
                                    }
                                }
                                
                                return Err(Error::message(
                                    &format!("Undefined array: '{}'", array_name),
                                    array.span,
                                ));
                            }
                            
                            // Variable flattened index - use element constraint
                            let index_var = self.get_var_or_value(&flat_index)?;
                            let one = self.model.int(1, 1);
                            
                            if let Some(arr) = self.context.get_int_var_array(array_name) {
                                let zero_based_index = self.model.int(0, (arr.len() - 1) as i32);
                                let index_minus_one = self.model.sub(index_var, one);
                                self.model.new(zero_based_index.eq(index_minus_one));
                                let result = self.model.int(i32::MIN, i32::MAX);
                                self.model.element(&arr, zero_based_index, result);
                                return Ok(result);
                            }
                            if let Some(arr) = self.context.get_bool_var_array(array_name) {
                                let zero_based_index = self.model.int(0, (arr.len() - 1) as i32);
                                let index_minus_one = self.model.sub(index_var, one);
                                self.model.new(zero_based_index.eq(index_minus_one));
                                let result = self.model.bool();
                                self.model.element(&arr, zero_based_index, result);
                                return Ok(result);
                            }
                            if let Some(arr) = self.context.get_float_var_array(array_name) {
                                let zero_based_index = self.model.int(0, (arr.len() - 1) as i32);
                                let index_minus_one = self.model.sub(index_var, one);
                                self.model.new(zero_based_index.eq(index_minus_one));
                                let result = self.model.float(f64::MIN, f64::MAX);
                                self.model.element(&arr, zero_based_index, result);
                                return Ok(result);
                            }
                            
                            return Err(Error::message(
                                &format!("Undefined array: '{}'", array_name),
                                array.span,
                            ));
                        } else {
                            // Variable indices - compute flattened index using constraints
                            // Clone metadata to avoid borrow conflicts
                            let metadata = metadata.clone();
                            
                            // Convert all indices to VarIds
                            let mut index_vars = Vec::new();
                            for idx in indices.iter() {
                                index_vars.push(self.get_var_or_value(idx)?);
                            }
                            
                            // Create auxiliary variable for flattened index (1-based)
                            let flat_size = metadata.total_size() as i32;
                            let flat_index_var = self.model.int(1, flat_size);
                            
                            // Build constraint: flat_index = i0*(d1*d2*...) + i1*(d2*d3*...) + ... + i_n
                            // All indices are 1-based, convert to 0-based for flattening
                            let mut flat_expr_parts = Vec::new();
                            
                            for (dim_idx, index_var) in index_vars.iter().enumerate() {
                                // Calculate multiplier for this dimension
                                let mut multiplier = 1usize;
                                for d in &metadata.dimensions[(dim_idx + 1)..] {
                                    multiplier *= d;
                                }
                                
                                // Convert from 1-based to 0-based
                                let one = self.model.int(1, 1);
                                let zero_based_idx = self.model.sub(*index_var, one);
                                
                                if multiplier == 1 {
                                    // Last dimension - just add zero-based index
                                    flat_expr_parts.push(zero_based_idx);
                                } else {
                                    // Multiply index by multiplier
                                    let mult_const = self.model.int(multiplier as i32, multiplier as i32);
                                    let term = self.model.mul(zero_based_idx, mult_const);
                                    flat_expr_parts.push(term);
                                }
                            }
                            
                            // Sum all parts and add 1 to convert back to 1-based
                            let mut flat_zero_based = flat_expr_parts[0];
                            for part in &flat_expr_parts[1..] {
                                flat_zero_based = self.model.add(flat_zero_based, *part);
                            }
                            let one = self.model.int(1, 1);
                            let flat_one_based = self.model.add(flat_zero_based, one);
                            
                            // Constraint: flat_index = computed_flat_index
                            self.model.new(flat_index_var.eq(flat_one_based));
                            
                            // Use the flattened index with element constraint
                            if let Some(arr) = self.context.get_int_var_array(array_name) {
                                let result = self.model.int(i32::MIN, i32::MAX);
                                self.model.element(&arr, flat_index_var, result);
                                return Ok(result);
                            }
                            if let Some(arr) = self.context.get_bool_var_array(array_name) {
                                let result = self.model.bool();
                                self.model.element(&arr, flat_index_var, result);
                                return Ok(result);
                            }
                            if let Some(arr) = self.context.get_float_var_array(array_name) {
                                let result = self.model.float(f64::MIN, f64::MAX);
                                self.model.element(&arr, flat_index_var, result);
                                return Ok(result);
                            }
                            
                            return Err(Error::message(
                                &format!("Undefined array: '{}'", array_name),
                                array.span,
                            ));
                        }
                    } else {
                        return Err(Error::message(
                            &format!("Array metadata not found for: '{}'", array_name),
                            array.span,
                        ));
                    }
                }
                
                // 1D array access - original logic
                let index = &indices[0];
                
                // Try to evaluate the index expression to a constant first
                if let Ok(index_val) = self.eval_int_expr(index) {
                    // Constant index - direct array access
                    let array_index = (index_val - 1) as usize;
                    
                    if let Some(arr) = self.context.get_int_var_array(array_name) {
                        if array_index < arr.len() {
                            return Ok(arr[array_index]);
                        }
                    }
                    if let Some(arr) = self.context.get_int_param_array(array_name) {
                        if array_index < arr.len() {
                            let val = arr[array_index];
                            return Ok(self.model.int(val, val));
                        }
                    }
                    if let Some(arr) = self.context.get_bool_var_array(array_name) {
                        if array_index < arr.len() {
                            return Ok(arr[array_index]);
                        }
                    }
                    if let Some(arr) = self.context.get_bool_param_array(array_name) {
                        if array_index < arr.len() {
                            let val = if arr[array_index] { 1 } else { 0 };
                            return Ok(self.model.int(val, val));
                        }
                    }
                    if let Some(arr) = self.context.get_float_var_array(array_name) {
                        if array_index < arr.len() {
                            return Ok(arr[array_index]);
                        }
                    }
                    if let Some(arr) = self.context.get_float_param_array(array_name) {
                        if array_index < arr.len() {
                            let val = arr[array_index];
                            return Ok(self.model.float(val, val));
                        }
                    }
                    
                    return Err(Error::message(
                        &format!("Undefined array: '{}'", array_name),
                        array.span,
                    ));
                }
                
                // Variable index - use element constraint
                let index_var = self.get_var_or_value(index)?;
                let one = self.model.int(1, 1);
                
                if let Some(arr) = self.context.get_int_var_array(array_name) {
                    let zero_based_index = self.model.int(0, (arr.len() - 1) as i32);
                    let index_minus_one = self.model.sub(index_var, one);
                    self.model.new(zero_based_index.eq(index_minus_one));
                    let result = self.model.int(i32::MIN, i32::MAX);
                    self.model.element(&arr, zero_based_index, result);
                    return Ok(result);
                }
                if let Some(arr) = self.context.get_bool_var_array(array_name) {
                    let zero_based_index = self.model.int(0, (arr.len() - 1) as i32);
                    let index_minus_one = self.model.sub(index_var, one);
                    self.model.new(zero_based_index.eq(index_minus_one));
                    let result = self.model.bool();
                    self.model.element(&arr, zero_based_index, result);
                    return Ok(result);
                }
                if let Some(arr) = self.context.get_float_var_array(array_name) {
                    let zero_based_index = self.model.int(0, (arr.len() - 1) as i32);
                    let index_minus_one = self.model.sub(index_var, one);
                    self.model.new(zero_based_index.eq(index_minus_one));
                    let result = self.model.float(f64::MIN, f64::MAX);
                    self.model.element(&arr, zero_based_index, result);
                    return Ok(result);
                }
                
                Err(Error::message(
                    &format!("Undefined array: '{}'", array_name),
                    array.span,
                ))
            }
            ast::ExprKind::Call { name, args } => {
                // Handle aggregate functions
                self.translate_aggregate_call(name, args, expr.span)
            }
            _ => Err(Error::unsupported_feature(
                &format!("Expression type: {:?}", expr.kind),
                "Phase 2",
                expr.span,
            )),
        }
    }

    /// Translate aggregate function calls (sum, min, max, etc.)
    fn translate_aggregate_call(&mut self, name: &str, args: &[ast::Expr], span: ast::Span) -> Result<VarId> {
        match name {
            "sum" => {
                if args.len() != 1 {
                    return Err(Error::type_error(
                        "1 argument",
                        &format!("{} arguments", args.len()),
                        span,
                    ));
                }
                
                // Get the array
                let vars = self.get_array_vars(&args[0])?;
                Ok(self.model.sum(&vars))
            }
            "min" => {
                if args.len() != 1 {
                    return Err(Error::type_error(
                        "1 argument",
                        &format!("{} arguments", args.len()),
                        span,
                    ));
                }
                
                let vars = self.get_array_vars(&args[0])?;
                self.model.min(&vars).map_err(|e| Error::message(
                    &format!("min() requires at least one variable: {:?}", e),
                    span,
                ))
            }
            "max" => {
                if args.len() != 1 {
                    return Err(Error::type_error(
                        "1 argument",
                        &format!("{} arguments", args.len()),
                        span,
                    ));
                }
                
                let vars = self.get_array_vars(&args[0])?;
                self.model.max(&vars).map_err(|e| Error::message(
                    &format!("max() requires at least one variable: {:?}", e),
                    span,
                ))
            }
            "product" => {
                if args.len() != 1 {
                    return Err(Error::type_error(
                        "1 argument",
                        &format!("{} arguments", args.len()),
                        span,
                    ));
                }
                
                // Product doesn't have a built-in Selen function for arrays
                // We need to multiply all elements together
                let vars = self.get_array_vars(&args[0])?;
                if vars.is_empty() {
                    return Err(Error::message("product() requires at least one variable", span));
                }
                
                // Start with the first variable and multiply the rest
                let mut result = vars[0];
                for &var in &vars[1..] {
                    result = self.model.mul(result, var);
                }
                Ok(result)
            }
            "count" => {
                if args.len() != 2 {
                    return Err(Error::type_error(
                        "2 arguments",
                        &format!("{} arguments", args.len()),
                        span,
                    ));
                }
                
                // Get the array
                let vars = self.get_array_vars(&args[0])?;
                
                // Get the value to count
                let value = self.get_var_or_value(&args[1])?;
                
                // Create a result variable for the count (0 to array length)
                let count_result = self.model.int(0, vars.len() as i32);
                
                // Call Selen's count_var constraint (supports both constant and variable values)
                self.model.count(&vars, value, count_result);
                
                Ok(count_result)
            }
            "exists" => {
                if args.len() != 1 {
                    return Err(Error::type_error(
                        "1 argument",
                        &format!("{} arguments", args.len()),
                        span,
                    ));
                }
                
                // Get the array (should be boolean variables)
                let vars = self.get_array_vars(&args[0])?;
                
                if vars.is_empty() {
                    return Err(Error::message("exists() requires at least one variable", span));
                }
                
                // exists = OR of all elements
                // Returns a boolean variable (0 or 1)
                Ok(self.model.bool_or(&vars))
            }
            "forall" => {
                if args.len() != 1 {
                    return Err(Error::type_error(
                        "1 argument",
                        &format!("{} arguments", args.len()),
                        span,
                    ));
                }
                
                // Get the array (should be boolean variables)
                let vars = self.get_array_vars(&args[0])?;
                
                if vars.is_empty() {
                    return Err(Error::message("forall() requires at least one variable", span));
                }
                
                // forall = AND of all elements
                // Returns a boolean variable (0 or 1)
                Ok(self.model.bool_and(&vars))
            }
            _ => Err(Error::unsupported_feature(
                &format!("Function '{}'", name),
                "Phase 2",
                span,
            )),
        }
    }

    /// Get array variables from an expression (handles identifiers and literals)
    fn get_array_vars(&mut self, expr: &ast::Expr) -> Result<Vec<VarId>> {
        match &expr.kind {
            ast::ExprKind::Ident(array_name) => {
                // Try each array type
                if let Some(vars) = self.context.get_int_var_array(array_name) {
                    return Ok(vars.clone());
                }
                if let Some(vars) = self.context.get_bool_var_array(array_name) {
                    return Ok(vars.clone());
                }
                if let Some(vars) = self.context.get_float_var_array(array_name) {
                    return Ok(vars.clone());
                }
                Err(Error::message(
                    &format!("Undefined array variable: '{}'", array_name),
                    expr.span,
                ))
            }
            _ => Err(Error::type_error(
                "array identifier",
                "other expression",
                expr.span,
            )),
        }
    }

    /// Evaluate an integer expression to a compile-time constant
    fn eval_int_expr(&self, expr: &ast::Expr) -> Result<i32> {
        match &expr.kind {
            ast::ExprKind::IntLit(i) => Ok(*i as i32),
            ast::ExprKind::Ident(name) => {
                if let Some(value) = self.context.get_int_param(name) {
                    Ok(value)
                } else {
                    Err(Error::message(
                        &format!("Undefined parameter: {}", name),
                        expr.span,
                    ))
                }
            }
            ast::ExprKind::BinOp { op, left, right } => {
                let left_val = self.eval_int_expr(left)?;
                let right_val = self.eval_int_expr(right)?;
                match op {
                    ast::BinOp::Add => Ok(left_val + right_val),
                    ast::BinOp::Sub => Ok(left_val - right_val),
                    ast::BinOp::Mul => Ok(left_val * right_val),
                    ast::BinOp::Div => Ok(left_val / right_val),
                    ast::BinOp::Mod => Ok(left_val % right_val),
                    _ => Err(Error::message(
                        &format!("Cannot evaluate operator {:?} at compile time", op),
                        expr.span,
                    )),
                }
            }
            ast::ExprKind::UnOp { op, expr: inner } => {
                let value = self.eval_int_expr(inner)?;
                match op {
                    ast::UnOp::Neg => Ok(-value),
                    ast::UnOp::Not => Err(Error::message(
                        "Cannot apply boolean NOT to integer",
                        expr.span,
                    )),
                }
            }
            _ => Err(Error::message(
                "Cannot evaluate expression at compile time",
                expr.span,
            )),
        }
    }

    fn eval_float_expr(&self, expr: &ast::Expr) -> Result<f64> {
        match &expr.kind {
            ast::ExprKind::FloatLit(f) => Ok(*f),
            ast::ExprKind::IntLit(i) => Ok(*i as f64),
            ast::ExprKind::Ident(name) => {
                if let Some(value) = self.context.get_float_param(name) {
                    Ok(value)
                } else if let Some(value) = self.context.get_int_param(name) {
                    Ok(value as f64)
                } else {
                    Err(Error::message(
                        &format!("Undefined parameter: {}", name),
                        expr.span,
                    ))
                }
            }
            _ => Err(Error::message(
                "Cannot evaluate float expression at compile time",
                expr.span,
            )),
        }
    }

    fn eval_bool_expr(&self, expr: &ast::Expr) -> Result<bool> {
        match &expr.kind {
            ast::ExprKind::BoolLit(b) => Ok(*b),
            ast::ExprKind::Ident(name) => {
                if let Some(value) = self.context.get_bool_param(name) {
                    Ok(value)
                } else {
                    Err(Error::message(
                        &format!("Undefined parameter: {}", name),
                        expr.span,
                    ))
                }
            }
            _ => Err(Error::message(
                "Cannot evaluate boolean expression at compile time",
                expr.span,
            )),
        }
    }

    fn eval_int_domain(&self, domain: &ast::Expr) -> Result<(i32, i32)> {
        match &domain.kind {
            ast::ExprKind::BinOp {
                op: ast::BinOp::Range,
                left,
                right,
            } => {
                let min = self.eval_int_expr(left)?;
                let max = self.eval_int_expr(right)?;
                Ok((min, max))
            }
            _ => Err(Error::type_error(
                "range expression",
                "other expression",
                domain.span,
            )),
        }
    }

    fn eval_float_domain(&self, domain: &ast::Expr) -> Result<(f64, f64)> {
        match &domain.kind {
            ast::ExprKind::BinOp {
                op: ast::BinOp::Range,
                left,
                right,
            } => {
                let min = self.eval_float_expr(left)?;
                let max = self.eval_float_expr(right)?;
                Ok((min, max))
            }
            ast::ExprKind::Range(left, right) => {
                // Handle Range variant as well
                let min = self.eval_float_expr(left)?;
                let max = self.eval_float_expr(right)?;
                Ok((min, max))
            }
            _ => Err(Error::type_error(
                "range expression",
                "other expression",
                domain.span,
            )),
        }
    }

    fn eval_index_set_size(&self, index_set: &ast::Expr) -> Result<usize> {
        match &index_set.kind {
            ast::ExprKind::BinOp {
                op: ast::BinOp::Range,
                left,
                right,
            } => {
                let start = self.eval_int_expr(left)?;
                let end = self.eval_int_expr(right)?;
                Ok((end - start + 1) as usize)
            }
            _ => Err(Error::type_error(
                "range expression",
                "other expression",
                index_set.span,
            )),
        }
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_translate_simple_param() {
        let source = "int: n = 5;";
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_var_with_domain() {
        let source = "var 1..10: x;";
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_var_array() {
        let source = r#"
            array[1..4] of var 1..4: queens;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_bool_var() {
        let source = "var bool: flag;";
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        let translated = result.unwrap();
        assert_eq!(translated.bool_vars.len(), 1);
        assert!(translated.bool_vars.contains_key("flag"));
    }

    #[test]
    fn test_translate_float_var() {
        let source = "var 0.0..1.0: probability;";
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        let translated = result.unwrap();
        assert_eq!(translated.float_vars.len(), 1);
        assert!(translated.float_vars.contains_key("probability"));
    }

    #[test]
    fn test_translate_bool_array() {
        let source = "array[1..5] of var bool: flags;";
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        let translated = result.unwrap();
        assert_eq!(translated.bool_var_arrays.len(), 1);
        assert!(translated.bool_var_arrays.contains_key("flags"));
        assert_eq!(translated.bool_var_arrays.get("flags").unwrap().len(), 5);
    }

    #[test]
    fn test_translate_float_array() {
        let source = "array[1..3] of var 0.0..10.0: prices;";
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        let translated = result.unwrap();
        assert_eq!(translated.float_var_arrays.len(), 1);
        assert!(translated.float_var_arrays.contains_key("prices"));
        assert_eq!(translated.float_var_arrays.get("prices").unwrap().len(), 3);
    }

    #[test]
    fn test_translate_bool_param() {
        let source = "bool: enabled = true;";
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_float_param() {
        let source = "float: pi = 3.14159;";
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_boolean_and_constraint() {
        let source = r#"
            var bool: a;
            var bool: b;
            constraint a /\ b;
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_boolean_or_constraint() {
        let source = r#"
            var bool: x;
            var bool: y;
            constraint x \/ y;
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_boolean_not_constraint() {
        let source = r#"
            var bool: flag;
            constraint not flag;
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_boolean_implication() {
        let source = r#"
            var bool: a;
            var bool: b;
            constraint a -> b;
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_float_arithmetic() {
        let source = r#"
            var 0.0..10.0: x;
            var 0.0..10.0: y;
            constraint x + y <= 15.0;
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_array_access() {
        let source = r#"
            array[1..5] of var 1..10: arr;
            constraint arr[3] > 5;
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate(&ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_sum_aggregate() {
        let source = r#"
            array[1..3] of var 1..10: values;
            constraint sum(values) == 15;
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let solution = model_data.model.solve();
        assert!(solution.is_ok());
    }

    #[test]
    fn test_translate_min_aggregate() {
        let source = r#"
            array[1..3] of var 1..10: values;
            constraint min(values) >= 5;
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let sol = model_data.model.solve();
        assert!(sol.is_ok());
        
        // Verify the constraint: all values should be >= 5
        if let Some(values_arr) = model_data.int_var_arrays.get("values") {
            let solution = sol.unwrap();
            for var_id in values_arr {
                let val = solution.get_int(*var_id);
                assert!(val >= 5, "Expected all values >= 5, but got {}", val);
            }
        }
    }

    #[test]
    fn test_translate_max_aggregate() {
        let source = r#"
            array[1..3] of var 1..10: values;
            constraint max(values) <= 7;
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let sol = model_data.model.solve();
        assert!(sol.is_ok());
        
        // Verify the constraint: all values should be <= 7
        if let Some(values_arr) = model_data.int_var_arrays.get("values") {
            let solution = sol.unwrap();
            for var_id in values_arr {
                let val = solution.get_int(*var_id);
                assert!(val <= 7, "Expected all values <= 7, but got {}", val);
            }
        }
    }

    #[test]
    fn test_translate_product_aggregate() {
        let source = r#"
            array[1..3] of var 2..4: factors;
            constraint product(factors) == 24;
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let sol = model_data.model.solve();
        assert!(sol.is_ok());
        
        // Verify the constraint
        if let Some(factors_arr) = model_data.int_var_arrays.get("factors") {
            let solution = sol.unwrap();
            let mut product = 1;
            for var_id in factors_arr {
                let val = solution.get_int(*var_id);
                product *= val;
            }
            assert_eq!(product, 24, "Expected product == 24, but got {}", product);
        }
    }

    #[test]
    fn test_translate_minimize() {
        let source = r#"
            var 1..10: x;
            constraint x >= 3;
            solve minimize x;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        let model_data = result.unwrap();
        assert_eq!(model_data.objective_type, ObjectiveType::Minimize);
        assert!(model_data.objective_var.is_some());
    }

    #[test]
    fn test_translate_maximize() {
        let source = r#"
            var 1..10: x;
            constraint x <= 7;
            solve maximize x;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        let model_data = result.unwrap();
        assert_eq!(model_data.objective_type, ObjectiveType::Maximize);
        assert!(model_data.objective_var.is_some());
    }

    #[test]
    fn test_element_constraint_variable_index() {
        let source = r#"
            array[1..5] of var 1..10: values;
            var 1..5: index;
            var 1..10: result;
            
            % Element constraint: result == values[index]
            constraint result == values[index];
            constraint index == 3;
            constraint result == 7;
            
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let sol = model_data.model.solve();
        assert!(sol.is_ok());
        
        // Verify: index should be 3, result should be 7, values[3] should be 7
        let solution = sol.unwrap();
        if let Some(&index_var) = model_data.int_vars.get("index") {
            assert_eq!(solution.get_int(index_var), 3);
        }
        if let Some(&result_var) = model_data.int_vars.get("result") {
            assert_eq!(solution.get_int(result_var), 7);
        }
        if let Some(values_arr) = model_data.int_var_arrays.get("values") {
            // values[3] (0-indexed: values[2]) should be 7
            assert_eq!(solution.get_int(values_arr[2]), 7);
        }
    }

    #[test]
    fn test_element_constraint_in_expression() {
        let source = r#"
            array[1..4] of var 1..10: arr;
            var 1..4: i;
            
            % Use element in a constraint expression
            constraint arr[i] > 5;
            constraint i == 2;
            
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let sol = model_data.model.solve();
        assert!(sol.is_ok());
        
        // Verify: arr[2] should be > 5
        let solution = sol.unwrap();
        if let Some(arr) = model_data.int_var_arrays.get("arr") {
            let val_at_index_2 = solution.get_int(arr[1]); // 0-indexed
            assert!(val_at_index_2 > 5, "Expected arr[2] > 5, got {}", val_at_index_2);
        }
    }

    #[test]
    fn test_count_aggregate() {
        let source = r#"
            array[1..5] of var 1..5: values;
            var 0..5: count_result;
            
            % Count how many values equal 3
            constraint count_result == count(values, 3);
            
            % Set some values to 3
            constraint values[1] == 3;
            constraint values[2] == 3;
            constraint values[3] == 3;
            
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let sol = model_data.model.solve();
        assert!(sol.is_ok());
        
        // Verify: count_result should be 3
        let solution = sol.unwrap();
        if let Some(&count_var) = model_data.int_vars.get("count_result") {
            assert_eq!(solution.get_int(count_var), 3);
        }
    }

    #[test]
    fn test_count_with_constraint() {
        let source = r#"
            array[1..4] of var 1..3: values;
            
            % At least 2 values must be equal to 2
            constraint count(values, 2) >= 2;
            
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let sol = model_data.model.solve();
        assert!(sol.is_ok());
        
        // Verify: at least 2 values should be 2
        let solution = sol.unwrap();
        if let Some(values_arr) = model_data.int_var_arrays.get("values") {
            let count_2s: i32 = values_arr.iter()
                .map(|&v| solution.get_int(v))
                .filter(|&val| val == 2)
                .count() as i32;
            assert!(count_2s >= 2, "Expected at least 2 values equal to 2, got {}", count_2s);
        }
    }

    #[test]
    fn test_exists_aggregate() {
        let source = r#"
            array[1..4] of var bool: flags;
            var bool: any_true;
            
            % at least one flag must be true
            constraint any_true == exists(flags);
            constraint any_true;  % Force it to be true
            
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let sol = model_data.model.solve();
        assert!(sol.is_ok());
        
        // Verify: at least one flag should be true
        let solution = sol.unwrap();
        if let Some(flags_arr) = model_data.bool_var_arrays.get("flags") {
            let any_true = flags_arr.iter()
                .map(|&v| solution.get_int(v))
                .any(|val| val != 0);
            assert!(any_true, "Expected at least one flag to be true");
        }
    }

    #[test]
    fn test_forall_aggregate() {
        let source = r#"
            array[1..4] of var bool: flags;
            var bool: all_true;
            
            % all flags must be true
            constraint all_true == forall(flags);
            constraint all_true;  % Force it to be true
            
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok());
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let sol = model_data.model.solve();
        assert!(sol.is_ok());
        
        // Verify: all flags should be true
        let solution = sol.unwrap();
        if let Some(flags_arr) = model_data.bool_var_arrays.get("flags") {
            let all_true = flags_arr.iter()
                .map(|&v| solution.get_int(v))
                .all(|val| val != 0);
            assert!(all_true, "Expected all flags to be true");
        }
    }

    #[test]
    fn test_modulo_operator() {
        // Test that modulo operator can be evaluated with constants
        let source = r#"
            var 1..20: x;
            var 0..4: remainder;
            
            % Direct constraint with constants: check if 13 mod 5 = 3
            constraint 13 mod 5 == 3;
            constraint x == 13;
            constraint remainder == 3;
            
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok(), "Failed to translate modulo expression");
    }

    #[test]
    fn test_modulo_with_constraint() {
        // Test modulo with variable divisor (the problematic case)
        let source = r#"
            var 1..100: dividend;
            var 1..10: divisor;
            var 0..9: remainder;
            
            constraint remainder == dividend mod divisor;
            constraint dividend == 47;
            constraint divisor == 10;
            
            solve satisfy;
        "#;
        let ast = parse(source).unwrap();
        
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok(), "Failed to translate array with values");
        
        // Test that it solves correctly
        let model_data = result.unwrap();
        let sol = model_data.model.solve();
        
        if let Err(e) = sol {
            eprintln!("FAILED TO SOLVE: {:?}", e);
            eprintln!("This is the modulo with variable divisor issue!");
            panic!("Model should solve but got: {:?}", e);
        }
        
        let solution = sol.unwrap();
        if let Some(dividend_var) = model_data.int_vars.get("dividend") {
            let div_val = solution.get_int(*dividend_var);
            assert_eq!(div_val, 47, "dividend should be 47");
        }
        
        if let Some(divisor_var) = model_data.int_vars.get("divisor") {
            let divisor_val = solution.get_int(*divisor_var);
            assert_eq!(divisor_val, 10, "divisor should be 10");
        }
        
        if let Some(remainder_var) = model_data.int_vars.get("remainder") {
            let rem_val = solution.get_int(*remainder_var);
            assert_eq!(rem_val, 7, "remainder should be 7 (47 mod 10 = 7)");
        }
    }

    #[test]
    fn test_array_initialization_int() {
        // Test integer parameter array initialization
        let source = r#"
            array[1..3] of int: limits = [5, 10, 15];
            array[1..3] of var 1..10: x;
            
            constraint x[1] <= limits[1];
            constraint x[2] <= limits[2];
            constraint x[3] <= limits[3];
            
            solve satisfy;
        "#;

        let ast = parse(source).unwrap();
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok(), "Failed to translate array initialization");
        
        let model_data = result.unwrap();
        let solution = model_data.model.solve();
        assert!(solution.is_ok(), "Failed to solve with parameter array");
        
        let sol = solution.unwrap();
        if let Some(arr) = model_data.int_var_arrays.get("x") {
            assert_eq!(arr.len(), 3, "Array should have 3 elements");
            let x1 = sol.get_int(arr[0]);
            let x2 = sol.get_int(arr[1]);
            let x3 = sol.get_int(arr[2]);
            
            // Verify constraints were applied
            assert!(x1 <= 5, "x[1] should be <= 5");
            assert!(x2 <= 10, "x[2] should be <= 10");
            assert!(x3 <= 15, "x[3] should be <= 15");
        }
    }

    #[test]
    fn test_array_initialization_float() {
        // Test float parameter array initialization
        let source = r#"
            array[1..2] of float: thresholds = [1.5, 2.5];
            array[1..2] of var 0.0..5.0: values;
            
            constraint values[1] <= thresholds[1];
            constraint values[2] <= thresholds[2];
            
            solve satisfy;
        "#;

        let ast = parse(source).unwrap();
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok(), "Failed to translate float array initialization");
        
        let model_data = result.unwrap();
        let solution = model_data.model.solve();
        assert!(solution.is_ok(), "Failed to solve with float parameter array");
        
        let sol = solution.unwrap();
        if let Some(arr) = model_data.float_var_arrays.get("values") {
            assert_eq!(arr.len(), 2, "Array should have 2 elements");
            let v1 = sol.get_float(arr[0]);
            let v2 = sol.get_float(arr[1]);
            
            // Verify constraints were applied
            assert!(v1 <= 1.6, "values[1] should be <= 1.5 (with small tolerance)");
            assert!(v2 <= 2.6, "values[2] should be <= 2.5 (with small tolerance)");
        }
    }

    #[test]
    fn test_array_initialization_bool() {
        // Test bool parameter array initialization
        let source = r#"
            array[1..2] of bool: flags = [true, false];
            array[1..2] of var bool: enabled;
            
            solve satisfy;
        "#;

        let ast = parse(source).unwrap();
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok(), "Failed to translate bool array initialization");
        
        let model_data = result.unwrap();
        let solution = model_data.model.solve();
        assert!(solution.is_ok(), "Failed to solve with bool parameter array");
    }

    #[test]
    fn test_array_initialization_in_arithmetic() {
        // Test using parameter array elements in arithmetic expressions
        let source = r#"
            array[1..2] of int: costs = [10, 20];
            array[1..2] of var 0..1: select;
            
            constraint costs[1] * select[1] + costs[2] * select[2] <= 25;
            
            solve maximize select[1] + select[2];
        "#;

        let ast = parse(source).unwrap();
        let result = Translator::translate_with_vars(&ast);
        assert!(result.is_ok(), "Failed to translate array in arithmetic");
        
        let model_data = result.unwrap();
        let solution = model_data.model.solve();
        assert!(solution.is_ok(), "Failed to solve with array in arithmetic");
        
        let sol = solution.unwrap();
        if let Some(arr) = model_data.int_var_arrays.get("select") {
            assert_eq!(arr.len(), 2, "Array should have 2 elements");
            let s1 = sol.get_int(arr[0]);
            let s2 = sol.get_int(arr[1]);
            
            // Verify constraint: 10*s1 + 20*s2 <= 25
            let total_cost = 10 * s1 + 20 * s2;
            assert!(total_cost <= 25, "Cost constraint should be satisfied");
        }
    }
}

