// ! Selen Model Exporter
//!
//! Exports a FlatZinc model as a standalone Selen Rust program for debugging.
//! 
//! # Architecture
//! 
//! The exporter is organized into modular sections:
//! - Header generation (problem description, imports)
//! - Variable classification (parameter arrays, variable arrays, scalars)
//! - Code generation for each section
//! - Constraint translation
//! - Solver invocation and output

use crate::ast::*;
use crate::error::FlatZincResult;
use std::fs::File;
use std::io::Write;

/// Represents the different types of variable declarations in FlatZinc
#[derive(Debug)]
#[allow(dead_code)]
enum VarCategory<'a> {
    /// Parameter array: constant array of values (e.g., coefficient vectors)
    ParameterArray(&'a VarDecl, Vec<f64>),
    /// Variable array: array of variable references
    VariableArray(&'a VarDecl),
    /// Scalar variable: single variable declaration
    ScalarVariable(&'a VarDecl),
}

/// Export a FlatZinc AST as a standalone Selen Rust program
pub fn export_selen_program(ast: &FlatZincModel, output_path: &str) -> FlatZincResult<()> {
    let mut file = File::create(output_path)?;
    
    // Generate code in sections
    write_header(&mut file, ast)?;
    write_imports(&mut file)?;
    write_main_start(&mut file)?;
    
    // Classify variables into categories
    let (param_arrays, var_arrays, scalar_vars) = classify_variables(&ast.var_decls);
    
    // Write each section
    write_parameter_arrays(&mut file, &param_arrays)?;
    write_scalar_variables(&mut file, &scalar_vars)?;
    write_variable_arrays(&mut file, &var_arrays)?;
    write_constraints(&mut file, &ast.constraints)?;
    
    let objective_var = write_solve_goal(&mut file, &ast.solve_goal)?;
    write_solver_invocation(&mut file, &ast.solve_goal, &objective_var)?;
    write_output_section(&mut file, &scalar_vars, &objective_var)?;
    write_main_end(&mut file)?;
    
    Ok(())
}

/// Write file header with problem description
fn write_header(file: &mut File, ast: &FlatZincModel) -> FlatZincResult<()> {
    writeln!(file, "// Auto-generated Selen test program from FlatZinc")?;
    writeln!(file, "// This program can be compiled and run independently to debug Selen behavior")?;
    writeln!(file, "//")?;
    writeln!(file, "// PROBLEM DESCRIPTION:")?;
    match &ast.solve_goal {
        SolveGoal::Satisfy { .. } => {
            writeln!(file, "//   Type: Satisfaction problem")?;
            writeln!(file, "//   Expected: Find any solution that satisfies all constraints")?;
        }
        SolveGoal::Minimize { objective, .. } => {
            writeln!(file, "//   Type: Minimization problem")?;
            writeln!(file, "//   Objective: minimize {:?}", objective)?;
            writeln!(file, "//   Expected: Find solution with smallest objective value")?;
        }
        SolveGoal::Maximize { objective, .. } => {
            writeln!(file, "//   Type: Maximization problem")?;
            writeln!(file, "//   Objective: maximize {:?}", objective)?;
            writeln!(file, "//   Expected: Find solution with largest objective value")?;
        }
    }
    writeln!(file, "//   Variables: {}", ast.var_decls.len())?;
    writeln!(file, "//   Constraints: {}", ast.constraints.len())?;
    writeln!(file, "//")?;
    writeln!(file, "// NOTE: If all output variables are zero in a maximization problem,")?;
    writeln!(file, "//       this suggests the solver is not optimizing correctly.\n")?;
    Ok(())
}

/// Write import statements
fn write_imports(file: &mut File) -> FlatZincResult<()> {
    writeln!(file, "use selen::prelude::*;")?;
    writeln!(file, "use selen::variables::Val;\n")?;
    Ok(())
}

/// Write main function start and model initialization
fn write_main_start(file: &mut File) -> FlatZincResult<()> {
    writeln!(file, "fn main() {{")?;
    writeln!(file, "    use selen::utils::config::SolverConfig;")?;
    writeln!(file, "    let config = SolverConfig {{")?;
    writeln!(file, "        timeout_ms: Some(300_000), // 5 minute timeout (in milliseconds)")?;
    writeln!(file, "        max_memory_mb: Some(4096), // 4GB memory limit")?;
    writeln!(file, "        ..Default::default()")?;
    writeln!(file, "    }};")?;
    writeln!(file, "    let mut model = Model::with_config(config);\n")?;
    Ok(())
}

/// Classify variable declarations into categories
fn classify_variables(var_decls: &[VarDecl]) -> (Vec<(&VarDecl, Vec<f64>)>, Vec<&VarDecl>, Vec<&VarDecl>) {
    let mut param_arrays: Vec<(&VarDecl, Vec<f64>)> = Vec::new();
    let mut var_arrays: Vec<&VarDecl> = Vec::new();
    let mut scalar_vars = Vec::new();
    
    for var_decl in var_decls {
        if let Type::Array { index_sets: _, element_type: _ } = &var_decl.var_type {
            if let Some(init) = &var_decl.init_value {
                // Check if this is a parameter array (initialized with literals) or variable array (initialized with var refs)
                if let Expr::ArrayLit(elements) = init {
                    // Try to extract as parameter array (all literals)
                    let values: Vec<f64> = elements.iter().filter_map(|e| {
                        match e {
                            Expr::FloatLit(f) => Some(*f),
                            Expr::IntLit(i) => Some(*i as f64),
                            _ => None,
                        }
                    }).collect();
                    if !values.is_empty() && values.len() == elements.len() {
                        // All elements are literals - this is a parameter array
                        param_arrays.push((var_decl, values));
                    } else {
                        // Contains variable references - this is a variable array
                        var_arrays.push(var_decl);
                    }
                } else {
                    var_arrays.push(var_decl);
                }
            } else {
                // Array with no initialization - still needs declaration
                var_arrays.push(var_decl);
            }
        } else {
            scalar_vars.push(var_decl);
        }
    }
    
    (param_arrays, var_arrays, scalar_vars)
}

/// Write parameter array declarations (constant coefficient vectors)
fn write_parameter_arrays(file: &mut File, param_arrays: &[(&VarDecl, Vec<f64>)]) -> FlatZincResult<()> {
    if !param_arrays.is_empty() {
        writeln!(file, "    // ===== PARAMETER ARRAYS =====")?;
        for (decl, values) in param_arrays {
            // Format all values as floats with .0 suffix to avoid type inference issues
            let formatted_values: Vec<String> = values.iter().map(|v| {
                if v.fract() == 0.0 && !v.is_infinite() && !v.is_nan() {
                    // Integer-valued floats get .0 suffix
                    let int_val = *v as i64;
                    format!("{}.0", int_val)
                } else {
                    // Already has decimal point
                    format!("{}", v)
                }
            }).collect();
            writeln!(file, "    let {} = vec![{}]; // {} elements",
                sanitize_name(&decl.name),
                formatted_values.join(", "),
                values.len())?;
        }
        writeln!(file)?;
    }
    Ok(())
}

/// Write scalar variable declarations
fn write_scalar_variables(file: &mut File, scalar_vars: &[&VarDecl]) -> FlatZincResult<()> {
    writeln!(file, "    // ===== VARIABLES =====")?;
    for var_decl in scalar_vars {
        write_variable_declaration(file, var_decl)?;
    }
    writeln!(file)?;
    Ok(())
}

/// Write variable array declarations
fn write_variable_arrays(file: &mut File, var_arrays: &[&VarDecl]) -> FlatZincResult<()> {
    if !var_arrays.is_empty() {
        writeln!(file, "    // ===== VARIABLE ARRAYS =====")?;
        for var_decl in var_arrays {
            write_variable_array_declaration(file, var_decl)?;
        }
        writeln!(file)?;
    }
    Ok(())
}

/// Write all constraints
fn write_constraints(file: &mut File, constraints: &[Constraint]) -> FlatZincResult<()> {
    writeln!(file, "    // ===== CONSTRAINTS ===== ({} total)", constraints.len())?;
    for constraint in constraints {
        write_constraint(file, constraint)?;
    }
    writeln!(file)?;
    Ok(())
}

/// Write solver invocation section
fn write_solver_invocation(file: &mut File, solve_goal: &SolveGoal, objective_var: &Option<String>) -> FlatZincResult<()> {
    writeln!(file, "    // ===== SOLVE =====")?;
    writeln!(file, "    println!(\"Solving...\");")?;
    
    // Choose solve method based on problem type
    match solve_goal {
        SolveGoal::Satisfy { .. } => {
            writeln!(file, "    match model.solve() {{")?;
        }
        SolveGoal::Minimize { .. } => {
            if let Some(obj_var) = objective_var {
                writeln!(file, "    match model.minimize({}) {{", obj_var)?;
            } else {
                writeln!(file, "    match model.solve() {{")?;
            }
        }
        SolveGoal::Maximize { .. } => {
            if let Some(obj_var) = objective_var {
                writeln!(file, "    match model.maximize({}) {{", obj_var)?;
            } else {
                writeln!(file, "    match model.solve() {{")?;
            }
        }
    }
    
    writeln!(file, "        Ok(solution) => {{")?;
    writeln!(file, "            println!(\"\\nSolution found!\");")?;
    writeln!(file, "            println!(\"===================\\n\");")?;
    
    // Print objective value if optimization
    Ok(())
}

/// Write the output section that prints solution variables
fn write_output_section(file: &mut File, scalar_vars: &[&VarDecl], objective_var: &Option<String>) -> FlatZincResult<()> {
    // Print objective value if present
    if let Some(obj_var) = objective_var {
        writeln!(file, "            // OBJECTIVE VALUE")?;
        writeln!(file, "            match solution[{}] {{", obj_var)?;
        writeln!(file, "                Val::ValI(i) => println!(\"  OBJECTIVE = {{}}\", i),")?;
        writeln!(file, "                Val::ValF(f) => println!(\"  OBJECTIVE = {{}}\", f),")?;
        writeln!(file, "            }}")?;
        writeln!(file, "            println!();")?;
    }
    
    // Print output variables only
    writeln!(file, "            // OUTPUT VARIABLES (marked with ::output_var annotation)")?;
    let mut has_output = false;
    for var_decl in scalar_vars {
        if var_decl.annotations.iter().any(|ann| ann.name == "output_var") {
            has_output = true;
            let name = sanitize_name(&var_decl.name);
            writeln!(file, "            match solution[{}] {{", name)?;
            writeln!(file, "                Val::ValI(i) => println!(\"  {} = {{}}\", i),", var_decl.name)?;
            writeln!(file, "                Val::ValF(f) => println!(\"  {} = {{}}\", f),", var_decl.name)?;
            writeln!(file, "            }}")?;
        }
    }
    
    if !has_output {
        writeln!(file, "            // (No output variables found - printing all)")?;
        for var_decl in scalar_vars {
            if let Some(name) = get_var_name(&var_decl.name) {
                writeln!(file, "            match solution[{}] {{", name)?;
                writeln!(file, "                Val::ValI(i) => println!(\"  {} = {{}}\", i),", var_decl.name)?;
                writeln!(file, "                Val::ValF(f) => println!(\"  {} = {{}}\", f),", var_decl.name)?;
                writeln!(file, "            }}")?;
            }
        }
    }
    
    writeln!(file, "        }}")?;
    writeln!(file, "        Err(e) => {{")?;
    writeln!(file, "            println!(\"No solution found: {{:?}}\", e);")?;
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    Ok(())
}

/// Write the closing brace for main function
fn write_main_end(file: &mut File) -> FlatZincResult<()> {
    writeln!(file, "}}")?;
    Ok(())
}

fn write_variable_declaration(file: &mut File, var_decl: &VarDecl) -> FlatZincResult<()> {
    let var_name = sanitize_name(&var_decl.name);
    
    match &var_decl.var_type {
        Type::Var(inner_type) => {
            match **inner_type {
                Type::Bool => {
                    writeln!(file, "    let {} = model.bool(); // {}", var_name, var_decl.name)?;
                }
                Type::Int => {
                    writeln!(file, "    let {} = model.int(i32::MIN, i32::MAX); // {} (unbounded)", 
                        var_name, var_decl.name)?;
                }
                Type::IntRange(min, max) => {
                    writeln!(file, "    let {} = model.int({}, {}); // {} [{}..{}]", 
                        var_name, min, max, var_decl.name, min, max)?;
                }
                Type::IntSet(ref values) => {
                    let min = values.iter().min().unwrap_or(&0);
                    let max = values.iter().max().unwrap_or(&0);
                    writeln!(file, "    let {} = model.int({}, {}); // {} {{{}}}",
                        var_name, min, max, var_decl.name,
                        values.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","))?;
                }
                Type::Float => {
                    writeln!(file, "    let {} = model.float(f64::NEG_INFINITY, f64::INFINITY); // {} (unbounded)", 
                        var_name, var_decl.name)?;
                }
                Type::FloatRange(min, max) => {
                    writeln!(file, "    let {} = model.float({}, {}); // {} [{}..{}]", 
                        var_name, min, max, var_decl.name, min, max)?;
                }
                _ => {
                    writeln!(file, "    // TODO: Unsupported type for {}: {:?}", var_decl.name, inner_type)?;
                }
            }
        }
        _ => {
            writeln!(file, "    // TODO: Unsupported variable type for {}: {:?}", var_decl.name, var_decl.var_type)?;
        }
    }
    
    Ok(())
}

fn write_variable_array_declaration(file: &mut File, var_decl: &VarDecl) -> FlatZincResult<()> {
    // Handle arrays of variables (e.g., array [1..35] of var float: lbutt = [...])
    let array_name = sanitize_name(&var_decl.name);
    
    if let Type::Array { element_type: _, .. } = &var_decl.var_type {
        // Extract the element variable names from initialization
        if let Some(Expr::ArrayLit(elements)) = &var_decl.init_value {
            writeln!(file, "    // Array of variables: {} ({} elements)", var_decl.name, elements.len())?;
            
            // Collect the variable identifiers
            let var_names: Vec<String> = elements.iter()
                .filter_map(|e| {
                    if let Expr::Ident(name) = e {
                        Some(sanitize_name(name))
                    } else {
                        None
                    }
                })
                .collect();
            
            if !var_names.is_empty() {
                writeln!(file, "    let {} = vec![{}];", 
                    array_name,
                    var_names.join(", "))?;
            } else {
                writeln!(file, "    // TODO: Array {} has non-variable elements", var_decl.name)?;
            }
        } else {
            writeln!(file, "    // TODO: Array {} has no initialization", var_decl.name)?;
        }
    } else {
        writeln!(file, "    // TODO: {} is not an array type", var_decl.name)?;
    }
    
    Ok(())
}

fn write_constraint(file: &mut File, constraint: &Constraint) -> FlatZincResult<()> {
    let predicate = &constraint.predicate;
    
    match predicate.as_str() {
        // Float linear constraints
        "float_lin_eq" => write_float_lin_eq(file, &constraint.args)?,
        "float_lin_le" => write_float_lin_le(file, &constraint.args)?,
        "float_lin_ne" => write_float_lin_ne(file, &constraint.args)?,
        "float_lin_eq_reif" => write_float_lin_eq_reif(file, &constraint.args)?,
        "float_lin_le_reif" => write_float_lin_le_reif(file, &constraint.args)?,
        
        // Float comparison constraints - convert to linear form
        "float_eq" | "float_le" | "float_lt" | "float_ne" => 
            write_float_comparison(file, predicate, &constraint.args)?,
        "float_eq_reif" | "float_le_reif" | "float_lt_reif" | "float_ne_reif" =>
            write_float_comparison_reif(file, predicate, &constraint.args)?,
        
        // Integer linear constraints
        "int_lin_eq" => write_int_lin_eq(file, &constraint.args)?,
        "int_lin_le" => write_int_lin_le(file, &constraint.args)?,
        "int_lin_ne" => write_int_lin_ne(file, &constraint.args)?,
        "int_lin_eq_reif" => write_int_lin_eq_reif(file, &constraint.args)?,
        "int_lin_le_reif" => write_int_lin_le_reif(file, &constraint.args)?,
        
        // Integer comparison constraints - convert to linear form
        "int_eq" | "int_le" | "int_lt" | "int_ne" => 
            write_int_comparison(file, predicate, &constraint.args)?,
        "int_eq_reif" | "int_le_reif" | "int_lt_reif" | "int_ne_reif" =>
            write_int_comparison_reif(file, predicate, &constraint.args)?,
        
        _ => {
            writeln!(file, "    // TODO: Unimplemented constraint: {}({} args)", 
                predicate, constraint.args.len())?;
        }
    }
    
    Ok(())
}

fn write_solve_goal(file: &mut File, solve_goal: &SolveGoal) -> FlatZincResult<Option<String>> {
    match solve_goal {
        SolveGoal::Satisfy { .. } => {
            writeln!(file, "    // solve satisfy;")?;
            Ok(None)
        }
        SolveGoal::Minimize { objective, .. } => {
            let obj_var = format_expr(objective);
            writeln!(file, "    // solve minimize {}; - Using Selen's minimize() method", obj_var)?;
            Ok(Some(obj_var))
        }
        SolveGoal::Maximize { objective, .. } => {
            let obj_var = format_expr(objective);
            writeln!(file, "    // solve maximize {}; - Using Selen's maximize() method", obj_var)?;
            Ok(Some(obj_var))
        }
    }
}

// Helper functions to write specific constraint types

fn write_float_lin_eq(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // float_lin_eq([coeffs], [vars], constant)
    if args.len() >= 3 {
        let coeffs = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let constant = format_expr(&args[2]);
        writeln!(file, "    model.lin_eq(&{}, &{}, {});", coeffs, vars, constant)?;
    }
    Ok(())
}

fn write_float_lin_le(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // float_lin_le([coeffs], [vars], constant)
    if args.len() >= 3 {
        let coeffs = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let constant = format_expr(&args[2]);
        writeln!(file, "    model.lin_le(&{}, &{}, {});", coeffs, vars, constant)?;
    }
    Ok(())
}

fn write_float_lin_ne(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() >= 3 {
        let coeffs = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let constant = format_expr(&args[2]);
        writeln!(file, "    model.lin_ne(&{}, &{}, {});", coeffs, vars, constant)?;
    }
    Ok(())
}

fn write_float_lin_eq_reif(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() >= 4 {
        let coeffs = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let constant = format_expr(&args[2]);
        let reif = format_expr(&args[3]);
        writeln!(file, "    model.lin_eq_reif(&{}, &{}, {}, {});", coeffs, vars, constant, reif)?;
    }
    Ok(())
}

fn write_float_lin_le_reif(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() >= 4 {
        let coeffs = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let constant = format_expr(&args[2]);
        let reif = format_expr(&args[3]);
        writeln!(file, "    model.lin_le_reif(&{}, &{}, {}, {});", coeffs, vars, constant, reif)?;
    }
    Ok(())
}

fn write_int_lin_eq(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() >= 3 {
        let coeffs = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let constant = format_expr(&args[2]);
        writeln!(file, "    model.lin_eq(&{}, &{}, {});", coeffs, vars, constant)?;
    }
    Ok(())
}

fn write_int_lin_le(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() >= 3 {
        let coeffs = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let constant = format_expr(&args[2]);
        writeln!(file, "    model.lin_le(&{}, &{}, {});", coeffs, vars, constant)?;
    }
    Ok(())
}

fn write_int_lin_ne(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() >= 3 {
        let coeffs = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let constant = format_expr(&args[2]);
        writeln!(file, "    model.lin_ne(&{}, &{}, {});", coeffs, vars, constant)?;
    }
    Ok(())
}

fn write_int_lin_eq_reif(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() >= 4 {
        let coeffs = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let constant = format_expr(&args[2]);
        let reif = format_expr(&args[3]);
        writeln!(file, "    model.lin_eq_reif(&{}, &{}, {}, {});", coeffs, vars, constant, reif)?;
    }
    Ok(())
}

fn write_int_lin_le_reif(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() >= 4 {
        let coeffs = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let constant = format_expr(&args[2]);
        let reif = format_expr(&args[3]);
        writeln!(file, "    model.lin_le_reif(&{}, &{}, {}, {});", coeffs, vars, constant, reif)?;
    }
    Ok(())
}

fn write_int_comparison(file: &mut File, predicate: &str, args: &[Expr]) -> FlatZincResult<()> {
    // Convert binary comparison to linear constraint
    // Handle cases where one or both args are constants
    if args.len() >= 2 {
        let a_is_const = matches!(&args[0], Expr::IntLit(_));
        let b_is_const = matches!(&args[1], Expr::IntLit(_));
        
        match (a_is_const, b_is_const) {
            (true, false) => {
                // Constant <= Variable: use constraint builder API
                let a_val = match &args[0] {
                    Expr::IntLit(i) => *i,
                    _ => 0,
                };
                let b = format_expr(&args[1]);
                match predicate {
                    "int_le" => writeln!(file, "    model.new({}.ge({}));", b, a_val)?,
                    "int_lt" => writeln!(file, "    model.new({}.gt({}));", b, a_val)?,
                    "int_eq" => writeln!(file, "    model.new({}.eq({}));", b, a_val)?,
                    "int_ne" => writeln!(file, "    model.new({}.ne({}));", b, a_val)?,
                    _ => writeln!(file, "    // TODO: Unsupported int comparison: {}", predicate)?,
                }
            }
            (false, true) => {
                // Variable <= Constant: use constraint builder API
                let a = format_expr(&args[0]);
                let b_val = match &args[1] {
                    Expr::IntLit(i) => *i,
                    _ => 0,
                };
                match predicate {
                    "int_le" => writeln!(file, "    model.new({}.le({}));", a, b_val)?,
                    "int_lt" => writeln!(file, "    model.new({}.lt({}));", a, b_val)?,
                    "int_eq" => writeln!(file, "    model.new({}.eq({}));", a, b_val)?,
                    "int_ne" => writeln!(file, "    model.new({}.ne({}));", a, b_val)?,
                    _ => writeln!(file, "    // TODO: Unsupported int comparison: {}", predicate)?,
                }
            }
            (false, false) => {
                // Variable <= Variable: use constraint builder API
                let a = format_expr(&args[0]);
                let b = format_expr(&args[1]);
                match predicate {
                    "int_le" => writeln!(file, "    model.new({}.le({}));", a, b)?,
                    "int_lt" => writeln!(file, "    model.new({}.lt({}));", a, b)?,
                    "int_eq" => writeln!(file, "    model.new({}.eq({}));", a, b)?,
                    "int_ne" => writeln!(file, "    model.new({}.ne({}));", a, b)?,
                    _ => writeln!(file, "    // TODO: Unsupported int comparison: {}", predicate)?,
                }
            }
            (true, true) => {
                // Both constants
                writeln!(file, "    // WARNING: Both args are constants in {}", predicate)?;
            }
        }
    }
    Ok(())
}

fn write_int_comparison_reif(file: &mut File, predicate: &str, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() >= 3 {
        let a = format_expr(&args[0]);
        let b = format_expr(&args[1]);
        let reif = format_expr(&args[2]);
        match predicate {
            "int_le_reif" => writeln!(file, "    model.lin_le_reif(&vec![1, -1], &vec![{}, {}], 0, {});", a, b, reif)?,
            "int_lt_reif" => writeln!(file, "    model.lin_le_reif(&vec![1, -1], &vec![{}, {}], -1, {});", a, b, reif)?,
            "int_eq_reif" => writeln!(file, "    model.lin_eq_reif(&vec![1, -1], &vec![{}, {}], 0, {});", a, b, reif)?,
            _ => writeln!(file, "    // TODO: Unsupported int comparison reif: {}", predicate)?,
        }
    }
    Ok(())
}

fn write_float_comparison(file: &mut File, predicate: &str, args: &[Expr]) -> FlatZincResult<()> {
    // Convert binary comparison to linear constraint
    // Handle cases where one or both args are constants
    if args.len() >= 2 {
        // Check if first arg is a constant
        let a_is_const = matches!(&args[0], Expr::FloatLit(_) | Expr::IntLit(_));
        let b_is_const = matches!(&args[1], Expr::FloatLit(_) | Expr::IntLit(_));
        
        match (a_is_const, b_is_const) {
            (true, false) => {
                // Constant <= Variable: e.g., -0.0 <= milk means milk >= 0
                // Use simpler constraint builder API: model.new(milk.ge(0.0))
                let a_val = match &args[0] {
                    Expr::FloatLit(f) => *f,
                    Expr::IntLit(i) => *i as f64,
                    _ => 0.0,
                };
                let b = format_expr(&args[1]);
                match predicate {
                    "float_le" => writeln!(file, "    model.new({}.ge({}));", b, format_float_constant(a_val))?,
                    "float_eq" => writeln!(file, "    model.new({}.eq({}));", b, format_float_constant(a_val))?,
                    "float_ne" => writeln!(file, "    model.new({}.ne({}));", b, format_float_constant(a_val))?,
                    _ => writeln!(file, "    // TODO: Unsupported float comparison: {}", predicate)?,
                }
            }
            (false, true) => {
                // Variable <= Constant: e.g., milk <= 10.0
                // Use simpler constraint builder API: model.new(milk.le(10.0))
                let a = format_expr(&args[0]);
                let b_val = match &args[1] {
                    Expr::FloatLit(f) => *f,
                    Expr::IntLit(i) => *i as f64,
                    _ => 0.0,
                };
                match predicate {
                    "float_le" => writeln!(file, "    model.new({}.le({}));", a, format_float_constant(b_val))?,
                    "float_eq" => writeln!(file, "    model.new({}.eq({}));", a, format_float_constant(b_val))?,
                    "float_ne" => writeln!(file, "    model.new({}.ne({}));", a, format_float_constant(b_val))?,
                    _ => writeln!(file, "    // TODO: Unsupported float comparison: {}", predicate)?,
                }
            }
            (false, false) => {
                // Variable <= Variable: e.g., x <= y
                // Use simpler constraint builder API: model.new(x.le(y))
                let a = format_expr(&args[0]);
                let b = format_expr(&args[1]);
                match predicate {
                    "float_le" => writeln!(file, "    model.new({}.le({}));", a, b)?,
                    "float_eq" => writeln!(file, "    model.new({}.eq({}));", a, b)?,
                    "float_ne" => writeln!(file, "    model.new({}.ne({}));", a, b)?,
                    _ => writeln!(file, "    // TODO: Unsupported float comparison: {}", predicate)?,
                }
            }
            (true, true) => {
                // Both constants - this is a tautology or contradiction, but we'll generate it anyway
                writeln!(file, "    // WARNING: Both args are constants in {} - this is likely a tautology/contradiction", predicate)?;
                let a = format_expr(&args[0]);
                let b = format_expr(&args[1]);
                match predicate {
                    "float_le" => writeln!(file, "    // {} <= {}", a, b)?,
                    "float_eq" => writeln!(file, "    // {} == {}", a, b)?,
                    "float_ne" => writeln!(file, "    // {} != {}", a, b)?,
                    _ => writeln!(file, "    // TODO: Unsupported float comparison: {}", predicate)?,
                }
            }
        }
    }
    Ok(())
}

fn write_float_comparison_reif(file: &mut File, predicate: &str, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() >= 3 {
        let a = format_expr(&args[0]);
        let b = format_expr(&args[1]);
        let reif = format_expr(&args[2]);
        match predicate {
            "float_le_reif" => writeln!(file, "    model.lin_le_reif(&vec![1.0, -1.0], &vec![{}, {}], 0.0, {});", a, b, reif)?,
            "float_eq_reif" => writeln!(file, "    model.lin_eq_reif(&vec![1.0, -1.0], &vec![{}, {}], 0.0, {});", a, b, reif)?,
            _ => writeln!(file, "    // TODO: Unsupported float comparison reif: {}", predicate)?,
        }
    }
    Ok(())
}

fn format_float_constant(val: f64) -> String {
    if val.is_infinite() {
        if val.is_sign_positive() {
            "f64::INFINITY".to_string()
        } else {
            "f64::NEG_INFINITY".to_string()
        }
    } else if val.fract() == 0.0 && !val.is_nan() {
        // Ensure integer-valued floats have .0 suffix
        format!("{}.0", val as i64)
    } else {
        format!("{}", val)
    }
}

fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => sanitize_name(name),
        Expr::IntLit(i) => i.to_string(),
        Expr::FloatLit(f) => {
            if f.is_infinite() {
                if f.is_sign_positive() {
                    "f64::INFINITY".to_string()
                } else {
                    "f64::NEG_INFINITY".to_string()
                }
            } else if f.fract() == 0.0 && !f.is_nan() {
                // Ensure integer-valued floats have .0 suffix
                format!("{}.0", *f as i64)
            } else {
                f.to_string()
            }
        }
        Expr::BoolLit(b) => b.to_string(),
        Expr::ArrayLit(elements) => {
            let formatted: Vec<String> = elements.iter().map(format_expr).collect();
            format!("vec![{}]", formatted.join(", "))
        }
        _ => format!("{:?}", expr), // Fallback
    }
}

fn sanitize_name(name: &str) -> String {
    name.replace("::", "_")
        .replace(".", "_")
        .replace("-", "_")
        .replace("[", "_")
        .replace("]", "_")
        .to_lowercase()
}

fn get_var_name(name: &str) -> Option<String> {
    if name.starts_with("X_INTRODUCED") || name.contains("::") {
        return None; // Skip internal variables
    }
    Some(sanitize_name(name))
}
