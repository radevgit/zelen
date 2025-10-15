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
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::cell::RefCell;

thread_local! {
    static VAR_TO_ARRAY_MAPPING: RefCell<HashMap<String, (String, usize)>> = RefCell::new(HashMap::new());
    static ARRAY_ALIASES: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

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

/// Information about an array of variables that should be created together
#[derive(Debug, Clone)]
struct ArrayVarGroup {
    /// Name of the array variable  
    array_name: String,
    /// Names of individual member variables
    member_names: Vec<String>,
    /// Type of the array elements (to determine int/float/bool and domain)
    element_type: Type,
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
    
    // Analyze variable arrays to find which individual variables are array members
    let original_array_groups = build_array_groups(&var_arrays, &ast.var_decls)?;
    
    // Merge small arrays (< 10 elements) by type to reduce declaration count
    let (merged_groups, merged_original_array_names) = merge_small_arrays(original_array_groups, 10);
    
    // Detect sequential patterns in scalar variables and create synthetic array groups
    let sequential_groups = detect_sequential_patterns(&scalar_vars);
    let mut array_groups = merged_groups;
    array_groups.extend(sequential_groups);
    
    // Build mapping: variable_name -> (array_name, index)
    // Also track: original_array_name -> new_array_name for arrays that share members
    let mut var_to_array: HashMap<String, (String, usize)> = HashMap::new();
    let mut array_aliases: HashMap<String, String> = HashMap::new();
    
    // First pass: build member mappings and track which arrays contain which members
    let mut member_to_arrays: HashMap<String, Vec<String>> = HashMap::new();
    for group in &array_groups {
        let array_name = sanitize_name(&group.array_name);
        for (idx, member_name) in group.member_names.iter().enumerate() {
            var_to_array.insert(member_name.clone(), (array_name.clone(), idx));
            member_to_arrays.entry(member_name.clone()).or_insert_with(Vec::new).push(array_name.clone());
        }
    }
    
    // Second pass: create aliases for FlatZinc arrays that reference merged members
    for var_array_decl in &var_arrays {
        if let Some(Expr::ArrayLit(elements)) = &var_array_decl.init_value {
            // Get the first member to find which merged array it belongs to
            if let Some(Expr::Ident(first_member)) = elements.first() {
                if let Some((target_array, _idx)) = var_to_array.get(first_member) {
                    let original_name = sanitize_name(&var_array_decl.name);
                    // Only create alias if the original array isn't already being declared
                    let processed_names: HashSet<String> = array_groups.iter()
                        .map(|g| sanitize_name(&g.array_name))
                        .collect();
                    if !processed_names.contains(&original_name) {
                        array_aliases.insert(original_name, target_array.clone());
                    }
                }
            }
        }
    }
    
    let array_members: HashSet<String> = array_groups.iter()
        .flat_map(|g| g.member_names.iter().cloned())
        .collect();
    
    // Filter scalar vars to exclude those that are array members
    let true_scalar_vars: Vec<&VarDecl> = scalar_vars.iter()
        .filter(|v| !array_members.contains(&v.name))
        .copied()
        .collect();
    
    // Build set of all array names that were processed in array_groups
    let processed_array_names: HashSet<String> = array_groups.iter()
        .map(|g| g.array_name.clone())
        .collect();
    
    // Don't write vec![...] for arrays that:
    // 1. Were already declared optimally, OR
    // 2. Have all members mapped to other arrays (would cause compile errors)
    let unmerged_var_arrays: Vec<&VarDecl> = var_arrays.iter()
        .filter(|v| {
            if processed_array_names.contains(&v.name) {
                return false;
            }
            // Check if all members are mapped (would reference non-existent variables)
            if let Some(Expr::ArrayLit(elements)) = &v.init_value {
                for elem in elements {
                    if let Expr::Ident(member_name) = elem {
                        if !var_to_array.contains_key(member_name) {
                            return true; // At least one member is not mapped, keep this vec
                        }
                    }
                }
                // All members are mapped, skip this vec
                return false;
            }
            true
        })
        .copied()
        .collect();
    
    // Set the global array alias mapping for constraint writing
    ARRAY_ALIASES.with(|aliases| {
        *aliases.borrow_mut() = array_aliases;
    });
    
    // Write each section
    write_parameter_arrays(&mut file, &param_arrays)?;
    write_array_variables_optimized_no_bindings(&mut file, &array_groups, &ast.var_decls, &var_to_array)?;
    write_scalar_variables(&mut file, &true_scalar_vars)?;
    // Only write vec![...] declarations for arrays that weren't merged
    write_variable_arrays_as_vecs(&mut file, &unmerged_var_arrays)?;
    write_constraints_with_array_mapping(&mut file, &ast.constraints, &var_to_array)?;
    
    let objective_var = write_solve_goal(&mut file, &ast.solve_goal)?;
    write_solver_invocation(&mut file, &ast.solve_goal, &objective_var)?;
    write_output_section(&mut file, &true_scalar_vars, &objective_var)?;
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

/// Build array groups from variable array declarations
/// Each group represents variables that should be created together using model.ints()/floats()/bools()
fn build_array_groups(var_arrays: &[&VarDecl], all_var_decls: &[VarDecl]) -> FlatZincResult<Vec<ArrayVarGroup>> {
    let mut groups = Vec::new();
    
    for var_array_decl in var_arrays {
        // Extract member variable names from the array initialization
        if let Some(Expr::ArrayLit(elements)) = &var_array_decl.init_value {
            let member_names: Vec<String> = elements.iter()
                .filter_map(|e| {
                    if let Expr::Ident(name) = e {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect();
            
            if !member_names.is_empty() {
                // Get the element type from the array type
                if let Type::Array { element_type, .. } = &var_array_decl.var_type {
                    groups.push(ArrayVarGroup {
                        array_name: var_array_decl.name.clone(),
                        member_names,
                        element_type: (**element_type).clone(),
                    });
                }
            }
        }
    }
    
    Ok(groups)
}

/// Merge small array groups by type to reduce declaration count
/// Also deduplicates arrays with identical member lists
/// Returns (merged_groups, set_of_original_array_names_that_were_merged)
fn merge_small_arrays(groups: Vec<ArrayVarGroup>, min_size: usize) -> (Vec<ArrayVarGroup>, HashSet<String>) {
    let mut result_groups = Vec::new();
    let mut small_groups_by_type: std::collections::HashMap<String, Vec<ArrayVarGroup>> = std::collections::HashMap::new();
    let mut merged_original_names = HashSet::new();
    
    // First, deduplicate arrays with identical member lists
    let mut member_signature_map: HashMap<String, ArrayVarGroup> = HashMap::new();
    let mut duplicates: Vec<String> = Vec::new();
    
    for group in groups {
        // Create signature: sorted member names + type
        let mut sorted_members = group.member_names.clone();
        sorted_members.sort();
        let signature = format!("{:?}:{}", sorted_members, format!("{:?}", group.element_type));
        
        if let Some(_existing) = member_signature_map.get(&signature) {
            // This is a duplicate - track it for alias creation
            duplicates.push(group.array_name.clone());
            merged_original_names.insert(group.array_name.clone());
        } else {
            // First occurrence of this signature
            member_signature_map.insert(signature, group);
        }
    }
    
    // Now separate remaining unique arrays into large and small
    for (_sig, group) in member_signature_map {
        if group.member_names.len() >= min_size {
            // Keep large arrays as-is
            result_groups.push(group);
        } else {
            // Small arrays will be merged by type
            let type_key = format!("{:?}", group.element_type);
            merged_original_names.insert(group.array_name.clone());
            small_groups_by_type.entry(type_key).or_insert_with(Vec::new).push(group);
        }
    }
    
    // Merge small groups by type
    for (type_key, small_groups) in small_groups_by_type {
        if small_groups.is_empty() {
            continue;
        }
        
        // Collect all members from small arrays
        let mut all_members = Vec::new();
        for small_group in &small_groups {
            all_members.extend(small_group.member_names.clone());
        }
        
        // Create a merged group
        let type_desc = if type_key.contains("Bool") {
            "bool"
        } else if type_key.contains("Float") {
            "float"
        } else if type_key.contains("Int") {
            "int"
        } else {
            "var"
        };
        
        let merged_name = format!("merged_{}_vars", type_desc);
        result_groups.push(ArrayVarGroup {
            array_name: merged_name,
            member_names: all_members,
            element_type: small_groups[0].element_type.clone(),
        });
    }
    
    (result_groups, merged_original_names)
}

/// Detect variable patterns and group by type/domain
/// Groups ALL variables with the same base name pattern and type together
/// For example: ALL x_introduced_N_ with type bool -> single array
fn detect_sequential_patterns(scalar_vars: &[&VarDecl]) -> Vec<ArrayVarGroup> {
    let mut groups = Vec::new();
    
    // Group variables by type only for very aggressive grouping
    let mut type_groups: std::collections::HashMap<String, Vec<&VarDecl>> = std::collections::HashMap::new();
    
    for var in scalar_vars {
        // Create a type key for grouping - group ALL vars of same type together
        let type_key = format!("{:?}", var.var_type);
        type_groups.entry(type_key).or_insert_with(Vec::new).push(var);
    }
    
    // Convert each type group into an ArrayVarGroup
    for (type_key, mut vars) in type_groups {
        // Create arrays even for small groups to maximize consolidation
        if vars.len() >= 2 {
            // Sort by name for consistent ordering
            vars.sort_by_key(|v| v.name.clone());
            
            let member_names: Vec<String> = vars.iter().map(|v| v.name.clone()).collect();
            
            // Create a descriptive array name based on type and count
            let type_desc = if type_key.contains("Bool") {
                "bool"
            } else if type_key.contains("Float") {
                "float"
            } else if type_key.contains("Int") {
                "int"
            } else {
                "var"
            };
            
            let synthetic_array_name = format!("grouped_{}_array_{}", type_desc, vars.len());
            groups.push(ArrayVarGroup {
                array_name: synthetic_array_name,
                member_names,
                element_type: vars[0].var_type.clone(),
            });
        }
    }
    
    groups
}

/// Extract base name and number from variable name like "X_INTRODUCED_169_" -> ("X_INTRODUCED_", 169)
fn extract_base_and_number(name: &str) -> Option<(String, usize)> {
    // Look for pattern: prefix + digits + optional trailing underscore
    let name_upper = name.to_uppercase();
    
    // Find the last sequence of digits
    let mut last_digit_start = None;
    let mut last_digit_end = None;
    
    let chars: Vec<char> = name_upper.chars().collect();
    for i in 0..chars.len() {
        if chars[i].is_ascii_digit() {
            if last_digit_start.is_none() || (i > 0 && !chars[i-1].is_ascii_digit()) {
                last_digit_start = Some(i);
            }
            last_digit_end = Some(i + 1);
        }
    }
    
    if let (Some(start), Some(end)) = (last_digit_start, last_digit_end) {
        let base = name_upper[..start].to_string();
        let num_str = &name_upper[start..end];
        if let Ok(num) = num_str.parse::<usize>() {
            return Some((base, num));
        }
    }
    
    None
}

/// Check if two types match for grouping purposes
fn types_match(t1: &Type, t2: &Type) -> bool {
    match (t1, t2) {
        (Type::Bool, Type::Bool) => true,
        (Type::IntRange(min1, max1), Type::IntRange(min2, max2)) => min1 == min2 && max1 == max2,
        (Type::FloatRange(min1, max1), Type::FloatRange(min2, max2)) => {
            (min1 - min2).abs() < 1e-10 && (max1 - max2).abs() < 1e-10
        }
        (Type::Var(inner1), Type::Var(inner2)) => types_match(inner1, inner2),
        _ => false,
    }
}

/// Write optimized array variable declarations using model.ints()/floats()/bools()
/// WITHOUT individual bindings - constraints will use array indexing directly
fn write_array_variables_optimized_no_bindings(file: &mut File, array_groups: &[ArrayVarGroup], all_var_decls: &[VarDecl], var_to_array: &HashMap<String, (String, usize)>) -> FlatZincResult<()> {
    if array_groups.is_empty() {
        return Ok(());
    }
    
    // Filter out arrays whose members are all mapped to OTHER arrays (would be duplicates)
    let filtered_groups: Vec<&ArrayVarGroup> = array_groups.iter()
        .filter(|group| {
            let group_name = sanitize_name(&group.array_name);
            // Keep this array if at least one member maps to THIS array (or is not mapped at all)
            group.member_names.iter().any(|member| {
                match var_to_array.get(member) {
                    Some((array_name, _idx)) => array_name == &group_name, // Keep if member maps to this array
                    None => true, // Keep if member not mapped anywhere
                }
            })
        })
        .collect();
    
    if filtered_groups.is_empty() {
        return Ok(());
    }
    
    writeln!(file, "    // ===== ARRAY VARIABLES (optimized) =====")?;
    
    for group in filtered_groups {
        // Look up the first member to get domain information
        if let Some(first_member_name) = group.member_names.first() {
            if let Some(first_member_decl) = all_var_decls.iter().find(|v| &v.name == first_member_name) {
                let array_name = sanitize_name(&group.array_name);
                let n = group.member_names.len();
                
                writeln!(file, "    // Array: {} ({} elements from {} to {})", 
                    group.array_name, n, 
                    group.member_names.first().unwrap(), 
                    group.member_names.last().unwrap())?;
                
                // Generate the appropriate model.ints()/floats()/bools() call based on type
                match &first_member_decl.var_type {
                    Type::IntRange(min, max) => {
                        writeln!(file, "    let {} = model.ints({}, {}, {});", array_name, n, min, max)?;
                    }
                    Type::FloatRange(min, max) => {
                        writeln!(file, "    let {} = model.floats({}, {}, {});", array_name, n, min, max)?;
                    }
                    Type::Bool => {
                        writeln!(file, "    let {} = model.bools({});", array_name, n)?;
                    }
                    Type::Var(inner_type) => {
                        // Unwrap the Var() wrapper
                        match &**inner_type {
                            Type::IntRange(min, max) => {
                                writeln!(file, "    let {} = model.ints({}, {}, {});", array_name, n, min, max)?;
                            }
                            Type::FloatRange(min, max) => {
                                writeln!(file, "    let {} = model.floats({}, {}, {});", array_name, n, min, max)?;
                            }
                            Type::Bool => {
                                writeln!(file, "    let {} = model.bools({});", array_name, n)?;
                            }
                            _ => {
                                writeln!(file, "    // TODO: Unsupported array element type for {}: {:?}", group.array_name, inner_type)?;
                            }
                        }
                    }
                    _ => {
                        writeln!(file, "    // TODO: Unsupported array element type for {}: {:?}", group.array_name, first_member_decl.var_type)?;
                    }
                }
            }
        }
    }
    
    writeln!(file)?;
    Ok(())
}

/// Write parameter array declarations (constant coefficient vectors)
fn write_parameter_arrays(file: &mut File, param_arrays: &[(&VarDecl, Vec<f64>)]) -> FlatZincResult<()> {
    if !param_arrays.is_empty() {
        writeln!(file, "    // ===== PARAMETER ARRAYS =====")?;
        for (decl, values) in param_arrays {
            // Check if this is an int array or float array based on the type
            let is_int_array = matches!(&decl.var_type, 
                Type::Array { element_type, .. } if matches!(&**element_type, Type::Int | Type::IntRange(_, _) | Type::IntSet(_)));
            
            // Format values based on array type
            let formatted_values: Vec<String> = values.iter().map(|v| {
                if is_int_array {
                    // Integer array: format as integers without .0
                    let int_val = *v as i64;
                    format!("{}", int_val)
                } else if v.fract() == 0.0 && !v.is_infinite() && !v.is_nan() {
                    // Float array with integer-valued floats: add .0 suffix
                    let int_val = *v as i64;
                    format!("{}.0", int_val)
                } else {
                    // Float array with non-integer values
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

/// Write variable array declarations as vec![...] (for backwards compatibility)
fn write_variable_arrays_as_vecs(file: &mut File, var_arrays: &[&VarDecl]) -> FlatZincResult<()> {
    if !var_arrays.is_empty() {
        writeln!(file, "    // ===== VARIABLE ARRAYS (as vecs for constraint compatibility) =====")?;
        for var_decl in var_arrays {
            write_variable_array_declaration(file, var_decl)?;
        }
        writeln!(file)?;
    }
    Ok(())
}

/// Write variable array declarations (legacy)
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
fn write_constraints_with_array_mapping(file: &mut File, constraints: &[Constraint], var_to_array: &HashMap<String, (String, usize)>) -> FlatZincResult<()> {
    writeln!(file, "    // ===== CONSTRAINTS ===== ({} total)", constraints.len())?;
    for constraint in constraints {
        write_constraint_with_mapping(file, constraint, var_to_array)?;
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

fn write_constraint_with_mapping(file: &mut File, constraint: &Constraint, var_to_array: &HashMap<String, (String, usize)>) -> FlatZincResult<()> {
    // Set the thread-local mapping for this constraint
    VAR_TO_ARRAY_MAPPING.with(|mapping| {
        *mapping.borrow_mut() = var_to_array.clone();
    });
    
    // Delegate to the regular constraint writer which will now use the mapping via format_expr
    write_constraint_impl(file, constraint)
}

fn write_constraint_impl(file: &mut File, constraint: &Constraint) -> FlatZincResult<()> {
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
        "fzn_int_eq_reif" | "int_eq_reif" | "int_le_reif" | "int_lt_reif" | "int_ne_reif" | 
        "int_eq_imp" | "int_le_imp" | "int_lt_imp" | "int_ne_imp" | 
        "fzn_int_le_reif" | "fzn_int_lt_reif" | "fzn_int_ne_reif" |
        "fzn_int_ge_reif" | "fzn_int_gt_reif" =>
            write_int_comparison_reif(file, predicate, &constraint.args)?,
        
        // Global cardinality constraints
        "fzn_global_cardinality" | "global_cardinality" | "gecode_global_cardinality" =>
            write_global_cardinality(file, &constraint.args)?,
        
        // Element constraints
        "fzn_array_int_element" | "array_int_element" => write_array_int_element(file, &constraint.args)?,
        "fzn_array_var_int_element" | "array_var_int_element" | "gecode_int_element" => write_array_var_int_element(file, &constraint.args)?,
        "fzn_array_bool_element" | "array_bool_element" => write_array_bool_element(file, &constraint.args)?,
        "fzn_array_var_bool_element" | "array_var_bool_element" => write_array_var_bool_element(file, &constraint.args)?,
        
        // Boolean operations
        "fzn_array_bool_and" | "array_bool_and" => write_array_bool_and(file, &constraint.args)?,
        "fzn_array_bool_or" | "array_bool_or" => write_array_bool_or(file, &constraint.args)?,
        "fzn_bool_clause" | "bool_clause" => write_bool_clause(file, &constraint.args)?,
        
        // Cardinality constraints
        "fzn_at_least_int" | "at_least_int" => write_at_least(file, &constraint.args)?,
        "fzn_at_most_int" | "at_most_int" => write_at_most(file, &constraint.args)?,
        "fzn_exactly_int" | "exactly_int" => write_exactly(file, &constraint.args)?,
        
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
            "fzn_int_le_reif" | "int_le_reif" => writeln!(file, "    model.lin_le_reif(&[1, -1], &[{}, {}], 0, {});", a, b, reif)?,
            "fzn_int_lt_reif" | "int_lt_reif" => writeln!(file, "    model.lin_le_reif(&[1, -1], &[{}, {}], -1, {});", a, b, reif)?,
            "fzn_int_eq_reif" | "int_eq_reif" => writeln!(file, "    model.lin_eq_reif(&[1, -1], &[{}, {}], 0, {});", a, b, reif)?,
            "fzn_int_ge_reif" | "int_ge_reif" => writeln!(file, "    model.lin_le_reif(&[-1, 1], &[{}, {}], 0, {});", a, b, reif)?,
            "fzn_int_gt_reif" | "int_gt_reif" => writeln!(file, "    model.lin_le_reif(&[-1, 1], &[{}, {}], -1, {});", a, b, reif)?,
            "fzn_int_ne_reif" | "int_ne_reif" => writeln!(file, "    model.lin_ne_reif(&[1, -1], &[{}, {}], 0, {});", a, b, reif)?,
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
        Expr::Ident(name) => {
            let sanitized = sanitize_name(name);
            // First check if this is an array alias (e.g., x_introduced_73_ -> grouped_int_array_56)
            let maybe_aliased = ARRAY_ALIASES.with(|aliases| {
                let alias_map = aliases.borrow();
                alias_map.get(&sanitized).cloned().unwrap_or(sanitized.clone())
            });
            
            // Then check if this variable is part of an array using thread-local mapping
            VAR_TO_ARRAY_MAPPING.with(|mapping| {
                let map = mapping.borrow();
                if let Some((array_name, idx)) = map.get(name) {
                    format!("{}[{}]", array_name, idx)
                } else {
                    maybe_aliased
                }
            })
        }
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
            // Check if this array contains any literals (constants)
            // If so, wrap them in int()/float()/bool() for Selen
            let has_literals = elements.iter().any(|e| {
                matches!(e, Expr::IntLit(_) | Expr::FloatLit(_) | Expr::BoolLit(_))
            });
            let has_idents = elements.iter().any(|e| {
                matches!(e, Expr::Ident(_))
            });
            let has_mixed_types = has_literals && has_idents;
            
            // Debug: Print first array with mixed types
            if has_mixed_types && elements.len() == 2 {
                eprintln!("DEBUG: Mixed array detected - literals:{} idents:{} elements:{:?}", 
                    has_literals, has_idents, elements);
            }
            
            let formatted: Vec<String> = elements.iter()
                .map(|e| {
                    match e {
                        Expr::IntLit(i) if has_mixed_types => format!("int({})", i),
                        Expr::FloatLit(f) if has_mixed_types => {
                            if f.fract() == 0.0 && !f.is_nan() && !f.is_infinite() {
                                format!("float({}.0)", *f as i64)
                            } else {
                                format!("float({})", f)
                            }
                        }
                        Expr::BoolLit(b) if has_mixed_types => format!("bool({})", b),
                        _ => format_expr(e)
                    }
                })
                .collect();
            format!("vec![{}]", formatted.join(", "))
        }
        _ => format!("{:?}", expr), // Fallback
    }
}

/// Format expression with array variable optimization
fn format_expr_with_arrays(expr: &Expr, var_to_array: &HashMap<String, (String, usize)>) -> String {
    match expr {
        Expr::Ident(name) => translate_var_name(name, var_to_array),
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
            let formatted: Vec<String> = elements.iter().map(|e| format_expr_with_arrays(e, var_to_array)).collect();
            format!("vec![{}]", formatted.join(", "))
        }
        _ => format!("{:?}", expr), // Fallback
    }
}

// Global cardinality constraint
fn write_global_cardinality(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // fzn_global_cardinality(vars, values, counts)
    if args.len() >= 3 {
        let vars = format_expr(&args[0]);
        let values = format_expr(&args[1]);
        let counts = format_expr(&args[2]);
        writeln!(file, "    model.gcc(&{}, &{}, &{});", vars, values, counts)?;
    }
    Ok(())
}

// Element constraints
fn write_array_int_element(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // array_int_element(index_1based, array, value)
    if args.len() >= 3 {
        let index_1based = format_expr(&args[0]);
        let array = format_expr(&args[1]);
        let value = format_expr(&args[2]);
        writeln!(file, "    let index_0based = model.sub({}, selen::variables::Val::ValI(1));", index_1based)?;
        writeln!(file, "    model.elem(&{}, index_0based, {});", array, value)?;
    }
    Ok(())
}

fn write_array_var_int_element(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() == 4 {
        // gecode_int_element(index, offset, array, value)
        let index = format_expr(&args[0]);
        let offset = format_expr(&args[1]);
        let array = format_expr(&args[2]);
        let value = format_expr(&args[3]);
        writeln!(file, "    let index_0based = model.sub({}, {});", index, offset)?;
        writeln!(file, "    model.elem(&{}, index_0based, {});", array, value)?;
    } else if args.len() >= 3 {
        // array_var_int_element(index_1based, array, value)
        let index_1based = format_expr(&args[0]);
        let array = format_expr(&args[1]);
        let value = format_expr(&args[2]);
        writeln!(file, "    let index_0based = model.sub({}, selen::variables::Val::ValI(1));", index_1based)?;
        writeln!(file, "    model.elem(&{}, index_0based, {});", array, value)?;
    }
    Ok(())
}

fn write_array_bool_element(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // array_bool_element(index_1based, array, value)
    if args.len() >= 3 {
        let index_1based = format_expr(&args[0]);
        let array = format_expr(&args[1]);
        let value = format_expr(&args[2]);
        writeln!(file, "    let index_0based = model.sub({}, selen::variables::Val::ValI(1));", index_1based)?;
        writeln!(file, "    model.elem(&{}, index_0based, {});", array, value)?;
    }
    Ok(())
}

fn write_array_var_bool_element(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    if args.len() == 4 {
        // gecode variant with offset
        let index = format_expr(&args[0]);
        let offset = format_expr(&args[1]);
        let array = format_expr(&args[2]);
        let value = format_expr(&args[3]);
        writeln!(file, "    let index_0based = model.sub({}, {});", index, offset)?;
        writeln!(file, "    model.elem(&{}, index_0based, {});", array, value)?;
    } else if args.len() >= 3 {
        // array_var_bool_element(index_1based, array, value)
        let index_1based = format_expr(&args[0]);
        let array = format_expr(&args[1]);
        let value = format_expr(&args[2]);
        writeln!(file, "    let index_0based = model.sub({}, selen::variables::Val::ValI(1));", index_1based)?;
        writeln!(file, "    model.elem(&{}, index_0based, {});", array, value)?;
    }
    Ok(())
}

// Boolean operations
fn write_array_bool_and(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // array_bool_and(vars, result)
    if args.len() >= 2 {
        let vars = format_expr(&args[0]);
        let result = format_expr(&args[1]);
        writeln!(file, "    let and_result = model.bool_and(&{});", vars)?;
        writeln!(file, "    model.new({}.eq(and_result));", result)?;
    }
    Ok(())
}

fn write_array_bool_or(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // array_bool_or(vars, result)
    if args.len() >= 2 {
        let vars = format_expr(&args[0]);
        let result = format_expr(&args[1]);
        writeln!(file, "    let or_result = model.bool_or(&{});", vars)?;
        writeln!(file, "    model.new({}.eq(or_result));", result)?;
    }
    Ok(())
}

fn write_bool_clause(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // bool_clause(pos_lits, neg_lits)
    if args.len() >= 2 {
        let pos = format_expr(&args[0]);
        let neg = format_expr(&args[1]);
        writeln!(file, "    model.bool_clause(&{}, &{});", pos, neg)?;
    }
    Ok(())
}

// Cardinality constraints
fn write_at_least(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // fzn_at_least_int(n, vars, value)
    if args.len() >= 3 {
        let n = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let value = format_expr(&args[2]);
        writeln!(file, "    model.at_least(&{}, {}, {});", vars, value, n)?;
    }
    Ok(())
}

fn write_at_most(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // fzn_at_most_int(n, vars, value)
    if args.len() >= 3 {
        let n = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let value = format_expr(&args[2]);
        writeln!(file, "    model.at_most(&{}, {}, {});", vars, value, n)?;
    }
    Ok(())
}

fn write_exactly(file: &mut File, args: &[Expr]) -> FlatZincResult<()> {
    // fzn_exactly_int(n, vars, value)
    if args.len() >= 3 {
        let n = format_expr(&args[0]);
        let vars = format_expr(&args[1]);
        let value = format_expr(&args[2]);
        writeln!(file, "    model.exactly(&{}, {}, {});", vars, value, n)?;
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

/// Translate a variable name to array access if it's part of an array
/// Returns either "array_name[idx]" or the sanitized variable name
fn translate_var_name(name: &str, var_to_array: &HashMap<String, (String, usize)>) -> String {
    if let Some((array_name, idx)) = var_to_array.get(name) {
        format!("{}[{}]", array_name, idx)
    } else {
        sanitize_name(name)
    }
}

fn get_var_name(name: &str) -> Option<String> {
    if name.starts_with("X_INTRODUCED") || name.contains("::") {
        return None; // Skip internal variables
    }
    Some(sanitize_name(name))
}
