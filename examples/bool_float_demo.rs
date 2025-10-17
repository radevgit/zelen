/// Example: Boolean and Float Variables
///
/// Demonstrates the new support for boolean and float variable types

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Boolean and Float Variables Demo ===\n");

    // Example 1: Boolean variables
    let bool_source = r#"
        var bool: flag1;
        var bool: flag2;
        constraint flag1 != flag2;
        solve satisfy;
    "#;

    println!("Example 1: Boolean Variables");
    println!("MiniZinc Source:");
    println!("{}", bool_source);
    
    let ast = zelen::parse(bool_source)?;
    let translated = zelen::Translator::translate_with_vars(&ast)?;
    
    println!("✓ Translated to Selen Model");
    println!("  Boolean variables: {:?}", translated.bool_vars.keys().collect::<Vec<_>>());
    
    match translated.model.solve() {
        Ok(solution) => {
            println!("✓ Solution found!");
            for (name, &var_id) in &translated.bool_vars {
                match solution[var_id] {
                    selen::variables::Val::ValI(val) => {
                        println!("  {} = {}", name, if val == 1 { "true" } else { "false" });
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            println!("✗ No solution: {:?}", e);
        }
    }

    println!("\n{}\n", "=".repeat(60));

    // Example 2: Float variables
    let float_source = r#"
        var 0.0..1.0: probability;
        var 0.0..10.0: price;
        constraint probability * price > 5.0;
        solve satisfy;
    "#;

    println!("Example 2: Float Variables with Domains");
    println!("MiniZinc Source:");
    println!("{}", float_source);
    
    let ast = zelen::parse(float_source)?;
    let translated = zelen::Translator::translate_with_vars(&ast)?;
    
    println!("✓ Translated to Selen Model");
    println!("  Float variables: {:?}", translated.float_vars.keys().collect::<Vec<_>>());
    
    match translated.model.solve() {
        Ok(solution) => {
            println!("✓ Solution found!");
            for (name, &var_id) in &translated.float_vars {
                if let selen::variables::Val::ValF(val) = solution[var_id] {
                    println!("  {} = {:.2}", name, val);
                }
            }
        }
        Err(e) => {
            println!("✗ No solution: {:?}", e);
        }
    }

    println!("\n{}\n", "=".repeat(60));

    // Example 3: Boolean array
    let bool_array_source = r#"
        array[1..5] of var bool: flags;
        solve satisfy;
    "#;

    println!("Example 3: Boolean Array");
    println!("MiniZinc Source:");
    println!("{}", bool_array_source);
    
    let ast = zelen::parse(bool_array_source)?;
    let translated = zelen::Translator::translate_with_vars(&ast)?;
    
    println!("✓ Translated to Selen Model");
    println!("  Boolean arrays: {:?}", translated.bool_var_arrays.keys().collect::<Vec<_>>());
    
    match translated.model.solve() {
        Ok(solution) => {
            println!("✓ Solution found!");
            if let Some(flags) = translated.bool_var_arrays.get("flags") {
                print!("  flags = [");
                for (i, var_id) in flags.iter().enumerate() {
                    if i > 0 { print!(", "); }
                    match solution[*var_id] {
                        selen::variables::Val::ValI(val) => {
                            print!("{}", if val == 1 { "true" } else { "false" });
                        }
                        _ => print!("?"),
                    }
                }
                println!("]");
            }
        }
        Err(e) => {
            println!("✗ No solution: {:?}", e);
        }
    }

    println!("\n=== Demo Complete ===");
    
    Ok(())
}
