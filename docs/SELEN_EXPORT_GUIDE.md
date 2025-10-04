# Zelen to Selen Export Guide

**Date**: October 4, 2025  
**Purpose**: Comprehensive guide for exporting FlatZinc models to Selen programmatic API  
**Audience**: Zelen developers and users creating standalone Selen programs

---

## Table of Contents

1. [Overview](#overview)
2. [API Mapping Reference](#api-mapping-reference)
3. [Variable Creation](#variable-creation)
4. [Constraint Translation](#constraint-translation)
5. [Data File Integration](#data-file-integration)
6. [Complete Example](#complete-example)
7. [Common Pitfalls](#common-pitfalls)
8. [Best Practices](#best-practices)

---

## Overview

When exporting a FlatZinc model to a standalone Selen program, you need to translate:
- **Variables** → Selen variable creation methods
- **Constraints** → Selen programmatic API calls
- **Data values** → Equality constraints
- **Solve goal** → `model.solve()` or `model.minimize()`/`model.maximize()`

### Key Principles

1. **Use programmatic API only** - No `post!` macro, no direct propagator access
2. **Preserve problem semantics** - Unbounded variables should use `i32::MIN/MAX` or `f64::INFINITY`
3. **Include data constraints** - Values from `.dzn` files must become constraints
4. **Handle introduced variables** - These are helper variables, create them like any other

---

## API Mapping Reference

### Model Creation

```rust
// CORRECT
let mut model = Model::default();

// ALSO CORRECT (with config)
let config = SolverConfig::default().with_time_limit(60.0);
let mut model = Model::with_config(config);

// WRONG
let mut model = Model::new();  // Takes required arguments!
```

### Solving

```rust
// Satisfaction problem
match model.solve() {
    Ok(solution) => { /* use solution */ }
    Err(e) => { /* handle error */ }
}

// Optimization
match model.minimize(objective_var) {
    Ok(solution) => { /* use solution */ }
    Err(e) => { /* handle error */ }
}

// WRONG
let mut solver = Solver::new(model);  // Solver is not public API
```

### Solution Access

```rust
// For any variable type (uses type inference)
let value: i32 = solution.get(var_id);
let value: f64 = solution.get(var_id);

// Type-specific methods (if needed for clarity)
let int_val: i32 = solution.get(int_var);
let float_val: f64 = solution.get(float_var);

// WRONG
solution.get_float_value(var)     // No such method
solution.get_int_value(var)       // No such method
solution[var]                     // Returns Val enum, not raw value
```

---

## Variable Creation

### Integer Variables

| FlatZinc Declaration | Selen API |
|---------------------|-----------|
| `var int: x;` (unbounded) | `let x = model.int(i32::MIN, i32::MAX);` |
| `var 1..10: x;` | `let x = model.int(1, 10);` |
| `var -5..5: x;` | `let x = model.int(-5, 5);` |
| `var bool: b;` | `let b = model.bool();` |

### Float Variables

| FlatZinc Declaration | Selen API |
|---------------------|-----------|
| `var float: x;` (unbounded) | `let x = model.float(f64::NEG_INFINITY, f64::INFINITY);` |
| `var 0.0..10.0: x;` | `let x = model.float(0.0, 10.0);` |
| `var -1.5..1.5: x;` | `let x = model.float(-1.5, 1.5);` |

### Array Parameters

```rust
// FlatZinc: array [1..3] of float: coeffs = [1.0, -1.0, 1.0];
// Selen: Use Rust slice or Vec directly
let coeffs = vec![1.0, -1.0, 1.0];
// or
let coeffs = [1.0, -1.0, 1.0];
```

**Important**: Selen's bound inference will automatically handle unbounded variables by inferring reasonable bounds from context.

---

## Constraint Translation

### Arithmetic Constraints

#### Integer Linear Equations

```rust
// FlatZinc: constraint int_lin_eq([2, 3, -1], [x, y, z], 5);
// Meaning: 2*x + 3*y - 1*z = 5
model.int_lin_eq(&[2, 3, -1], &[x, y, z], 5);
```

#### Float Linear Equations

```rust
// FlatZinc: constraint float_lin_eq([1.0, -1.0], [x, y], -1.0);
// Meaning: 1.0*x - 1.0*y = -1.0  →  x = y - 1.0
model.float_lin_eq(&[1.0, -1.0], &[x, y], -1.0);
```

#### Float Linear Inequalities

```rust
// FlatZinc: constraint float_lin_le([2.5, 3.0], [x, y], 100.0);
// Meaning: 2.5*x + 3.0*y ≤ 100.0
model.float_lin_le(&[2.5, 3.0], &[x, y], 100.0);
```

#### Float Linear Not-Equal

```rust
// FlatZinc: constraint float_lin_ne([2.0, 3.0], [x, y], 12.0);
// Meaning: 2.0*x + 3.0*y ≠ 12.0
model.float_lin_ne(&[2.0, 3.0], &[x, y], 12.0);
```

### Multiplication Constraints

#### Float Multiplication

```rust
// FlatZinc: constraint float_times(x, y, z);
// Meaning: z = x * y
let z_calc = model.mul(x, y);
model.new(z.eq(z_calc));
```

**Why two steps?**
- `model.mul(x, y)` creates a new variable representing `x * y`
- `model.new(z.eq(z_calc))` constrains `z` to equal that result
- This is the programmatic API pattern for multiplication

#### Integer Multiplication

```rust
// FlatZinc: constraint int_times(x, y, z);
// Same pattern as floats
let z_calc = model.mul(x, y);
model.new(z.eq(z_calc));
```

### Comparison Constraints

#### Basic Comparisons

```rust
// FlatZinc: constraint int_le(x, y);
// Selen: x ≤ y
model.new(x.le(y));

// FlatZinc: constraint int_lt(x, y);
// Selen: x < y
model.new(x.lt(y));

// FlatZinc: constraint int_eq(x, y);
// Selen: x = y
model.new(x.eq(y));

// FlatZinc: constraint int_ne(x, y);
// Selen: x ≠ y
model.new(x.ne(y));
```

#### Reified Comparisons

```rust
// FlatZinc: constraint int_le_reif(x, y, b);
// Meaning: b ⇔ (x ≤ y)
model.int_le_reif(x, y, b);

// Similarly for other comparisons
model.int_lt_reif(x, y, b);  // b ⇔ (x < y)
model.int_eq_reif(x, y, b);  // b ⇔ (x = y)
model.int_ne_reif(x, y, b);  // b ⇔ (x ≠ y)

// Float versions
model.float_le_reif(x, y, b);
model.float_lt_reif(x, y, b);
```

### Global Constraints

#### All Different

```rust
// FlatZinc: constraint all_different_int([x, y, z]);
let vars = vec![x, y, z];
model.alldiff(&vars);
```

#### Element Constraint

```rust
// FlatZinc: constraint array_int_element(index, [a, b, c, d], result);
// Meaning: result = array[index]
let array = vec![a, b, c, d];
model.array_int_element(index, &array, result);

// Float version
// FlatZinc: constraint array_float_element(index, [a, b, c], result);
let array = vec![a, b, c];
model.array_float_element(index, &array, result);
```

### Type Conversion Constraints

```rust
// FlatZinc: constraint int2float(int_var, float_var);
model.int2float(int_var, float_var);

// FlatZinc: constraint float2int(float_var, int_var);  // Truncation
model.float2int(float_var, int_var);
```

---

## Data File Integration

**Critical Issue**: When a `.dzn` data file provides values, these MUST be added as constraints!

### Example Problem

Given:
```minizinc
% loan.mzn
var float: P;
var float: R;
var 0.0..10.0: I;
% ... constraints ...
```

And data file:
```minizinc
% loan1.dzn
P = 1000.0;
R = 260.0;
I = 0.04;
```

### Incorrect Export (Under-constrained)

```rust
// WRONG: No data constraints!
let mut model = Model::default();
let p = model.float(f64::NEG_INFINITY, f64::INFINITY);  // Unbounded!
let r = model.float(f64::NEG_INFINITY, f64::INFINITY);  // Unbounded!
let i = model.float(0.0, 10.0);
// ... constraints ...
// Result: Solver finds extreme values like P=-20000000, R=-10000
```

### Correct Export (With Data Constraints)

```rust
// CORRECT: Include data as constraints
let mut model = Model::default();
let p = model.float(f64::NEG_INFINITY, f64::INFINITY);
let r = model.float(f64::NEG_INFINITY, f64::INFINITY);
let i = model.float(0.0, 10.0);

// Add data constraints from .dzn file
model.new(p.eq(1000.0));
model.new(r.eq(260.0));
model.new(i.eq(0.04));

// ... rest of constraints ...
// Result: Solver finds meaningful solution
```

### Implementation Approach

When Zelen parses both a `.fzn` file and `.dzn` file:

1. Parse variable declarations from `.fzn`
2. Parse data values from `.dzn`
3. For each data value, generate an equality constraint:
   ```rust
   // For numeric data
   model.new(var.eq(value));
   
   // For array data, constrain each element
   for (i, &val) in array_data.iter().enumerate() {
       model.new(array_vars[i].eq(val));
   }
   ```

---

## Complete Example

### Original FlatZinc (loan.fzn)

```flatzinc
var float: R;
var float: P;
var 0.0..10.0: I;
var float: B1;
var float: B2;
var float: B3;
var float: B4;
var 1.0..11.0: X_INTRODUCED_1_;
var float: X_INTRODUCED_2_;

constraint float_lin_eq([1.0,-1.0],[I,X_INTRODUCED_1_],-1.0);
constraint float_times(P,X_INTRODUCED_1_,X_INTRODUCED_2_);
constraint float_lin_eq([1.0,-1.0,1.0],[B1,X_INTRODUCED_2_,R],-0.0);
% ... more constraints ...

solve satisfy;
```

### Data File (loan1.dzn)

```minizinc
P = 1000.0;
R = 260.0;
I = 0.04;
```

### Correct Selen Export

```rust
use selen::prelude::*;

fn main() {
    let mut model = Model::default();
    
    // ===== VARIABLES =====
    let r = model.float(f64::NEG_INFINITY, f64::INFINITY);
    let p = model.float(f64::NEG_INFINITY, f64::INFINITY);
    let i = model.float(0.0, 10.0);
    let b1 = model.float(f64::NEG_INFINITY, f64::INFINITY);
    let b2 = model.float(f64::NEG_INFINITY, f64::INFINITY);
    let b3 = model.float(f64::NEG_INFINITY, f64::INFINITY);
    let b4 = model.float(f64::NEG_INFINITY, f64::INFINITY);
    let x1 = model.float(1.0, 11.0);
    let x2 = model.float(f64::NEG_INFINITY, f64::INFINITY);
    
    // ===== DATA CONSTRAINTS (from .dzn file) =====
    model.new(p.eq(1000.0));
    model.new(r.eq(260.0));
    model.new(i.eq(0.04));
    
    // ===== CONSTRAINTS (from .fzn file) =====
    
    // X1 = I + 1
    model.float_lin_eq(&[1.0, -1.0], &[i, x1], -1.0);
    
    // X2 = P * X1
    let x2_calc = model.mul(p, x1);
    model.new(x2.eq(x2_calc));
    
    // B1 = X2 - R
    model.float_lin_eq(&[1.0, -1.0, 1.0], &[b1, x2, r], 0.0);
    
    // ... more constraints ...
    
    // ===== SOLVE =====
    match model.solve() {
        Ok(solution) => {
            let p_val: f64 = solution.get(p);
            let r_val: f64 = solution.get(r);
            let i_val: f64 = solution.get(i);
            let b4_val: f64 = solution.get(b4);
            
            println!("Borrowing {:.2} at {:.1}% interest", p_val, i_val * 100.0);
            println!("Repaying {:.2} per quarter", r_val);
            println!("Balance after 4 quarters: {:.2}", b4_val);
        }
        Err(e) => {
            println!("No solution found: {:?}", e);
        }
    }
}
```

---

## Common Pitfalls

### ❌ Pitfall 1: Using `Model::new()` Without Arguments

```rust
// WRONG
let mut model = Model::new();  // Error: missing required arguments

// CORRECT
let mut model = Model::default();
```

### ❌ Pitfall 2: Creating Solver Directly

```rust
// WRONG
let mut solver = Solver::new(model);
match solver.solve() { ... }

// CORRECT
match model.solve() { ... }
```

### ❌ Pitfall 3: Wrong Solution Access

```rust
// WRONG
let value = solution.get_float_value(var);  // No such method
let value = solution[var];                  // Returns Val enum

// CORRECT
let value: f64 = solution.get(var);
```

### ❌ Pitfall 4: Using Propagators Directly

```rust
// WRONG - Internal API
model.props.equals(x, y);
model.props.less_than(x, y);

// CORRECT - Programmatic API
model.new(x.eq(y));
model.new(x.lt(y));
```

### ❌ Pitfall 5: Omitting Data Constraints

```rust
// WRONG - Under-constrained problem
let p = model.float(f64::NEG_INFINITY, f64::INFINITY);
// ... constraints but no data values ...
// Result: Extreme values

// CORRECT - Include data
let p = model.float(f64::NEG_INFINITY, f64::INFINITY);
model.new(p.eq(1000.0));  // From .dzn file
```

### ❌ Pitfall 6: Incorrect Multiplication Pattern

```rust
// WRONG - No direct float_times method
model.float_times(x, y, z);

// CORRECT - Two-step pattern
let z_calc = model.mul(x, y);
model.new(z.eq(z_calc));
```

---

## Best Practices

### 1. **Preserve Unbounded Semantics**

When FlatZinc declares `var float: x`, use infinite bounds:

```rust
let x = model.float(f64::NEG_INFINITY, f64::INFINITY);
```

Selen's automatic bound inference will handle this appropriately.

### 2. **Always Include Data Constraints**

If a `.dzn` file provides values, add them as constraints:

```rust
// From .dzn: P = 1000.0;
model.new(p.eq(1000.0));
```

### 3. **Use Type Inference for Solution Access**

```rust
// Cleaner with type annotation
let value: f64 = solution.get(var);

// Instead of verbose methods
let value = solution.get(var); // Type inferred from context
```

### 4. **Handle Optional Variables Gracefully**

If FlatZinc has `opt` variables (optional inputs):
- If they have values in `.dzn`, add equality constraints
- If they don't have values, they become decision variables

### 5. **Add Comments Explaining FlatZinc Origin**

```rust
// From FlatZinc: constraint float_lin_eq([1.0,-1.0],[I,X1],-1.0);
// Meaning: I - X1 = -1.0  →  X1 = I + 1
model.float_lin_eq(&[1.0, -1.0], &[i, x1], -1.0);
```

### 6. **Test with Multiple Data Sets**

Create multiple versions with different `.dzn` data to verify correctness.

### 7. **Configure Solver Appropriately**

```rust
let config = SolverConfig::default()
    .with_time_limit(60.0)
    .with_unbounded_inference_factor(1000);
let mut model = Model::with_config(config);
```

---

## Advanced Topics

### Custom Bound Inference Factor

For problems with known value ranges:

```rust
// Conservative inference (tighter bounds)
let config = SolverConfig::default()
    .with_unbounded_inference_factor(100);

// Aggressive inference (wider bounds)
let config = SolverConfig::default()
    .with_unbounded_inference_factor(10000);

let mut model = Model::with_config(config);
```

### Optimization Problems

```rust
// FlatZinc: solve minimize cost;
let cost = model.float(0.0, 1000.0);
// ... constraints ...
match model.minimize(cost) {
    Ok(solution) => {
        let cost_val: f64 = solution.get(cost);
        println!("Minimum cost: {:.2}", cost_val);
    }
    Err(e) => println!("No solution: {:?}", e),
}

// FlatZinc: solve maximize profit;
match model.maximize(profit) {
    Ok(solution) => { /* ... */ }
    Err(e) => { /* ... */ }
}
```

### Array Variables

```rust
// FlatZinc: array [1..5] of var 1..10: arr;
let arr: Vec<VarId> = (0..5)
    .map(|_| model.int(1, 10))
    .collect();

// Use in constraints
model.alldiff(&arr);
model.array_int_element(index, &arr, result);
```

---

## Troubleshooting

### Problem: "Extreme values in solution"

**Symptom**: Variables take values like -20000000 or similar extreme numbers.

**Cause**: Unbounded variables without data constraints.

**Solution**: Add equality constraints from `.dzn` file data.

### Problem: "No solution found"

**Symptom**: `model.solve()` returns `Err`.

**Cause**: Over-constrained problem or incorrect constraint translation.

**Solution**: 
1. Check FlatZinc constraint semantics carefully
2. Verify data constraints are correct
3. Test with a simpler version of the problem

### Problem: "Type mismatch errors"

**Symptom**: Compilation errors about mismatched types.

**Cause**: Mixing integer and float operations.

**Solution**: Use type conversion constraints:
```rust
model.int2float(int_var, float_var);
model.float2int(float_var, int_var);
```

---

## Zelen Implementation Checklist

When implementing FlatZinc to Selen export in Zelen:

- [ ] Parse variable declarations and map to `model.int()` / `model.float()` / `model.bool()`
- [ ] Use `i32::MIN/MAX` for unbounded integers, `f64::INFINITY` for unbounded floats
- [ ] Parse array parameters as Rust `Vec` or arrays
- [ ] Map each FlatZinc constraint to the appropriate Selen method
- [ ] Use two-step pattern for multiplication: `let result = model.mul(x, y); model.new(z.eq(result));`
- [ ] Parse `.dzn` data files if provided
- [ ] Add equality constraints for all data values
- [ ] Map solve goal: `satisfy` → `model.solve()`, `minimize/maximize` → `model.minimize()`/`model.maximize()`
- [ ] Use `solution.get(var)` with type inference for value extraction
- [ ] Generate clean, commented code with FlatZinc origins
- [ ] Test exported programs compile and run correctly
- [ ] Verify solutions match expected values from `.dzn` data

---

## Summary

**Key Takeaways**:

1. **Model creation**: `Model::default()` or `Model::with_config(config)`
2. **Variables**: Use `i32::MIN/MAX` and `f64::INFINITY` for unbounded
3. **Multiplication**: Two-step pattern with `model.mul()` then `model.new(z.eq(...))`
4. **Data values**: MUST be added as equality constraints
5. **Solution access**: `let value: Type = solution.get(var)`
6. **Programmatic API**: Use `model.new(x.eq(y))`, not `model.props.equals(x, y)`
7. **Solving**: Call `model.solve()` directly, not `Solver::new(model)`

Following these guidelines will ensure Zelen exports produce correct, idiomatic Selen programs that solve problems as expected.

---

**For questions or issues**, refer to:
- Selen API documentation: `selen/docs/`
- Selen examples: `selen/examples/`
- Bound inference docs: `selen/docs/development/BOUND_INFERENCE_DESIGN.md`
- Integration guide: `zelen/docs/SELEN_BOUND_INFERENCE.md`
