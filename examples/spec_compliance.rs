//! FlatZinc Specification Compliance Example
//!
//! This example demonstrates how Zelen aligns with the FlatZinc specification
//! for output formatting and solver behavior.
//!
//! References:
//! - FlatZinc Spec: https://docs.minizinc.dev/en/stable/fzn-spec.html

use zelen::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FlatZinc Specification Compliance ===\n");

    // 1. Standard Output Format (Section 4.3.3.1)
    println!("1. Standard Output Format");
    println!("-------------------------");
    println!("Per spec: Each solution ends with '----------'");
    println!("Search complete ends with '=========='");
    println!();
    {
        let fzn = r#"
            var 1..3: x;
            solve satisfy;
        "#;
        
        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        solver.solve().ok();
        print!("{}", solver.to_flatzinc());
    }

    // 2. Statistics Format (Section 4.3.3.2)
    println!("\n2. Statistics Format (Optional)");
    println!("--------------------------------");
    println!("Per spec: Statistics use format '%%%mzn-stat: name=value'");
    println!("Terminated with '%%%mzn-stat-end'");
    println!();
    println!("With statistics enabled:");
    {
        let fzn = r#"
            var 1..10: x;
            constraint int_eq(x, 5);
            solve satisfy;
        "#;
        
        let mut solver = FlatZincSolver::new();
        solver.with_statistics(true);
        solver.load_str(fzn)?;
        solver.solve().ok();
        print!("{}", solver.to_flatzinc());
    }
    
    println!("\nWithout statistics (cleaner output):");
    {
        let fzn = r#"
            var 1..10: x;
            constraint int_eq(x, 5);
            solve satisfy;
        "#;
        
        let mut solver = FlatZincSolver::new();
        solver.with_statistics(false);
        solver.load_str(fzn)?;
        solver.solve().ok();
        print!("{}", solver.to_flatzinc());
    }

    // 3. Unsatisfiable Problems
    println!("\n3. Unsatisfiable Problems");
    println!("-------------------------");
    println!("Per spec: Output '=====UNSATISFIABLE====='");
    println!();
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

    // 4. Standard Statistics Names (Section 4.3.3.2)
    println!("\n4. Standard Statistics Names");
    println!("----------------------------");
    println!("Per spec, standard statistics include:");
    println!("  - solutions: Number of solutions found");
    println!("  - nodes: Number of search nodes");
    println!("  - failures: Number of failures (backtracks)");
    println!("  - solveTime: Solving time in seconds");
    println!("  - peakMem: Peak memory in Mbytes (optional)");
    println!();
    println!("Zelen outputs: solutions, nodes, failures, solveTime");
    println!();

    // 5. Multiple Solutions (Section 4.3.3.1)
    println!("\n5. Multiple Solutions");
    println!("---------------------");
    println!("Per spec:");
    println!("  -a flag: Find all solutions");
    println!("  -n <i>: Find up to i solutions");
    println!();
    println!("Zelen API:");
    {
        let fzn = r#"
            var 1..2: x;
            solve satisfy;
        "#;
        
        println!("  solver.find_all_solutions()  // Equivalent to -a");
        println!("  solver.max_solutions(3)      // Equivalent to -n 3");
        println!();
        
        let mut solver = FlatZincSolver::new();
        solver.find_all_solutions();
        solver.load_str(fzn)?;
        solver.solve().ok();
        print!("{}", solver.to_flatzinc());
    }

    // 6. Command-Line Equivalents (Section 4.3.5)
    println!("\n6. FlatZinc Solver Standard Flags");
    println!("----------------------------------");
    println!("Standard command-line options and their Zelen equivalents:");
    println!();
    println!("  FlatZinc Flag         Zelen API");
    println!("  -------------         ---------");
    println!("  -a                    solver.find_all_solutions()");
    println!("  -n <i>                solver.max_solutions(i)");
    println!("  -s                    solver.with_statistics(true)");
    println!("  (no -s)               solver.with_statistics(false)");
    println!();

    // 7. Satisfaction vs Optimization
    println!("\n7. Satisfaction vs Optimization");
    println!("--------------------------------");
    println!("Per spec:");
    println!("  - Satisfaction: solve satisfy");
    println!("  - Minimize: solve minimize <var>");
    println!("  - Maximize: solve maximize <var>");
    println!();
    println!("Example - Minimize:");
    {
        let fzn = r#"
            var 1..10: x;
            var 1..10: y;
            constraint int_plus(x, y, 10);
            solve minimize x;
        "#;
        
        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        solver.solve().ok();
        print!("{}", solver.to_flatzinc());
    }

    println!("\n=== Specification Summary ===");
    println!();
    println!("Output Format:");
    println!("  ✓ Variable assignments: varname = value;");
    println!("  ✓ Solution separator: ----------");
    println!("  ✓ Search complete: ==========");
    println!("  ✓ Unsatisfiable: =====UNSATISFIABLE=====");
    println!();
    println!("Statistics (Optional):");
    println!("  ✓ Format: %%%mzn-stat: name=value");
    println!("  ✓ Terminator: %%%mzn-stat-end");
    println!("  ✓ Standard names: solutions, nodes, failures, solveTime");
    println!("  ✓ Configurable via with_statistics(bool)");
    println!();
    println!("Multiple Solutions:");
    println!("  ✓ API ready: find_all_solutions(), max_solutions(n)");
    println!("  ⚠ Pending: Selen backend support for enumeration");
    println!();

    Ok(())
}
