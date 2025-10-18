use std::fs;

fn main() {
    let mzn_code = fs::read_to_string("examples/models/sudoku.mzn")
        .expect("Failed to read sudoku.mzn");
    
    println!("=== Parsing Sudoku Example ===\n");
    
    let ast = match zelen::parse(&mzn_code) {
        Ok(ast) => {
            println!("✓ Parsing successful!");
            println!("AST contains {} items", ast.items.len());
            ast
        }
        Err(e) => {
            println!("✗ Parsing failed: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("\n=== Translating Sudoku Example ===\n");
    
    match zelen::translate(&ast) {
        Ok(_model) => {
            println!("✓ Translation successful!");
            println!("Model created successfully with 2D array constraints");
        }
        Err(e) => {
            println!("✗ Translation failed: {}", e);
            std::process::exit(1);
        }
    };
}
