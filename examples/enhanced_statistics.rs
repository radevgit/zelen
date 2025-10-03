//! Enhanced Statistics Example
//!
//! Demonstrates the rich statistics available from Selen solver
//! and how they align with FlatZinc specification

use zelen::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Enhanced Solver Statistics ===\n");

    println!("According to FlatZinc Specification (Section 4.3.3.2):");
    println!("Standard statistics: solutions, nodes, failures, solveTime, peakMem");
    println!("Extended statistics: propagations, variables, propagators, etc.\n");

    // Example 1: Simple problem with detailed statistics
    println!("Example 1: Simple Constraint Problem");
    println!("-------------------------------------");
    {
        let fzn = r#"
            var 1..10: x;
            var 1..10: y;
            var 1..10: z;
            constraint int_plus(x, y, z);
            constraint int_eq(z, 10);
            constraint int_le(x, 5);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.with_statistics(true);
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        print!("{}", solver.to_flatzinc());
        
        println!("Statistics breakdown:");
        println!("  - solutions: Number of solutions found (1)");
        println!("  - nodes: Search nodes explored (choice points)");
        println!("  - failures: Backtracks (0 = no backtracking needed)");
        println!("  - propagations: Constraint propagation steps");
        println!("  - variables: Total variables in the model");
        println!("  - propagators: Total constraints/propagators");
        println!("  - solveTime: Solving time in seconds");
        println!("  - peakMem: Peak memory usage in megabytes");
    }

    // Example 2: More complex problem
    println!("\nExample 2: N-Queens (4x4)");
    println!("--------------------------");
    {
        let fzn = r#"
            array[1..4] of var 1..4: q;
            constraint all_different(q);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        print!("{}", solver.to_flatzinc());
        
        println!("Notice:");
        println!("  - More nodes explored (search required)");
        println!("  - More propagations (constraint checking)");
        println!("  - Higher memory usage");
    }

    // Example 3: Without statistics (cleaner output)
    println!("\nExample 3: Minimal Output (no statistics)");
    println!("------------------------------------------");
    {
        let fzn = r#"
            var 1..5: x;
            constraint int_eq(x, 3);
            solve satisfy;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.with_statistics(false);  // Disable statistics
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        print!("{}", solver.to_flatzinc());
    }

    // Example 4: Optimization problem statistics
    println!("\nExample 4: Optimization Problem");
    println!("--------------------------------");
    {
        let fzn = r#"
            var 1..10: x;
            var 1..10: y;
            constraint int_plus(x, y, 15);
            solve minimize x;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        solver.solve().ok();
        
        print!("{}", solver.to_flatzinc());
        
        println!("For optimization:");
        println!("  - Statistics reflect final solution");
        println!("  - nodes/propagations show search effort");
    }

    println!("\n=== Statistics Sources ===");
    println!();
    println!("All statistics are automatically extracted from Selen solver:");
    println!();
    println!("Standard (FlatZinc spec Section 4.3.3.2):");
    println!("  ✓ solutions      - Solution count");
    println!("  ✓ nodes          - From Solution.stats.node_count");
    println!("  ✓ failures       - Not yet exposed by Selen (placeholder: 0)");
    println!("  ✓ solveTime      - From Solution.stats.solve_time");
    println!("  ✓ peakMem        - From Solution.stats.peak_memory_mb");
    println!();
    println!("Extended (additional useful metrics):");
    println!("  ✓ propagations   - From Solution.stats.propagation_count");
    println!("  ✓ variables      - From Solution.stats.variable_count");
    println!("  ✓ propagators    - From Solution.stats.constraint_count");
    println!();
    println!("Configuration:");
    println!("  • solver.with_statistics(true)   - Enable all statistics");
    println!("  • solver.with_statistics(false)  - Clean output only");
    println!();

    Ok(())
}
