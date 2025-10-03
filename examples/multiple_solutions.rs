//! Example: Multiple Solutions and Statistics Configuration
//!
//! Demonstrates:
//! 1. Finding multiple solutions
//! 2. Configuring statistics output
//! 3. Accessing individual solutions

use zelen::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multiple Solutions Example ===\n");

    // Example 1: Single solution (default)
    println!("Example 1: Single Solution (default)");
    println!("-------------------------------------");
    {
        let fzn = r#"
            var 1..3: x;
            var 1..3: y;
            constraint int_plus(x, y, 4);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        println!("Found {} solution(s)", solver.solution_count());
        print!("{}", solver.to_flatzinc());
    }

    // Example 2: Request multiple solutions (up to 3)
    println!("\nExample 2: Find Up To 3 Solutions");
    println!("----------------------------------");
    {
        let fzn = r#"
            var 1..3: x;
            var 1..3: y;
            constraint int_plus(x, y, 4);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.max_solutions(3);  // Request up to 3 solutions
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        println!("Found {} solution(s)", solver.solution_count());
        print!("{}", solver.to_flatzinc());
    }

    // Example 3: Request all solutions
    println!("\nExample 3: Find All Solutions");
    println!("------------------------------");
    {
        let fzn = r#"
            var 1..2: x;
            var 1..2: y;
            constraint int_plus(x, y, 3);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.find_all_solutions();  // Request all solutions
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        println!("Found {} solution(s)", solver.solution_count());
        print!("{}", solver.to_flatzinc());
    }

    // Example 4: Disable statistics
    println!("\nExample 4: Without Statistics");
    println!("------------------------------");
    {
        let fzn = r#"
            var 1..10: x;
            constraint int_eq(x, 5);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.with_statistics(false);  // Disable statistics
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        print!("{}", solver.to_flatzinc());
    }

    // Example 5: Custom solver options
    println!("\nExample 5: Custom Solver Options");
    println!("---------------------------------");
    {
        let fzn = r#"
            var 1..10: x;
            constraint int_eq(x, 7);
            solve satisfy;
        "#;

        let options = SolverOptions {
            find_all_solutions: false,
            max_solutions: Some(1),
            include_statistics: true,
        };

        let mut solver = FlatZincSolver::with_options(options);
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        print!("{}", solver.to_flatzinc());
    }

    // Example 6: Accessing individual solutions
    println!("\nExample 6: Accessing Individual Solutions");
    println!("------------------------------------------");
    {
        let fzn = r#"
            var 1..5: x;
            var 1..5: y;
            constraint int_times(x, y, 6);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        
        if solver.solve().is_ok() {
            println!("Total solutions found: {}", solver.solution_count());
            
            // Access first solution
            if let Some(_solution) = solver.get_solution(0) {
                println!("Successfully retrieved solution 0");
            }
        }
        
        print!("{}", solver.to_flatzinc());
    }

    println!("\n=== Configuration Summary ===");
    println!("1. solver.find_all_solutions()     - Find all solutions");
    println!("2. solver.max_solutions(n)         - Find up to n solutions");
    println!("3. solver.with_statistics(bool)    - Enable/disable statistics");
    println!("4. solver.solution_count()         - Number of solutions found");
    println!("5. solver.get_solution(i)          - Access solution i");
    println!("\nNote: Multiple solution enumeration depends on Selen's capabilities.");
    println!("Currently, Selen's solve() returns a single solution.");

    Ok(())
}
