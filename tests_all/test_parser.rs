#![allow(dead_code)]

use zelen::parse;

fn main() {
    // Test 1: Simple error
    let source1 = "int n = 5"; // Missing colon
    match parse(source1) {
        Ok(_) => println!("Test 1: Should have failed!"),
        Err(e) => println!("Test 1 Error:\n{}\n", e),
    }
    
    // Test 2: N-Queens model
    let source2 = r#"
        int: n = 4;
        array[1..n] of var 1..n: queens;
        constraint alldifferent(queens);
        solve satisfy;
    "#;
    match parse(source2) {
        Ok(model) => println!("Test 2: Successfully parsed {} items", model.items.len()),
        Err(e) => println!("Test 2 Error:\n{}\n", e),
    }
    
    // Test 3: Expression test
    let source3 = r#"
        constraint sum(arr) <= 100;
    "#;
    match parse(source3) {
        Ok(model) => println!("Test 3: Successfully parsed {} items", model.items.len()),
        Err(e) => println!("Test 3 Error:\n{}\n", e),
    }
    
    // Test 4: Generator call
    let source4 = r#"
        constraint forall(i in 1..n)(x[i] > 0);
    "#;
    match parse(source4) {
        Ok(model) => println!("Test 4: Successfully parsed {} items", model.items.len()),
        Err(e) => println!("Test 4 Error:\n{}\n", e),
    }
}
