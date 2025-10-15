//! Translation demonstration - shows MiniZinc → Selen Model → Solve
//!
//! This replaces the old compiler demo which generated string code.
//! The new architecture builds Selen Model objects directly for execution.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MiniZinc Translation & Solving Demo ===\n");

    // Example 1: Simple variable with literal domain
    let simple_var_source = r#"
var 1..10: x;
var 1..10: y;
constraint x < y;
solve satisfy;
"#;

    println!("Example 1: Simple Variables with Constraint");
    println!("MiniZinc Source:");
    println!("{}", simple_var_source);
    
    let ast = zelen::parse(simple_var_source)?;
    let translated = zelen::Translator::translate_with_vars(&ast)?;
    
    println!("✓ Translated to Selen Model");
    println!("  Variables: {:?}", translated.int_vars.keys().collect::<Vec<_>>());
    
    match translated.model.solve() {
        Ok(solution) => {
            println!("✓ Solution found!");
            if let Some(&x) = translated.int_vars.get("x") {
                if let Some(&y) = translated.int_vars.get("y") {
                    if let selen::variables::Val::ValI(x_val) = solution[x] {
                        if let selen::variables::Val::ValI(y_val) = solution[y] {
                            println!("  x = {}, y = {}", x_val, y_val);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ No solution: {:?}", e);
        }
    }

    println!("\n{}\n", "=".repeat(60));

    // Example 2: Parameter and variable
    let param_source = r#"
int: n = 5;
var 1..n: x;
solve satisfy;
"#;

    println!("Example 2: Parameter with Expression");
    println!("MiniZinc Source:");
    println!("{}", param_source);
    
    let ast = zelen::parse(param_source)?;
    let translated = zelen::Translator::translate_with_vars(&ast)?;
    
    println!("✓ Translated to Selen Model");
    println!("  Parameters: n = 5");
    println!("  Variables: {:?}", translated.int_vars.keys().collect::<Vec<_>>());
    
    match translated.model.solve() {
        Ok(solution) => {
            println!("✓ Solution found!");
            if let Some(&x) = translated.int_vars.get("x") {
                if let selen::variables::Val::ValI(x_val) = solution[x] {
                    println!("  x = {} (domain was 1..5)", x_val);
                }
            }
        }
        Err(e) => {
            println!("✗ No solution: {:?}", e);
        }
    }

    println!("\n{}\n", "=".repeat(60));

    // Example 3: Array with constraint
    let array_source = r#"
array[1..4] of var 1..4: queens;
constraint alldifferent(queens);
solve satisfy;
"#;

    println!("Example 3: N-Queens Array with Alldifferent");
    println!("MiniZinc Source:");
    println!("{}", array_source);
    
    let ast = zelen::parse(array_source)?;
    let translated = zelen::Translator::translate_with_vars(&ast)?;
    
    println!("✓ Translated to Selen Model");
    println!("  Variable arrays: {:?}", translated.int_var_arrays.keys().collect::<Vec<_>>());
    
    match translated.model.solve() {
        Ok(solution) => {
            println!("✓ Solution found!");
            if let Some(queens) = translated.int_var_arrays.get("queens") {
                print!("  queens = [");
                for (i, var_id) in queens.iter().enumerate() {
                    if i > 0 { print!(", "); }
                    if let selen::variables::Val::ValI(val) = solution[*var_id] {
                        print!("{}", val);
                    }
                }
                println!("]");
            }
        }
        Err(e) => {
            println!("✗ No solution: {:?}", e);
        }
    }

    println!("\n=== Translation & Solving Complete ===");
    
    Ok(())
}
