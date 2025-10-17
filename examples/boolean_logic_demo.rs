use zelen::parse;
use zelen::translator::Translator;

fn main() {
    println!("=== Boolean Logic and Advanced Features Demo ===\n");

    // Example 1: Boolean AND and OR
    example_boolean_and_or();
    println!("\n{}\n", "=".repeat(60));

    // Example 2: Boolean NOT
    example_boolean_not();
    println!("\n{}\n", "=".repeat(60));

    // Example 3: Boolean Implication
    example_boolean_implication();
    println!("\n{}\n", "=".repeat(60));

    // Example 4: Float Arithmetic
    example_float_arithmetic();
    println!("\n{}\n", "=".repeat(60));

    // Example 5: Array Indexing
    example_array_indexing();

    println!("\n=== Demo Complete ===");
}

fn example_boolean_and_or() {
    println!("Example 1: Boolean AND and OR");
    
    let source = r#"
        var bool: lights_on;
        var bool: door_open;
        var bool: alarm_active;
        
        % Alarm is active if lights are on AND door is open
        constraint alarm_active <-> (lights_on /\ door_open);
        
        % At least one safety feature must be active
        constraint lights_on \/ alarm_active;
        
        solve satisfy;
    "#;
    
    println!("MiniZinc Source:\n{}", source);
    
    match parse(source) {
        Ok(ast) => {
            match Translator::translate_with_vars(&ast) {
                Ok(model_data) => {
                    println!("✓ Translated to Selen Model");
                    
                    match model_data.model.solve() {
                        Ok(solution) => {
                            println!("✓ Solution found!");
                            
                            for (name, &var_id) in &model_data.bool_vars {
                                if let selen::variables::Val::ValI(val) = solution[var_id] {
                                    println!("  {} = {}", name, val != 0);
                                }
                            }
                        }
                        Err(e) => println!("✗ No solution: {:?}", e),
                    }
                }
                Err(e) => println!("✗ Translation error: {:?}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
}

fn example_boolean_not() {
    println!("Example 2: Boolean NOT");
    
    let source = r#"
        var bool: system_enabled;
        var bool: maintenance_mode;
        
        % System is enabled only when NOT in maintenance mode
        constraint system_enabled <-> not maintenance_mode;
        
        % Must be in one of the two states
        constraint system_enabled \/ maintenance_mode;
        
        solve satisfy;
    "#;
    
    println!("MiniZinc Source:\n{}", source);
    
    match parse(source) {
        Ok(ast) => {
            match Translator::translate_with_vars(&ast) {
                Ok(model_data) => {
                    println!("✓ Translated to Selen Model");
                    
                    match model_data.model.solve() {
                        Ok(solution) => {
                            println!("✓ Solution found!");
                            
                            for (name, &var_id) in &model_data.bool_vars {
                                if let selen::variables::Val::ValI(val) = solution[var_id] {
                                    println!("  {} = {}", name, val != 0);
                                }
                            }
                        }
                        Err(e) => println!("✗ No solution: {:?}", e),
                    }
                }
                Err(e) => println!("✗ Translation error: {:?}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
}

fn example_boolean_implication() {
    println!("Example 3: Boolean Implication");
    
    let source = r#"
        var bool: raining;
        var bool: umbrella;
        var bool: wet;
        
        % If it's raining and no umbrella, then you get wet
        constraint (raining /\ not umbrella) -> wet;
        
        % If you have umbrella, you don't get wet
        constraint umbrella -> not wet;
        
        % It is raining
        constraint raining;
        
        solve satisfy;
    "#;
    
    println!("MiniZinc Source:\n{}", source);
    
    match parse(source) {
        Ok(ast) => {
            match Translator::translate_with_vars(&ast) {
                Ok(model_data) => {
                    println!("✓ Translated to Selen Model");
                    
                    match model_data.model.solve() {
                        Ok(solution) => {
                            println!("✓ Solution found!");
                            
                            for (name, &var_id) in &model_data.bool_vars {
                                if let selen::variables::Val::ValI(val) = solution[var_id] {
                                    println!("  {} = {}", name, val != 0);
                                }
                            }
                        }
                        Err(e) => println!("✗ No solution: {:?}", e),
                    }
                }
                Err(e) => println!("✗ Translation error: {:?}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
}

fn example_float_arithmetic() {
    println!("Example 4: Float Arithmetic in Constraints");
    
    let source = r#"
        var 0.0..100.0: price;
        var 0.0..1.0: tax_rate;
        var 0.0..150.0: total;
        
        % Total is price plus tax
        constraint total = price + (price * tax_rate);
        
        % Total must be under budget
        constraint total <= 100.0;
        
        % Reasonable tax rate
        constraint tax_rate >= 0.05;
        constraint tax_rate <= 0.20;
        
        solve satisfy;
    "#;
    
    println!("MiniZinc Source:\n{}", source);
    
    match parse(source) {
        Ok(ast) => {
            match Translator::translate_with_vars(&ast) {
                Ok(model_data) => {
                    println!("✓ Translated to Selen Model");
                    
                    match model_data.model.solve() {
                        Ok(solution) => {
                            println!("✓ Solution found!");
                            
                            for (name, &var_id) in &model_data.float_vars {
                                if let selen::variables::Val::ValF(val) = solution[var_id] {
                                    println!("  {} = {:.2}", name, val);
                                }
                            }
                        }
                        Err(e) => println!("✗ No solution: {:?}", e),
                    }
                }
                Err(e) => println!("✗ Translation error: {:?}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
}

fn example_array_indexing() {
    println!("Example 5: Array Indexing in Constraints");
    
    let source = r#"
        array[1..5] of var 1..10: values;
        
        % First element must be less than 5
        constraint values[1] < 5;
        
        % Third element must be greater than 5
        constraint values[3] > 5;
        
        % Fifth element must equal first element plus second
        constraint values[5] = values[1] + values[2];
        
        solve satisfy;
    "#;
    
    println!("MiniZinc Source:\n{}", source);
    
    match parse(source) {
        Ok(ast) => {
            match Translator::translate_with_vars(&ast) {
                Ok(model_data) => {
                    println!("✓ Translated to Selen Model");
                    
                    let model = model_data.model;
                    match model.solve() {
                        Ok(solution) => {
                            println!("✓ Solution found!");
                            
                            // Get the array variable IDs
                            if let Some(values_arr) = model_data.int_var_arrays.get("values") {
                                print!("  values = [");
                                for (i, var_id) in values_arr.iter().enumerate() {
                                    if i > 0 {
                                        print!(", ");
                                    }
                                    print!("{}", solution.get_int(*var_id));
                                }
                                println!("]");
                            }
                        }
                        Err(e) => println!("✗ No solution: {:?}", e),
                    }
                }
                Err(e) => println!("✗ Translation error: {:?}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
}
