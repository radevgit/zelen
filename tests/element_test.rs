use zelen::parse;
use zelen::translator::Translator;

fn main() {
    let source = r#"
        array[1..5] of var 1..10: values;
        var 1..5: index;
        var 1..10: result;
        
        % Element constraint: result == values[index]
        constraint result == values[index];
        constraint index == 3;
        constraint result == 7;
        
        solve satisfy;
    "#;
    
    println!("Testing element constraint with variable index...\n");
    println!("MiniZinc Source:\n{}", source);
    
    match parse(source) {
        Ok(ast) => {
            println!("✓ Parsed successfully");
            
            match Translator::translate_with_vars(&ast) {
                Ok(model_data) => {
                    println!("✓ Translated to Selen Model");
                    
                    match model_data.model.solve() {
                        Ok(solution) => {
                            println!("✓ Solution found!\n");
                            
                            if let Some(&index_var) = model_data.int_vars.get("index") {
                                let index_val = solution.get_int(index_var);
                                println!("  index = {}", index_val);
                            }
                            
                            if let Some(&result_var) = model_data.int_vars.get("result") {
                                let result_val = solution.get_int(result_var);
                                println!("  result = {}", result_val);
                            }
                            
                            if let Some(values_arr) = model_data.int_var_arrays.get("values") {
                                print!("  values = [");
                                for (i, &var_id) in values_arr.iter().enumerate() {
                                    if i > 0 {
                                        print!(", ");
                                    }
                                    print!("{}", solution.get_int(var_id));
                                }
                                println!("]");
                                
                                // Check values[3] (0-indexed: values[2])
                                let val_at_3 = solution.get_int(values_arr[2]);
                                println!("  values[3] (MiniZinc 1-indexed) = {}", val_at_3);
                                
                                if val_at_3 == 7 {
                                    println!("\n✓ Element constraint works correctly!");
                                } else {
                                    println!("\n✗ ERROR: values[3] should be 7, but got {}", val_at_3);
                                }
                            }
                        }
                        Err(e) => println!("✗ No solution found: {:?}", e),
                    }
                }
                Err(e) => println!("✗ Translation error: {:?}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
}
