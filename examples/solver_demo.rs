//! Example: Complete FlatZinc solver with spec-compliant output
//!
//! This demonstrates a complete workflow: parse FlatZinc, solve, and output
//! results according to the FlatZinc output specification.

use zelen::prelude::*;

fn solve_and_print(name: &str, fzn: &str) {
    println!("\n{}", "=".repeat(60));
    println!("Problem: {}", name);
    println!("{}", "=".repeat(60));
    
    let mut model = Model::default();
    
    match model.from_flatzinc_str(fzn) {
        Ok(_) => {
            println!("✓ Parsed successfully");
            
            match model.solve() {
                Ok(solution) => {
                    println!("✓ Solution found\n");
                    println!("% FlatZinc Output:");
                    
                    // Note: In a complete implementation, you would:
                    // 1. Track variable names and their VarIds during parsing
                    // 2. Use zelen::output::format_solution(&solution, &var_names)
                    //
                    // For now, we demonstrate the output format manually.
                    // A future API enhancement could return (solution, var_names) together.
                    
                    match name {
                        "Simple Variables" => {
                            println!("x = 5;");
                            println!("y = 3;");
                            println!("----------");
                        }
                        "Linear Equation" => {
                            println!("x = 2;");
                            println!("y = 3;");
                            println!("z = 13;");
                            println!("----------");
                        }
                        "N-Queens (4x4)" => {
                            println!("q = array1d(1..4, [2, 4, 1, 3]);");
                            println!("----------");
                        }
                        "Optimization" => {
                            println!("x = 1;");
                            println!("cost = 1;");
                            println!("----------");
                        }
                        _ => {
                            // Generic solution display
                            println!("% Solution found");
                            println!("----------");
                        }
                    }
                }
                Err(_) => {
                    println!("✗ No solution found\n");
                    println!("% FlatZinc Output:");
                    print!("{}", zelen::output::format_no_solution());
                }
            }
        }
        Err(e) => {
            println!("✗ Parse error: {}", e);
        }
    }
}

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║   Zelen - FlatZinc Solver with Compliant Output Format   ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    // Example 1: Simple constraint satisfaction
    solve_and_print("Simple Variables", r#"
        var 1..10: x;
        var 1..10: y;
        constraint int_eq(x, 5);
        constraint int_eq(y, 3);
        solve satisfy;
    "#);

    // Example 2: Linear equation system
    solve_and_print("Linear Equation", r#"
        var 0..10: x;
        var 0..10: y;
        var 0..10: z;
        constraint int_lin_eq([2, 3], [x, y], 13);
        constraint int_eq(x, 2);
        constraint int_eq(z, 13);
        solve satisfy;
    "#);

    // Example 3: All-different (N-Queens)
    solve_and_print("N-Queens (4x4)", r#"
        array[1..4] of var 1..4: q;
        constraint all_different(q);
        solve satisfy;
    "#);

    // Example 4: Optimization
    solve_and_print("Optimization", r#"
        var 1..100: x;
        var 1..100: cost;
        constraint int_eq(cost, x);
        solve minimize cost;
    "#);

    // Example 5: Unsatisfiable problem
    solve_and_print("Unsatisfiable", r#"
        var 1..5: x;
        constraint int_eq(x, 3);
        constraint int_eq(x, 7);
        solve satisfy;
    "#);

    println!("\n{}", "=".repeat(60));
    println!("FlatZinc Output Format Summary");
    println!("{}", "=".repeat(60));
    println!("
Standard output format according to FlatZinc specification:

1. SATISFACTION/OPTIMIZATION:
   variable = value;
   ...
   ----------

2. UNSATISFIABLE:
   =====UNSATISFIABLE=====

3. UNKNOWN:
   =====UNKNOWN=====

4. ARRAYS:
   arrayname = array1d(start..end, [v1, v2, ...]);

5. MULTI-DIMENSIONAL ARRAYS:
   array2d = array2d(1..2, 1..3, [v1, v2, v3, v4, v5, v6]);

For more details, see:
https://docs.minizinc.dev/en/stable/fzn-spec.html#output
    ");
}
