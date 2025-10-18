#![allow(dead_code)]

// Pure Selen test: Modulo with two variables
// This shows the issue directly in Selen API without MiniZinc layer

use selen::prelude::*;

fn main() {
    println!("=== Pure Selen: Modulo with Two Variables ===\n");

    let mut m = Model::default();

    // Create variables
    let dividend = m.int(1, 100);
    let divisor = m.int(1, 10);  // Divisor with range [1..10]
    let remainder = m.int(0, 9);

    // Add constraints
    m.new(dividend.eq(47));       // dividend = 47
    m.new(divisor.eq(10));        // divisor = 10
    
    // remainder = dividend mod divisor
    let mod_result = m.modulo(dividend, divisor);
    m.new(remainder.eq(mod_result));

    println!("Selen Model Setup:");
    println!("  dividend: [1..100] constrained to 47");
    println!("  divisor: [1..10] constrained to 10");
    println!("  remainder: [0..9]");
    println!("  constraint: dividend == 47");
    println!("  constraint: divisor == 10");
    println!("  constraint: remainder == (dividend mod divisor)");
    println!("\nExpected: dividend=47, divisor=10, remainder=7 (47 mod 10 = 7)\n");

    match m.solve() {
        Ok(solution) => {
            println!("✓ Solution found!");
            let div_val = solution.get_int(dividend);
            let divisor_val = solution.get_int(divisor);
            let rem_val = solution.get_int(remainder);
            println!("  dividend = {}", div_val);
            println!("  divisor = {}", divisor_val);
            println!("  remainder = {}", rem_val);
            
            if rem_val == 7 {
                println!("  ✓ CORRECT! (47 mod 10 = 7)");
            } else {
                println!("  ✗ WRONG! (47 mod 10 should be 7, not {})", rem_val);
            }
        }
        Err(e) => {
            println!("✗ No solution found!");
            println!("  Error: {:?}", e);
            println!("\nThis happens when divisor has an initial variable domain [1..10]");
            println!("then is constrained to 10 afterwards.");
            println!("\nCompare with selen_modulo_test.rs where divisor is created");
            println!("as a constant [10..10] from the start - that works!");
        }
    }
}
