/// Example: Solve 4-Queens with diagonal constraints
///
/// This demonstrates a fuller N-Queens solution with diagonal constraints

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // N-Queens with all constraints
    // Note: This example currently only has column constraints (alldifferent)
    // Full N-Queens needs diagonal constraints too:
    //   - queens[i] + i != queens[j] + j  (ascending diagonal)
    //   - queens[i] - i != queens[j] - j  (descending diagonal)
    // These will be added when we implement more constraint types
    let source = r#"
        int: n = 4;
        array[1..n] of var 1..n: queens;
        
        % All queens must be in different rows (implicit from domain)
        % All queens must be in different columns
        constraint alldifferent(queens);
        
        % TODO: Add diagonal constraints when supported:
        % constraint forall(i,j in 1..n where i < j)(
        %     queens[i] + i != queens[j] + j /\
        %     queens[i] - i != queens[j] - j
        % );
        
        solve satisfy;
    "#;

    println!("Parsing 4-Queens problem...");
    let ast = zelen::parse(source)?;
    
    println!("Translating to Selen Model...");
    let translated = zelen::Translator::translate_with_vars(&ast)?;
    
    println!("Solving...");
    match translated.model.solve() {
        Ok(solution) => {
            println!("\n✓ Solution found!");
            
            // Extract the queens array
            if let Some(queens) = translated.int_var_arrays.get("queens") {
                println!("\nQueens positions:");
                for (i, var_id) in queens.iter().enumerate() {
                    if let selen::variables::Val::ValI(col) = solution[*var_id] {
                        println!("  Queen {} (row {}) is in column {}", i + 1, i + 1, col);
                    }
                }
                
                println!("\n⚠️  Note: This solution satisfies 'alldifferent' (different columns)");
                println!("   but diagonal constraints are not yet implemented.");
                println!("   All queens on the main diagonal is a valid solution for column-only constraints!");
                
                // Print the board
                println!("\nChessboard (. = empty, Q = queen):");
                for row in 0..4 {
                    print!("  ");
                    for col in 0..4 {
                        let queen_var = queens[row];
                        if let selen::variables::Val::ValI(queen_col) = solution[queen_var] {
                            if queen_col == (col + 1) as i32 {
                                print!("Q ");
                            } else {
                                print!(". ");
                            }
                        }
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            println!("No solution found: {:?}", e);
        }
    }

    Ok(())
}
