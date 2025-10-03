//! Clean API example - the way it should be!
//!
//! Just call the methods: load → solve → to_flatzinc()

use zelen::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FlatZinc Solver - Clean API ===\n");

    // Example 1: Simple usage
    println!("Example 1: Simple Problem");
    println!("-------------------------");
    {
        let fzn = r#"
            var 1..10: x;
            var 1..10: y;
            constraint int_eq(x, 5);
            constraint int_plus(x, y, 12);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        print!("{}", solver.to_flatzinc());
    }

    // Example 2: Unsatisfiable problem
    println!("\nExample 2: Unsatisfiable");
    println!("------------------------");
    {
        let fzn = r#"
            var 1..5: x;
            constraint int_eq(x, 3);
            constraint int_eq(x, 7);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        print!("{}", solver.to_flatzinc());
    }

    // Example 3: From file
    println!("\nExample 3: N-Queens");
    println!("-------------------");
    {
        let fzn = r#"
            array[1..4] of var 1..4: q;
            constraint all_different(q);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        
        if solver.solve().is_ok() {
            solver.print_flatzinc();
        } else {
            println!("No solution found");
        }
    }

    println!("\n=== API Summary ===");
    println!("1. FlatZincSolver::new()         - create solver");
    println!("2. solver.load_str(fzn)          - load FlatZinc");
    println!("3. solver.solve()                - solve (satisfy/minimize/maximize)");
    println!("4. solver.to_flatzinc()          - get formatted output with statistics");
    println!("5. solver.print_flatzinc()       - print directly");
    println!("\nNo manual formatting, no manual context, no manual anything!");

    Ok(())
}
