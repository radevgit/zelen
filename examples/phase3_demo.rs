use zelen::parse;
use zelen::translator::{ObjectiveType, Translator};

fn main() {
    println!("=== Phase 3 Features Demo: Element Constraint & Advanced Aggregates ===\n");

    example_element_constraint();
    println!("\n{}\n", "=".repeat(60));

    example_count_aggregate();
    println!("\n{}\n", "=".repeat(60));

    example_exists_aggregate();
    println!("\n{}\n", "=".repeat(60));

    example_forall_aggregate();
    println!("\n{}\n", "=".repeat(60));

    example_complex_scheduling();

    println!("\n=== Demo Complete ===");
}

fn example_element_constraint() {
    println!("Example 1: Element Constraint (Variable Array Index)");

    let source = r#"
        array[1..5] of var 1..10: arr;
        var 1..5: index;
        var 1..10: selected;
        
        % Select element at variable index
        constraint selected == arr[index];
        constraint index == 3;
        constraint selected == 7;
        
        % Other elements can be anything
        constraint arr[1] >= 1;
        
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

                            if let Some(&index_var) = model_data.int_vars.get("index") {
                                println!("  index = {}", solution.get_int(index_var));
                            }

                            if let Some(&selected_var) = model_data.int_vars.get("selected") {
                                println!("  selected = {}", solution.get_int(selected_var));
                            }

                            if let Some(arr) = model_data.int_var_arrays.get("arr") {
                                print!("  arr = [");
                                for (i, &var_id) in arr.iter().enumerate() {
                                    if i > 0 { print!(", "); }
                                    print!("{}", solution.get_int(var_id));
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

fn example_count_aggregate() {
    println!("Example 2: Count Aggregate");

    let source = r#"
        array[1..6] of var 1..3: values;
        
        % Count how many values equal 2
        constraint count(values, 2) == 3;
        
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
                                let mut count_2s = 0;
                                for (i, &var_id) in values_arr.iter().enumerate() {
                                    if i > 0 { print!(", "); }
                                    let val = solution.get_int(var_id);
                                    print!("{}", val);
                                    if val == 2 { count_2s += 1; }
                                }
                                println!("]");
                                println!("  Count of 2s: {}", count_2s);
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

fn example_exists_aggregate() {
    println!("Example 3: Exists Aggregate (Any Element True)");

    let source = r#"
        array[1..4] of var bool: flags;
        var bool: any_set;
        
        % Check if at least one flag is true
        constraint any_set == exists(flags);
        constraint any_set;  % Force at least one to be true
        
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

                            if let Some(flags_arr) = model_data.bool_var_arrays.get("flags") {
                                print!("  flags = [");
                                for (i, &var_id) in flags_arr.iter().enumerate() {
                                    if i > 0 { print!(", "); }
                                    print!("{}", if solution.get_int(var_id) != 0 { "true" } else { "false" });
                                }
                                println!("]");
                                
                                let any_true = flags_arr.iter()
                                    .any(|&v| solution.get_int(v) != 0);
                                println!("  Any flag true: {}", any_true);
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

fn example_forall_aggregate() {
    println!("Example 4: Forall Aggregate (All Elements True)");

    let source = r#"
        array[1..4] of var bool: requirements;
        var bool: all_met;
        
        % Check if all requirements are met
        constraint all_met == forall(requirements);
        constraint all_met;  % Force all to be true
        
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

                            if let Some(req_arr) = model_data.bool_var_arrays.get("requirements") {
                                print!("  requirements = [");
                                for (i, &var_id) in req_arr.iter().enumerate() {
                                    if i > 0 { print!(", "); }
                                    print!("{}", if solution.get_int(var_id) != 0 { "true" } else { "false" });
                                }
                                println!("]");
                                
                                let all_true = req_arr.iter()
                                    .all(|&v| solution.get_int(v) != 0);
                                println!("  All requirements met: {}", all_true);
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

fn example_complex_scheduling() {
    println!("Example 5: Complex Scheduling Problem");

    let source = r#"
        % 5 tasks, each can be assigned to day 1-7
        array[1..5] of var 1..7: task_days;
        
        % Select which day to check (variable index)
        var 1..7: check_day;
        var 1..5: tasks_on_check_day;
        
        % Count how many tasks are on the check_day
        constraint tasks_on_check_day == count(task_days, check_day);
        constraint check_day == 3;  % Check day 3
        
        % At least one task on day 3
        constraint tasks_on_check_day >= 1;
        
        % Ensure at least one task is assigned
        constraint task_days[1] > 0;
        
        solve minimize max(task_days);
    "#;

    println!("MiniZinc Source:\n{}", source);

    match parse(source) {
        Ok(ast) => {
            match Translator::translate_with_vars(&ast) {
                Ok(model_data) => {
                    println!("✓ Translated to Selen Model");
                    println!("  Objective: {:?}", model_data.objective_type);

                    let result = if let (ObjectiveType::Minimize, Some(obj_var)) = (model_data.objective_type, model_data.objective_var) {
                        println!("  Running minimize...");
                        model_data.model.minimize(obj_var)
                    } else {
                        println!("  Running satisfy...");
                        model_data.model.solve()
                    };

                    match result {
                        Ok(solution) => {
                            println!("✓ Optimal solution found!");

                            if let Some(task_arr) = model_data.int_var_arrays.get("task_days") {
                                print!("  task_days = [");
                                for (i, &var_id) in task_arr.iter().enumerate() {
                                    if i > 0 { print!(", "); }
                                    print!("{}", solution.get_int(var_id));
                                }
                                println!("]");
                            }

                            if let Some(&check_day_var) = model_data.int_vars.get("check_day") {
                                println!("  check_day = {}", solution.get_int(check_day_var));
                            }

                            if let Some(&tasks_count_var) = model_data.int_vars.get("tasks_on_check_day") {
                                println!("  tasks on check_day = {}", solution.get_int(tasks_count_var));
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
