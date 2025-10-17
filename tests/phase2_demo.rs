use zelen::parse;
use zelen::translator::{ObjectiveType, Translator};

fn main() {
    println!("=== Phase 2 Features Demo: Aggregates & Optimization ===\n");

    // Example 1: Array Aggregates (sum, min, max)
    example_array_aggregates();
    println!("\n{}\n", "=".repeat(60));

    // Example 2: Product aggregate
    example_product_aggregate();
    println!("\n{}\n", "=".repeat(60));

    // Example 3: Minimization
    example_minimization();
    println!("\n{}\n", "=".repeat(60));

    // Example 4: Maximization
    example_maximization();
    println!("\n{}\n", "=".repeat(60));

    // Example 5: Complex optimization with aggregates
    example_complex_optimization();

    println!("\n=== Demo Complete ===");
}

fn example_array_aggregates() {
    println!("Example 1: Array Aggregates (sum, min, max)");
    
    let source = r#"
        array[1..5] of var 1..10: values;
        
        % Sum of all values must equal 20
        constraint sum(values) == 20;
        
        % Minimum value must be at least 2
        constraint min(values) >= 2;
        
        % Maximum value must be at most 8
        constraint max(values) <= 8;
        
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
                            
                            if let Some(values_arr) = model_data.int_var_arrays.get("values") {
                                print!("  values = [");
                                let mut sum = 0;
                                let mut min_val = i32::MAX;
                                let mut max_val = i32::MIN;
                                
                                for (i, var_id) in values_arr.iter().enumerate() {
                                    if i > 0 {
                                        print!(", ");
                                    }
                                    let val = solution.get_int(*var_id);
                                    print!("{}", val);
                                    sum += val;
                                    min_val = min_val.min(val);
                                    max_val = max_val.max(val);
                                }
                                println!("]");
                                println!("  sum = {}, min = {}, max = {}", sum, min_val, max_val);
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

fn example_product_aggregate() {
    println!("Example 2: Product Aggregate");
    
    let source = r#"
        array[1..3] of var 2..5: factors;
        
        % Product of all factors must equal 24
        constraint product(factors) == 24;
        
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
                            
                            if let Some(factors_arr) = model_data.int_var_arrays.get("factors") {
                                print!("  factors = [");
                                let mut product = 1;
                                
                                for (i, var_id) in factors_arr.iter().enumerate() {
                                    if i > 0 {
                                        print!(", ");
                                    }
                                    let val = solution.get_int(*var_id);
                                    print!("{}", val);
                                    product *= val;
                                }
                                println!("]");
                                println!("  product = {}", product);
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

fn example_minimization() {
    println!("Example 3: Minimization");
    
    let source = r#"
        array[1..4] of var 1..10: costs;
        
        % All costs must be different
        constraint alldifferent(costs);
        
        % First cost must be at least 3
        constraint costs[1] >= 3;
        
        % Minimize total cost
        solve minimize sum(costs);
    "#;
    
    println!("MiniZinc Source:\n{}", source);
    
    match parse(source) {
        Ok(ast) => {
            match Translator::translate_with_vars(&ast) {
                Ok(model_data) => {
                    println!("✓ Translated to Selen Model");
                    println!("  Objective: {:?}", model_data.objective_type);
                    
                    // Check if we have an optimization objective
                    let mut model = model_data.model;
                    let result = match (model_data.objective_type, model_data.objective_var) {
                        (ObjectiveType::Minimize, Some(obj_var)) => {
                            println!("  Running minimize...");
                            model.minimize(obj_var)
                        }
                        (ObjectiveType::Maximize, Some(obj_var)) => {
                            println!("  Running maximize...");
                            model.maximize(obj_var)
                        }
                        _ => {
                            println!("  Running satisfy...");
                            model.solve()
                        }
                    };
                    
                    match result {
                        Ok(solution) => {
                            println!("✓ Optimal solution found!");
                            
                            if let Some(costs_arr) = model_data.int_var_arrays.get("costs") {
                                print!("  costs = [");
                                let mut total_cost = 0;
                                
                                for (i, var_id) in costs_arr.iter().enumerate() {
                                    if i > 0 {
                                        print!(", ");
                                    }
                                    let val = solution.get_int(*var_id);
                                    print!("{}", val);
                                    total_cost += val;
                                }
                                println!("]");
                                println!("  Total cost (minimized) = {}", total_cost);
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

fn example_maximization() {
    println!("Example 4: Maximization");
    
    let source = r#"
        array[1..4] of var 1..10: profits;
        
        % Total must not exceed 30
        constraint sum(profits) <= 30;
        
        % Minimum profit must be at least 2
        constraint min(profits) >= 2;
        
        % Maximize the maximum profit
        solve maximize max(profits);
    "#;
    
    println!("MiniZinc Source:\n{}", source);
    
    match parse(source) {
        Ok(ast) => {
            match Translator::translate_with_vars(&ast) {
                Ok(model_data) => {
                    println!("✓ Translated to Selen Model");
                    println!("  Objective: {:?}", model_data.objective_type);
                    
                    let mut model = model_data.model;
                    let result = match (model_data.objective_type, model_data.objective_var) {
                        (ObjectiveType::Minimize, Some(obj_var)) => model.minimize(obj_var),
                        (ObjectiveType::Maximize, Some(obj_var)) => {
                            println!("  Running maximize...");
                            model.maximize(obj_var)
                        }
                        _ => model.solve(),
                    };
                    
                    match result {
                        Ok(solution) => {
                            println!("✓ Optimal solution found!");
                            
                            if let Some(profits_arr) = model_data.int_var_arrays.get("profits") {
                                print!("  profits = [");
                                let mut total = 0;
                                let mut max_profit = i32::MIN;
                                
                                for (i, var_id) in profits_arr.iter().enumerate() {
                                    if i > 0 {
                                        print!(", ");
                                    }
                                    let val = solution.get_int(*var_id);
                                    print!("{}", val);
                                    total += val;
                                    max_profit = max_profit.max(val);
                                }
                                println!("]");
                                println!("  Total = {}, Max profit (maximized) = {}", total, max_profit);
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

fn example_complex_optimization() {
    println!("Example 5: Complex Optimization with Aggregates");
    
    let source = r#"
        int: n = 5;
        array[1..n] of var 0..20: production;
        var 0..100: total_production;
        var 0..20: max_production;
        
        % Define relationships
        constraint total_production == sum(production);
        constraint max_production == max(production);
        
        % Must produce at least 30 units total
        constraint total_production >= 30;
        
        % Balance constraint: max shouldn't be more than 2x min
        constraint max_production <= 2 * min(production);
        
        % Minimize the maximum production (load balancing)
        solve minimize max_production;
    "#;
    
    println!("MiniZinc Source:\n{}", source);
    
    match parse(source) {
        Ok(ast) => {
            match Translator::translate_with_vars(&ast) {
                Ok(model_data) => {
                    println!("✓ Translated to Selen Model");
                    println!("  Objective: {:?}", model_data.objective_type);
                    
                    let mut model = model_data.model;
                    let result = match (model_data.objective_type, model_data.objective_var) {
                        (ObjectiveType::Minimize, Some(obj_var)) => {
                            println!("  Running minimize (load balancing)...");
                            model.minimize(obj_var)
                        }
                        (ObjectiveType::Maximize, Some(obj_var)) => model.maximize(obj_var),
                        _ => model.solve(),
                    };
                    
                    match result {
                        Ok(solution) => {
                            println!("✓ Optimal solution found!");
                            
                            if let Some(production_arr) = model_data.int_var_arrays.get("production") {
                                print!("  production = [");
                                let mut total = 0;
                                let mut min_prod = i32::MAX;
                                let mut max_prod = i32::MIN;
                                
                                for (i, var_id) in production_arr.iter().enumerate() {
                                    if i > 0 {
                                        print!(", ");
                                    }
                                    let val = solution.get_int(*var_id);
                                    print!("{}", val);
                                    total += val;
                                    min_prod = min_prod.min(val);
                                    max_prod = max_prod.max(val);
                                }
                                println!("]");
                                println!("  Total: {}, Min: {}, Max (minimized): {}", total, min_prod, max_prod);
                                println!("  Load balance ratio: {:.2}", max_prod as f64 / min_prod as f64);
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
