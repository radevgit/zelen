/// Test forall loops (comprehensions) in constraints
use zelen::{parse, translator::Translator};

fn main() {
    println!("=== Testing Forall Loops ===\n");

    // Test 1: Simple forall with constraint
    println!("Test 1: Simple forall loop\n");
    let source1 = r#"
        int: n = 5;
        array[1..n] of var 1..10: x;
        
        constraint forall(i in 1..n)(x[i] >= i);
        
        solve satisfy;
    "#;

    let ast = parse(source1).unwrap();
    let result = Translator::translate_with_vars(&ast);
    
    match result {
        Ok(model_data) => {
            match model_data.model.solve() {
                Ok(sol) => {
                    println!("✓ SOLVED!");
                    if let Some(arr) = model_data.int_var_arrays.get("x") {
                        println!("  x array values:");
                        for (i, var_id) in arr.iter().enumerate() {
                            let val = sol.get_int(*var_id);
                            println!("    x[{}] = {}", i + 1, val);
                            assert!(val >= (i + 1) as i32, "Constraint violated!");
                        }
                    }
                    println!("  ✓ All constraints satisfied!\n");
                }
                Err(e) => println!("✗ FAILED TO SOLVE: {:?}\n", e),
            }
        }
        Err(e) => println!("✗ TRANSLATION ERROR: {:?}\n", e),
    }

    // Test 2: Forall with multiple generators (nested loops)
    println!("Test 2: Forall with multiple generators\n");
    let source2 = r#"
        int: n = 3;
        array[1..n] of var 1..10: x;
        
        constraint forall(i in 1..n, j in i+1..n)(
            x[i] < x[j]
        );
        
        solve satisfy;
    "#;

    let ast2 = parse(source2);
    match ast2 {
        Ok(parsed_ast) => {
            let result2 = Translator::translate_with_vars(&parsed_ast);
            match result2 {
                Ok(model_data) => {
                    match model_data.model.solve() {
                        Ok(sol) => {
                            println!("✓ SOLVED!");
                            if let Some(arr) = model_data.int_var_arrays.get("x") {
                                println!("  x array values:");
                                for (i, var_id) in arr.iter().enumerate() {
                                    let val = sol.get_int(*var_id);
                                    println!("    x[{}] = {}", i + 1, val);
                                }
                                // Verify multi-generator constraints
                                let vals: Vec<_> = arr.iter().map(|v| sol.get_int(*v)).collect();
                                for i in 0..vals.len() {
                                    for j in (i+1)..vals.len() {
                                        assert!(vals[i] < vals[j], "Constraint x[{}] < x[{}] violated!", i+1, j+1);
                                    }
                                }
                            }
                            println!("  ✓ All constraints satisfied!\n");
                        }
                        Err(e) => println!("✗ FAILED TO SOLVE: {:?}\n", e),
                    }
                }
                Err(e) => println!("✗ TRANSLATION ERROR: {:?}\n", e),
            }
        }
        Err(e) => println!("✗ PARSE ERROR: {:?}\n", e),
    }
}
