/// Example: Simple constraints with comparisons
///
/// Tests binary operators in constraints

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        var 1..10: x;
        var 1..10: y;
        
        constraint x < y;
        constraint x + y < 15;
        
        solve satisfy;
    "#;

    println!("Parsing model with constraints...");
    let ast = zelen::parse(source)?;
    
    println!("Translating to Selen Model...");
    let model = zelen::translate(&ast)?;
    
    println!("Solving...");
    match model.solve() {
        Ok(solution) => {
            println!("\n✓ Solution found!");
            println!("{:?}", solution);
        }
        Err(e) => {
            println!("No solution found: {:?}", e);
        }
    }

    Ok(())
}
