// ! Selen Model Exporter
//!
//! Exports a FlatZinc model as a standalone Selen Rust program for debugging.

use crate::ast::*;
use crate::error::FlatZincResult;
use std::fs::File;
use std::io::Write;

/// Export a FlatZinc AST as a standalone Selen Rust program
pub fn export_selen_program(ast: &FlatZincModel, output_path: &str) -> FlatZincResult<()> {
    let mut file = File::create(output_path)?;
    
    // Write header
    writeln!(file, "// Auto-generated Selen test program from FlatZinc")?;
    writeln!(file, "// This program can be compiled and run independently to debug Selen behavior\n")?;
    writeln!(file, "use selen::prelude::*;")?;
    writeln!(file, "use selen::variables::Val;\n")?;
    writeln!(file, "fn main() {{")?;
    writeln!(file, "    let mut model = Model::default();\n")?;
    
    // Write variables
    writeln!(file, "    // ===== VARIABLES =====")?;
    for var_decl in &ast.var_decls {
        write_variable_declaration(&mut file, var_decl)?;
    }
    writeln!(file)?;
    
    // Write constraints
    writeln!(file, "    // ===== CONSTRAINTS =====")?;
    for constraint in &ast.constraints {
        write_constraint(&mut file, constraint)?;
    }
    writeln!(file)?;
    
    // Write solve goal
    writeln!(file, "    // ===== SOLVE GOAL =====")?;
    write_solve_goal(&mut file, &ast.solve_goal)?;
    writeln!(file)?;
    
    // Write solver invocation
    writeln!(file, "    // ===== SOLVE =====")?;
    writeln!(file, "    match model.solve() {{")?;
    writeln!(file, "        Ok(solution) => {{")?;
    writeln!(file, "            println!(\"Solution found:\");")?;
    
    // Print all variables
    for var_decl in &ast.var_decls {
        if let Some(name) = get_var_name(&var_decl.name) {
            writeln!(file, "            match solution[{}] {{", name)?;
            writeln!(file, "                Val::ValI(i) => println!(\"  {} = {{}}\", i),", var_decl.name)?;
            writeln!(file, "                Val::ValF(f) => println!(\"  {} = {{}}\", f),", var_decl.name)?;
            writeln!(file, "            }}")?;
        }
    }
    
    writeln!(file, "        }}")?;
    writeln!(file, "        Err(e) => {{")?;
    writeln!(file, "            println!(\"No solution: {{:?}}\", e);")?;
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
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
        Type::Array { .. } => {
            // Handle array declarations
            if var_decl.init_value.is_some() {
                // This is a parameter array or variable array with initialization
                writeln!(file, "    // Array parameter: {} (initialization skipped in export)", var_decl.name)?;
            } else {
                writeln!(file, "    // Array variable: {} (TODO: implement array support)", var_decl.name)?;
            }
        }
        _ => {
            writeln!(file, "    // TODO: Unsupported variable type for {}: {:?}", var_decl.name, var_decl.var_type)?;
        }
    }
    
    Ok(())
}

fn write_constraint(file: &mut File, constraint: &Constraint) -> FlatZincResult<()> {
    let predicate = &constraint.predicate;
    
    writeln!(file, "    // {}({})", predicate, 
        constraint.args.iter().map(|_| "...").collect::<Vec<_>>().join(", "))?;
    
    // For now, write a comment with the constraint
    // In a full implementation, we'd generate actual Selen API calls
    writeln!(file, "    // TODO: Implement constraint: {} with {} args", predicate, constraint.args.len())?;
    
    Ok(())
}

fn write_solve_goal(file: &mut File, solve_goal: &SolveGoal) -> FlatZincResult<()> {
    match solve_goal {
        SolveGoal::Satisfy { .. } => {
            writeln!(file, "    // solve satisfy;")?;
        }
        SolveGoal::Minimize { objective, .. } => {
            writeln!(file, "    // solve minimize {:?};", objective)?;
            writeln!(file, "    // TODO: Implement minimization")?;
        }
        SolveGoal::Maximize { objective, .. } => {
            writeln!(file, "    // solve maximize {:?};", objective)?;
            writeln!(file, "    // TODO: Implement maximization")?;
        }
    }
    Ok(())
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
