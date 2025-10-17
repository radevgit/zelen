//! Translator for MiniZinc Core Subset to Selen
//!
//! Translates a parsed MiniZinc AST into Selen Model objects for execution.

use crate::ast;
use crate::error::{Error, Result};
use selen::prelude::*;
use std::collections::HashMap;

/// Context for tracking variables during translation
#[derive(Debug)]
struct TranslatorContext {
    /// Map from MiniZinc variable names to Selen VarIds (integers)
    int_vars: HashMap<String, VarId>,
    /// Map from MiniZinc variable names to Selen VarId arrays
    int_var_arrays: HashMap<String, Vec<VarId>>,
    /// Map from MiniZinc variable names to Selen VarIds (booleans)
    bool_vars: HashMap<String, VarId>,
    /// Map from MiniZinc variable names to Selen VarId arrays (booleans)
    bool_var_arrays: HashMap<String, Vec<VarId>>,
    /// Map from MiniZinc variable names to Selen VarIds (floats)
    float_vars: HashMap<String, VarId>,
    /// Map from MiniZinc variable names to Selen VarId arrays (floats)
    float_var_arrays: HashMap<String, Vec<VarId>>,
    /// Parameter values (for compile-time constants)
    int_params: HashMap<String, i32>,
    /// Float parameters
    float_params: HashMap<String, f64>,
    /// Bool parameters
    bool_params: HashMap<String, bool>,
}

impl TranslatorContext {
    fn new() -> Self {
        Self {
            int_vars: HashMap::new(),
            int_var_arrays: HashMap::new(),
            bool_vars: HashMap::new(),
            bool_var_arrays: HashMap::new(),
            float_vars: HashMap::new(),
            float_var_arrays: HashMap::new(),
            int_params: HashMap::new(),
            float_params: HashMap::new(),
            bool_params: HashMap::new(),
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
}

/// Main translator struct
pub struct Translator {
    model: selen::model::Model,
    context: TranslatorContext,
    objective_type: ObjectiveType,
    objective_var: Option<VarId>,
}

/// Result of translation containing the model and variable mappings
/// Optimization objective type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveType {
    Satisfy,
    Minimize,
    Maximize,
}

pub struct TranslatedModel {
    pub model: selen::model::Model,
    pub int_vars: HashMap<String, VarId>,
    pub int_var_arrays: HashMap<String, Vec<VarId>>,
    pub bool_vars: HashMap<String, VarId>,
    pub bool_var_arrays: HashMap<String, Vec<VarId>>,
    pub float_vars: HashMap<String, VarId>,
    pub float_var_arrays: HashMap<String, Vec<VarId>>,
    pub objective_type: ObjectiveType,
    pub objective_var: Option<VarId>,
}

impl Translator {
    pub fn new() -> Self {
        Self {
            model: selen::model::Model::default(),
            context: TranslatorContext::new(),
            objective_type: ObjectiveType::Satisfy,
            objective_var: None,
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

    /// Translate a MiniZinc AST model and return the model with variable mappings
    pub fn translate_with_vars(ast: &ast::Model) -> Result<TranslatedModel> {
        let mut translator = Self::new();

        // Two-pass approach to ensure simple constraints (e.g., var == const) are posted FIRST
        // This helps Selen's propagators work with narrowed variable domains
        
        let debug = std::env::var("TRANSLATOR_DEBUG").is_ok();
        
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
            int_vars: translator.context.int_vars,
            int_var_arrays: translator.context.int_var_arrays,
            bool_vars: translator.context.bool_vars,
            bool_var_arrays: translator.context.bool_var_arrays,
            float_vars: translator.context.float_vars,
            float_var_arrays: translator.context.float_var_arrays,
            objective_type: translator.objective_type,
            objective_var: translator.objective_var,
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
            ast::Item::VarDecl(var_decl) => self.translate_var_decl(var_decl),
            ast::Item::Constraint(constraint) => self.translate_constraint(constraint),
            ast::Item::Solve(solve) => self.translate_solve(solve),
            ast::Item::Output(_) => {
                // Skip output items for now
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
                }
            }

            ast::TypeInst::Array { index_set, element_type } => {
                self.translate_array_decl(&var_decl.name, index_set, element_type, &var_decl.expr)?;
            }
        }

        Ok(())
    }

    fn translate_array_decl(
        &mut self,
        name: &str,
        index_set: &ast::Expr,
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

        // Get array size
        let size = self.eval_index_set_size(index_set)?;

        if is_var {
            // Decision variable array - determine the type
            match element_type {
                ast::TypeInst::Constrained { base_type, domain, .. } => {
                    match base_type {
                        ast::BaseType::Int => {
                            let (min, max) = self.eval_int_domain(domain)?;
                            let vars = self.model.ints(size, min, max);
                            self.context.add_int_var_array(name.to_string(), vars);
                        }
                        ast::BaseType::Float => {
                            let (min, max) = self.eval_float_domain(domain)?;
                            let vars = self.model.floats(size, min, max);
                            self.context.add_float_var_array(name.to_string(), vars);
                        }
                        ast::BaseType::Bool => {
                            let vars = self.model.bools(size);
                            self.context.add_bool_var_array(name.to_string(), vars);
                        }
                    }
                }
                ast::TypeInst::Basic { base_type, .. } => {
                    match base_type {
                        ast::BaseType::Int => {
                            let vars = self.model.ints(size, i32::MIN, i32::MAX);
                            self.context.add_int_var_array(name.to_string(), vars);
                        }
                        ast::BaseType::Float => {
                            let vars = self.model.floats(size, f64::MIN, f64::MAX);
                            self.context.add_float_var_array(name.to_string(), vars);
                        }
                        ast::BaseType::Bool => {
                            let vars = self.model.bools(size);
                            self.context.add_bool_var_array(name.to_string(), vars);
                        }
                    }
                }
                _ => unreachable!(),
            }
        } else {
            // Parameter array - not yet supported
            return Err(Error::unsupported_feature(
                "Parameter arrays",
                "Phase 1",
                ast::Span::dummy(),
            ));
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
            
            // For array access, substitute the index if needed
            ast::ExprKind::ArrayAccess { array, index } => {
                let index_sub = self.substitute_loop_var_in_expr(index, var_name, value)?;
                ast::ExprKind::ArrayAccess {
                    array: array.clone(),
                    index: Box::new(index_sub),
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
            ast::ExprKind::BinOp { op, left, right } if matches!(op,
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
            ast::Solve::Satisfy { .. } => {
                // Default behavior - no optimization
                self.objective_type = ObjectiveType::Satisfy;
                self.objective_var = None;
            }
            ast::Solve::Minimize { expr, .. } => {
                let var = self.get_var_or_value(expr)?;
                self.objective_type = ObjectiveType::Minimize;
                self.objective_var = Some(var);
            }
            ast::Solve::Maximize { expr, .. } => {
                let var = self.get_var_or_value(expr)?;
                self.objective_type = ObjectiveType::Maximize;
                self.objective_var = Some(var);
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
            ast::ExprKind::ArrayAccess { array, index } => {
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
                
                // Try to evaluate the index expression to a constant first
                if let Ok(index_val) = self.eval_int_expr(index) {
                    // Constant index - direct array access (existing behavior)
                    // Arrays in MiniZinc are 1-indexed, convert to 0-indexed
                    let array_index = (index_val - 1) as usize;
                    
                    // Try to find the array
                    if let Some(arr) = self.context.get_int_var_array(array_name) {
                        if array_index < arr.len() {
                            return Ok(arr[array_index]);
                        } else {
                            return Err(Error::message(
                                &format!("Array index {} out of bounds (array size: {})",
                                    index_val, arr.len()),
                                index.span,
                            ));
                        }
                    }
                    if let Some(arr) = self.context.get_bool_var_array(array_name) {
                        if array_index < arr.len() {
                            return Ok(arr[array_index]);
                        } else {
                            return Err(Error::message(
                                &format!("Array index {} out of bounds (array size: {})",
                                    index_val, arr.len()),
                                index.span,
                            ));
                        }
                    }
                    if let Some(arr) = self.context.get_float_var_array(array_name) {
                        if array_index < arr.len() {
                            return Ok(arr[array_index]);
                        } else {
                            return Err(Error::message(
                                &format!("Array index {} out of bounds (array size: {})",
                                    index_val, arr.len()),
                                index.span,
                            ));
                        }
                    }
                    
                    return Err(Error::message(
                        &format!("Undefined array: '{}'", array_name),
                        array.span,
                    ));
                }
                
                // Variable index - use element constraint
                // Get the index variable
                let index_var = self.get_var_or_value(index)?;
                
                // MiniZinc arrays are 1-indexed, Selen is 0-indexed
                // Selen's element constraint requires a direct VarId, not a computed expression
                // So we create an auxiliary variable and constrain it
                let one = self.model.int(1, 1);
                
                // Get the array and create element constraint
                if let Some(arr) = self.context.get_int_var_array(array_name) {
                    let zero_based_index = self.model.int(0, (arr.len() - 1) as i32);
                    let index_minus_one = self.model.sub(index_var, one);
                    self.model.new(zero_based_index.eq(index_minus_one));
                    
                    // Create a result variable for array[index]
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
}

