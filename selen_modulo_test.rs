// This is a pure Selen example to test if Selen can solve: remainder = 47 mod 10
// Run with: cargo run --example selen_modulo_test (if placed in examples/)
// Or compile and place in root: rustc --edition 2021 -L target/debug/deps selen_modulo_test.rs -o selen_modulo_test

use selen::model::Model;

fn main() {
    println!("=== Testing Selen Modulo Directly ===\n");

    // Test: remainder = 47 mod 10, should give remainder = 7
    let mut model = Model::new();

    // Create variables
    let number = model.int(10, 100);
    let remainder = model.int(0, 9);

    // Add constraints
    model.new(number.eq(model.int(47, 47)));  // number = 47
    
    // remainder = number mod 10
    // modulo returns a VarId, so we need to constrain it
    let mod_result = model.modulo(number, model.int(10, 10));
    model.new(remainder.eq(mod_result));

    println!("Selen Model Created");
    println!("  number: constrained to 47");
    println!("  remainder: domain [0..9]");
    println!("  constraint: remainder == (47 mod 10)");
    println!();

    match model.solve() {
        Ok(solution) => {
            println!("✓ Solution found!");
            println!("  number = {}", solution.get_int(number));
            println!("  remainder = {}", solution.get_int(remainder));
            println!("  Expected remainder: 7 (47 mod 10 = 7)");
        }
        Err(e) => {
            println!("✗ No solution found");
            println!("  Error: {:?}", e);
            println!("\n  THIS IS THE PROBLEM - Please investigate if Selen's modulo");
            println!("  can properly handle: remainder == dividend mod divisor");
        }
    }
}
