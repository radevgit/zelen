/// Example: Solve N-Queens using Zelen translator
///
/// This example demonstrates translating MiniZinc to Selen Model and solving it.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple N-Queens model
    let source = r#"
        int: n = 4;
        array[1..n] of var 1..n: queens;
        constraint alldifferent(queens);
        solve satisfy;
    "#;

    println!("Parsing MiniZinc model...");
    let ast = zelen::parse(source)?;
    println!("Parsed {} items", ast.items.len());

    println!("\nTranslating to Selen Model...");
    let translated = zelen::Translator::translate_with_vars(&ast)?;
    println!("Translation successful!");

    println!("\nSolving...");
    match translated.model.solve() {
        Ok(solution) => {
            println!("✓ Solution found!");
            
            // Extract and display the queens array values
            if let Some(queens) = translated.int_var_arrays.get("queens") {
                print!("\nQueens array: [");
                for (i, var_id) in queens.iter().enumerate() {
                    if i > 0 { print!(", "); }
                    match solution[*var_id] {
                        selen::variables::Val::ValI(val) => print!("{}", val),
                        _ => print!("?"),
                    }
                }
                println!("]");
                
                println!("\nInterpretation: Queen in row i is placed in column queens[i]");
            }
            
            println!("\nSolve stats: {} propagations, {} nodes, {:?}",
                solution.stats.propagation_count,
                solution.stats.node_count,
                solution.stats.solve_time
            );
        }
        Err(e) => {
            println!("No solution found: {:?}", e);
        }
    }

    Ok(())
}
