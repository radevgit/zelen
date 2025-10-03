//! Test optimization with intermediate solutions

use zelen::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Optimization Test ===\n");

    // Test 1: Minimize with single solution
    println!("Test 1: Minimize (optimal only)");
    println!("--------------------------------");
    {
        let fzn = r#"
            var 1..10: x;
            var 1..10: y;
            var 0..100: cost;
            constraint int_plus(x, y, cost);
            solve minimize cost;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        if solver.solve().is_err() {
            println!("No solution found");
            return Ok(());
        }
        
        println!("Found {} solution(s)", solver.solution_count());
        solver.print_flatzinc();
    }

    // Test 2: Minimize with intermediate solutions
    println!("\nTest 2: Minimize (show intermediate)");
    println!("-------------------------------------");
    {
        let fzn = r#"
            var 1..10: x;
            var 1..10: y;
            var 0..100: cost;
            constraint int_plus(x, y, cost);
            solve minimize cost;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.max_solutions(5);  // Show up to 5 intermediate solutions
        solver.load_str(fzn)?;
        if solver.solve().is_err() {
            println!("No solution found");
            return Ok(());
        }
        
        println!("Found {} solution(s) (including intermediate)", solver.solution_count());
        solver.print_flatzinc();
    }

    // Test 3: Maximize
    println!("\nTest 3: Maximize (optimal only)");
    println!("--------------------------------");
    {
        let fzn = r#"
            var 1..10: x;
            var 1..10: y;
            var 0..100: profit;
            constraint int_plus(x, y, profit);
            solve maximize profit;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.load_str(fzn)?;
        if solver.solve().is_err() {
            println!("No solution found");
            return Ok(());
        }
        
        println!("Found {} solution(s)", solver.solution_count());
        solver.print_flatzinc();
    }

    // Test 4: Maximize with intermediate solutions
    println!("\nTest 4: Maximize (show intermediate)");
    println!("-------------------------------------");
    {
        let fzn = r#"
            var 1..10: x;
            var 1..10: y;
            var 0..100: profit;
            constraint int_plus(x, y, profit);
            solve maximize profit;
        "#;

        let mut solver = FlatZincSolver::new();
        solver.max_solutions(5);  // Show up to 5 intermediate solutions
        solver.load_str(fzn)?;
        if solver.solve().is_err() {
            println!("No solution found");
            return Ok(());
        }
        
        println!("Found {} solution(s) (including intermediate)", solver.solution_count());
        solver.print_flatzinc();
    }

    Ok(())
}
