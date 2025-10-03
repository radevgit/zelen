//! Simple usage example showing basic FlatZinc solving
//!
//! This demonstrates the basic usage:
//! 1. Parse FlatZinc string
//! 2. Solve the model
//! 3. Check if solution was found
//!
//! For automatic FlatZinc-formatted output, see `clean_api.rs` which uses FlatZincSolver.

use zelen::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic FlatZinc Usage ===\n");

    // Example 1: Simple satisfaction problem
    println!("Example 1: Satisfaction Problem");
    println!("--------------------------------");
    {
        let fzn = r#"
            var 1..10: x;
            var 1..10: y;
            constraint int_eq(x, 5);
            constraint int_plus(x, y, 12);
            solve satisfy;
        "#;

        let mut model = Model::default();
        let context = model.load_flatzinc_str(fzn)?;

        println!("Parsed {} variables", context.var_names.len());
        
        match model.solve() {
            Ok(_solution) => {
                println!("✓ Solution found!");
            }
            Err(_) => {
                println!("✗ No solution exists");
            }
        }
    }

    // Example 2: Another satisfaction problem
    println!("\nExample 2: Another Problem");
    println!("--------------------------");
    {
        let fzn = r#"
            var 1..5: a;
            var 1..5: b;
            constraint int_ne(a, b);
            solve satisfy;
        "#;

        let mut model = Model::default();
        let context = model.load_flatzinc_str(fzn)?;

        println!("Parsed {} variables", context.var_names.len());

        match model.solve() {
            Ok(_solution) => {
                println!("✓ Solution found!");
            }
            Err(_) => {
                println!("✗ No solution exists");
            }
        }
    }

    // Example 3: Access context information
    println!("\nExample 3: Using Context Information");
    println!("-------------------------------------");
    {
        let fzn = r#"
            array[1..3] of var 1..3: q;
            constraint all_different(q);
            solve satisfy;
        "#;

        let mut model = Model::default();
        let context = model.load_flatzinc_str(fzn)?;

        // Access variable mappings
        println!("Variables defined:");
        for (var_id, name) in &context.var_names {
            println!("  {} -> {:?}", name, var_id);
        }

        // Access array information
        println!("\nArrays defined:");
        for (name, vars) in &context.arrays {
            println!("  {} has {} elements", name, vars.len());
        }

        // Check solve goal
        println!("\nSolve goal: {:?}", context.solve_goal);

        match model.solve() {
            Ok(_solution) => {
                println!("✓ Solution found!");
            }
            Err(_) => {
                println!("✗ No solution exists");
            }
        }
    }

    println!("\nFor automatic FlatZinc-formatted output, use FlatZincSolver:");
    println!("  cargo run --example clean_api");

    Ok(())
}
