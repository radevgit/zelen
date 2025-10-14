# Selen API Syntax Corrections

## Date
October 4, 2025

## Issue
The hand-crafted `loan_problem.rs` example and the `exporter.rs` code generator were using incorrect Selen API syntax.

## Corrections Made

### 1. Model Creation
**WRONG:**
```rust
let mut model = Model::new();
```

**CORRECT:**
```rust
let mut model = Model::default();
```

### 2. Solving
**WRONG:**
```rust
let mut solver = Solver::new(model);
match solver.solve() {
```

**CORRECT:**
```rust
match model.solve() {
```

### 3. Multiplication Constraints (float_times)
**WRONG:**
```rust
model.float_times(p, x1, x2);  // Does not exist!
```

**CORRECT:**
```rust
let x2_result = model.mul(p, x1);  // Create expression
model.new(x2.eq(x2_result));       // Post equality constraint
```

**Explanation:** 
- Selen does not have a `float_times` method on Model
- Instead, use `model.mul(a, b)` which returns an expression
- Then post an equality constraint: `model.new(c.eq(expression))`
- This is how Zelen's mapper implements `map_float_times()` in `src/mapper/constraints/float.rs`

### 4. Solution Value Extraction
**WRONG:**
```rust
let val = solution.get_float_value(var);  // Does not exist!
```

**CORRECT:**
```rust
// Method 1: Use get_float() for float variables (panics if wrong type)
let val = solution.get_float(var);

// Method 2: Use get_int() for integer variables (panics if wrong type)
let val = solution.get_int(var);

// Method 3: Use indexing operator (returns &Val)
match solution[var] {
    Val::ValI(i) => println!("int: {}", i),
    Val::ValF(f) => println!("float: {}", f),
}

// Method 4: Use generic get with type annotation (uses GetValue trait)
let val: f64 = solution.get(var);       // Panics if wrong type
let val: Option<f64> = solution.get(var);  // Returns None if wrong type
```

**Explanation:**
- `solution.get_float(var)` returns `f64` (panics if not float)
- `solution.get_int(var)` returns `i32` (panics if not int)  
- `solution[var]` returns `&Val` (use pattern matching)
- `solution.get::<T>(var)` uses the `GetValue<T>` trait
- `Val` is an enum: `Val::ValI(i32)` or `Val::ValF(f64)`

## Files Fixed

### 1. `/home/ross/devpublic/selen/examples/loan_problem.rs`
Hand-crafted standalone Selen program for testing float constraints.

**Changes:**
- ✅ Changed `Model::new()` → `Model::default()`
- ✅ Changed `Solver::new(model)` → `model.solve()`
- ✅ Fixed all 4 `float_times` calls to use `model.mul()` + `model.new()`
- ✅ Fixed solution value extraction to use `solution.get()` with `Val` pattern matching

### 2. `/home/ross/devpublic/zelen/src/exporter.rs`
Auto-generates Selen test programs from FlatZinc AST.

**Changes:**
- ✅ Changed generated `Model::new()` → `Model::default()`
- ✅ Changed generated `Solver::new(model)` → `model.solve()`
- ✅ Fixed generated solution extraction to use `solution.get()` with `Val` matching
- ✅ Fixed unused variable warnings (`element_type`, `init`)

## Verification

### Test Compilation
The loan_problem.rs can now be compiled in the Selen workspace:
```bash
cd /home/ross/devpublic/selen
cargo build --example loan_problem
cargo run --example loan_problem
```

### Test Export Feature
The export feature now generates syntactically correct Selen programs:
```bash
cd /home/ross/devpublic/zelen
cargo run -- /tmp/loan.fzn --export-selen /tmp/exported.rs
# Generated file will have correct Selen API usage
```

## Architecture Notes

### Selen's Expression-Based API
Selen uses an expression-based constraint API:
- Arithmetic operations (`add`, `sub`, `mul`, `div`, `abs`) return expressions
- Comparison operations (`eq`, `lt`, `le`, `gt`, `ge`) return constraints
- Post constraints with `model.new(constraint)`

### Example Pattern
```rust
// Create variables
let a = model.float(0.0, 10.0);
let b = model.float(0.0, 10.0);
let c = model.float(0.0, 100.0);

// c = a * b
let product = model.mul(a, b);
model.new(c.eq(product));

// Or inline:
model.new(c.eq(model.mul(a, b)));
```

### Linear Constraints vs Arithmetic
- **Linear constraints**: Direct methods like `model.float_lin_eq(&coeffs, &vars, constant)`
- **Arithmetic constraints**: Expression-based like `model.new(c.eq(model.mul(a, b)))`
- Both are valid; use what's most natural for the problem

## References

### Source Code
- Zelen's float mapper: `src/mapper/constraints/float.rs` (lines 210-230)
- Zelen's mapper initialization: `src/mapper.rs` (line 633)
- Zelen's output formatting: `src/output.rs` (lines 100-110)

### API Usage Examples
- FlatZinc solver: `src/bin/zelen.rs`
- Integration tests: `src/integration.rs`
- Mapper implementation: `src/mapper.rs`

## Status
✅ **ALL FIXES COMPLETE** - Both loan_problem.rs and exporter.rs now use correct Selen API
