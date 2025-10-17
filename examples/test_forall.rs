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

    // Test 2: Forall with multiple generators (if supported)
    println!("Test 2: Forall with inequality constraints\n");
    let source2 = r#"
        int: n = 4;
        array[1..n] of var 1..10: queens;
        
        constraint forall(i in 1..n, j in i+1..n)(
            abs(queens[i] - queens[j]) > abs(i - j)
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
                            if let Some(arr) = model_data.int_var_arrays.get("queens") {
                                println!("  queens array values:");
                                for (i, var_id) in arr.iter().enumerate() {
                                    let val = sol.get_int(*var_id);
                                    println!("    queens[{}] = {}", i + 1, val);
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
