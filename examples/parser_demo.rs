use zelen::parse;

fn main() {
    println!("=== MiniZinc Parser Examples ===\n");
    
    // Example 1: N-Queens problem
    println!("Example 1: N-Queens Problem");
    let nqueens = r#"
% N-Queens Problem
int: n = 8;

% Decision variables: queen position in each row
array[1..n] of var 1..n: queens;

% All queens in different columns
constraint alldifferent(queens);

% No two queens on same diagonal
constraint forall(i in 1..n, j in i+1..n) (
    queens[i] != queens[j] + (j - i) /\
    queens[i] != queens[j] - (j - i)
);

solve satisfy;

output ["queens = ", show(queens), "\n"];
    "#;
    
    match parse(nqueens) {
        Ok(model) => println!("✓ Successfully parsed {} items\n", model.items.len()),
        Err(e) => println!("✗ Parse error:\n{}\n", e),
    }
    
    // Example 2: Simple optimization
    println!("Example 2: Simple Optimization");
    let optimization = r#"
int: budget = 100;
array[1..5] of int: costs = [10, 20, 15, 30, 25];
array[1..5] of int: values = [50, 100, 75, 150, 125];
array[1..5] of var 0..1: x;

constraint sum(i in 1..5)(costs[i] * x[i]) <= budget;

solve maximize sum(i in 1..5)(values[i] * x[i]);
    "#;
    
    match parse(optimization) {
        Ok(model) => println!("✓ Successfully parsed {} items\n", model.items.len()),
        Err(e) => println!("✗ Parse error:\n{}\n", e),
    }
    
    // Example 3: Syntax error - missing colon
    println!("Example 3: Syntax Error (missing colon)");
    let error1 = r#"
int n = 5;
var 1..n: x;
    "#;
    
    match parse(error1) {
        Ok(_) => println!("✗ Should have failed!\n"),
        Err(e) => println!("✓ Caught error:\n{}\n", e),
    }
    
    // Example 4: Syntax error - missing semicolon
    println!("Example 4: Syntax Error (missing semicolon)");
    let error2 = r#"
int: n = 5
var 1..n: x;
    "#;
    
    match parse(error2) {
        Ok(_) => println!("✗ Should have failed!\n"),
        Err(e) => println!("✓ Caught error:\n{}\n", e),
    }
    
    // Example 5: Complex expressions
    println!("Example 5: Complex Expressions");
    let complex = r#"
int: n = 10;
array[1..n] of var 1..100: x;

constraint sum(x) == 500;
constraint forall(i in 1..n-1)(x[i] <= x[i+1]);
constraint x[1] >= 10;
constraint x[n] <= 90;

solve minimize sum(i in 1..n)(x[i] * x[i]);
    "#;
    
    match parse(complex) {
        Ok(model) => println!("✓ Successfully parsed {} items\n", model.items.len()),
        Err(e) => println!("✗ Parse error:\n{}\n", e),
    }
    
    // Example 6: Array comprehension
    println!("Example 6: Array Comprehension");
    let array_comp = r#"
int: n = 10;
array[1..n] of int: squares = [i*i | i in 1..n];
array[int] of int: evens = [i | i in 1..20 where i mod 2 == 0];
    "#;
    
    match parse(array_comp) {
        Ok(model) => println!("✓ Successfully parsed {} items\n", model.items.len()),
        Err(e) => println!("✗ Parse error:\n{}\n", e),
    }
    
    // Example 7: Set operations
    println!("Example 7: Set Operations");
    let sets = r#"
set of int: S = {1, 3, 5, 7, 9};
set of int: T = 1..10;
var 1..10: x;

constraint x in S;
constraint x in T;
    "#;
    
    match parse(sets) {
        Ok(model) => println!("✓ Successfully parsed {} items\n", model.items.len()),
        Err(e) => println!("✗ Parse error:\n{}\n", e),
    }
    
    println!("=== All examples completed ===");
}
