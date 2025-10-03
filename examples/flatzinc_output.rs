//! Example: FlatZinc-compliant output formatting
//!
//! This example demonstrates the OutputFormatter API and methods for formatting
//! solutions according to the FlatZinc output specification.
//! See: https://docs.minizinc.dev/en/stable/fzn-spec.html#output

use zelen::output::{OutputFormatter, SearchType, SolveStatistics};
use selen::variables::Val;
use std::time::Duration;

fn main() {
    println!("=== FlatZinc Output Formatter API Demo ===\n");

    // Example 1: format_search_complete() - indicates search is done
    println!("Example 1: Search Complete Indicator");
    println!("-------------------------------------");
    {
        let formatter = OutputFormatter::new(SearchType::Satisfy);
        print!("{}", formatter.format_search_complete());
        println!("^ This '==========' indicates the search completed\n");
    }

    // Example 2: format_unsatisfiable() - no solution exists
    println!("Example 2: Unsatisfiable Problem");
    println!("---------------------------------");
    {
        let formatter = OutputFormatter::new(SearchType::Satisfy);
        print!("{}", formatter.format_unsatisfiable());
        println!("^ Output when no solution exists\n");
    }

    // Example 3: format_unknown() - solver couldn't determine
    println!("Example 3: Unknown Status");
    println!("-------------------------");
    {
        let formatter = OutputFormatter::new(SearchType::Satisfy);
        print!("{}", formatter.format_unknown());
        println!("^ Output when solver status is unknown\n");
    }

    // Example 4: format_unbounded() - for optimization problems
    println!("Example 4: Unbounded Optimization");
    println!("----------------------------------");
    {
        let formatter = OutputFormatter::new(SearchType::Minimize);
        print!("{}", formatter.format_unbounded());
        println!("^ Output when optimization problem is unbounded\n");
    }

    // Example 5: format_array() - array variable output
    println!("Example 5: Array Output");
    println!("-----------------------");
    {
        let formatter = OutputFormatter::new(SearchType::Satisfy);
        let values = vec![Val::ValI(2), Val::ValI(4), Val::ValI(1), Val::ValI(3)];
        print!("{}", formatter.format_array("q", (1, 4), &values));
        println!("^ Array formatted according to FlatZinc spec\n");
    }

    // Example 6: format_array_2d() - 2D array output
    println!("Example 6: 2D Array Output");
    println!("--------------------------");
    {
        let formatter = OutputFormatter::new(SearchType::Satisfy);
        let values = vec![
            Val::ValI(1), Val::ValI(2), Val::ValI(3),
            Val::ValI(4), Val::ValI(5), Val::ValI(6),
        ];
        print!("{}", formatter.format_array_2d("grid", (1, 2), (1, 3), &values));
        println!("^ 2D array (2x3) formatted for FlatZinc\n");
    }

    // Example 7: Statistics output
    println!("Example 7: With Statistics");
    println!("--------------------------");
    {
        let stats = SolveStatistics {
            solutions: 1,
            nodes: 42,
            failures: 5,
            propagations: Some(100),
            solve_time: Some(Duration::from_millis(123)),
            peak_memory_mb: Some(2), // 2 MB
            variables: Some(10),
            propagators: Some(5),
        };
        
        let formatter = OutputFormatter::new(SearchType::Satisfy)
            .with_statistics(stats);
        
        print!("{}", formatter.format_search_complete());
        println!("^ Search complete with statistics (%%%mzn-stat comments)\n");
    }

    // Example 8: Complete satisfaction problem workflow
    println!("Example 8: Complete Satisfaction Problem Output");
    println!("------------------------------------------------");
    {
        let formatter = OutputFormatter::new(SearchType::Satisfy);
        
        // In a real scenario, you would use:
        // print!("{}", formatter.format_solution(&solution, &var_names));
        
        // Which would output something like:
        println!("x = 5;");
        println!("y = 7;");
        println!("----------");
        print!("{}", formatter.format_search_complete());
    }

    // Example 9: Multiple solutions (optimization)
    println!("\nExample 9: Optimization with Intermediate Solutions");
    println!("----------------------------------------------------");
    {
        let formatter = OutputFormatter::new(SearchType::Minimize);
        
        // In optimization, each better solution is output:
        println!("x = 10;  % First solution");
        println!("----------");
        println!("x = 5;   % Better solution");
        println!("----------");
        println!("x = 1;   % Best solution found");
        println!("----------");
        print!("{}", formatter.format_search_complete());
        println!();
    }

    println!("\n=== API Summary ===");
    println!("OutputFormatter methods:");
    println!("  • format_solution(solution, var_names) - formats variable assignments");
    println!("  • format_array(name, range, values) - formats 1D array");
    println!("  • format_array_2d(name, r1, r2, values) - formats 2D array");
    println!("  • format_search_complete() - outputs '=========='");
    println!("  • format_unsatisfiable() - outputs '=====UNSATISFIABLE====='");
    println!("  • format_unknown() - outputs '=====UNKNOWN====='");
    println!("  • format_unbounded() - outputs '=====UNBOUNDED====='");
    println!("  • with_statistics(stats) - enables statistics output");
    println!("\nAll methods follow the FlatZinc specification:");
    println!("https://docs.minizinc.dev/en/stable/fzn-spec.html#output");
}
