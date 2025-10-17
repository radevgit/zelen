/// Final verification: Translator modulo now works!
use zelen::{parse, translator::Translator};

fn main() {
    println!("=== VERIFICATION: Translator Modulo Fix ===\n");

    let test_cases = vec![
        ("Simple modulo", r#"
            var 1..100: dividend;
            var 1..10: divisor;
            var 0..9: remainder;
            
            constraint remainder == dividend mod divisor;
            constraint dividend == 47;
            constraint divisor == 10;
            
            solve satisfy;
        "#, 7),
        
        ("Different values", r#"
            var 1..1000: x;
            var 2..20: y;
            var 0..19: r;
            
            constraint r == x mod y;
            constraint x == 123;
            constraint y == 11;
            
            solve satisfy;
        "#, 2),  // 123 mod 11 = 2
        
        ("Edge case: power of 2", r#"
            var 1..256: a;
            var 1..16: b;
            var 0..15: rem;
            
            constraint rem == a mod b;
            constraint a == 255;
            constraint b == 16;
            
            solve satisfy;
        "#, 15),  // 255 mod 16 = 15
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (name, source, expected) in test_cases {
        println!("Test: {}", name);
        
        let ast = parse(source);
        if ast.is_err() {
            println!("  ✗ FAILED: Parse error");
            failed += 1;
            continue;
        }

        let result = Translator::translate_with_vars(&ast.unwrap());
        if result.is_err() {
            println!("  ✗ FAILED: Translation error");
            failed += 1;
            continue;
        }

        let model_data = result.unwrap();
        
        match model_data.model.solve() {
            Ok(sol) => {
                if let Some(rem_var) = model_data.int_vars.get("rem")
                    .or_else(|| model_data.int_vars.get("remainder"))
                    .or_else(|| model_data.int_vars.get("r")) {
                    let rem_val = sol.get_int(*rem_var);
                    if rem_val == expected {
                        println!("  ✓ PASSED: remainder = {}\n", rem_val);
                        passed += 1;
                    } else {
                        println!("  ✗ FAILED: expected {}, got {}\n", expected, rem_val);
                        failed += 1;
                    }
                } else {
                    println!("  ✗ FAILED: remainder variable not found\n");
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  ✗ FAILED: Solver error: {:?}\n", e);
                failed += 1;
            }
        }
    }

    println!("=== RESULTS ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    
    if failed == 0 {
        println!("\n✓ ALL TESTS PASSED! Modulo fix is working!");
    }
}
