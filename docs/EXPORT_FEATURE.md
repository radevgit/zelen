# Export Feature - Generate Selen Test Programs from FlatZinc

**Feature**: `--export-selen` flag in Zelen
**Purpose**: Export any FlatZinc problem as a standalone Selen Rust program for debugging

---

## Usage

```bash
# Export a FlatZinc problem to a Rust file
./zelen --export-selen output.rs input.fzn

# Example: Export the loan problem
./zelen --export-selen /tmp/loan_debug.rs /tmp/loan.fzn
```

This generates a standalone Rust program that can be:
1. Compiled independently: `rustc --edition 2021 output.rs -L ../selen/target/release/deps`
2. Added to Selen's examples: `cp output.rs ../selen/examples/`
3. Run with cargo: `cd ../selen && cargo run --example loan_problem`

---

## What Gets Exported

The exported program contains:
- ✅ All variable declarations with correct types and bounds
- ✅ Variable names as Rust identifiers (sanitized)
- ⚠️  Constraint scaffolding (TODOs for full implementation)
- ✅ Solve invocation and solution printing

---

## Hand-Crafted Example

A **complete, hand-crafted** version is available:
**`/home/ross/devpublic/selen/examples/loan_problem.rs`**

This file has:
- ✅ All 11 float variables properly created
- ✅ All 9 constraints (float_lin_eq, float_times) implemented
- ✅ Expected solution values from Coin-BC
- ✅ Verification checks and error analysis
- ✅ Detailed comments explaining each constraint

Run it in Selen:
```bash
cd /home/ross/devpublic/selen
cargo run --example loan_problem
```

---

## Benefits for Debugging

### 1. Reproduce Issues in Selen
Test float problems directly without FlatZinc parser:
```rust
// Instead of debugging through Zelen → FlatZinc → Selen
// Debug directly: Selen native API
let i = model.float(0.0, 10.0);
model.float_lin_eq(&[1.0, -1.0], &[i, x1], -1.0);
```

### 2. Isolate Bound Inference Issues
```rust
// Test with different bound strategies
let p1 = model.float(f64::NEG_INFINITY, f64::INFINITY);  // Current
let p2 = model.float(-10000.0, 10000.0);                  // Conservative
let p3 = model.float(100.0, 10000.0);                     // Realistic
```

### 3. Compare Solutions
Expected vs Actual side-by-side:
```
EXPECTED:          ACTUAL:
P  ≈ 1000.00       P  = -20010000.0
I  ≈ 4.00          I  = 0.0000009999
R  ≈ 260.00        R  = -10000.0
B4 ≈ 65.78         B4 = -19970079.98
```

### 4. Verify Constraints Manually
```rust
println!("X1 = I + 1?  {:.4} vs {:.4}", x1_val, i_val + 1.0);
// Check if constraints are actually satisfied
```

---

## Future Enhancements

### Current Limitations
The auto-exported file has:
- ⚠️  Constraint TODOs (not fully implemented)
- ⚠️  No array parameter support
- ⚠️  No optimization objective handling

### Proposed Full Implementation
Generate complete constraint implementations:
```rust
// Current (TODO)
// TODO: Implement constraint: float_lin_eq with 3 args

// Future (Full)
model.float_lin_eq(
    &[1.0, -1.0, 1.0],
    &[b1, x2, r],
    0.0
);
```

This would require:
1. Parsing constraint arguments fully
2. Mapping Expr to Rust values
3. Generating correct Selen API calls

---

## Implementation Details

### Files Modified
- **`src/bin/zelen.rs`**: Added `--export-selen` CLI flag
- **`src/solver.rs`**: Added `export_selen_program()` method, stores AST
- **`src/exporter.rs`**: New module for export logic
- **`src/error.rs`**: Added `From<std::io::Error>` conversion
- **`src/lib.rs`**: Registered exporter module

### Key Design Decisions
1. **Store AST in solver**: Enables export without re-parsing
2. **Sanitize names**: Convert FlatZinc names to valid Rust identifiers
3. **Preserve bounds**: Export exact bounds from FlatZinc
4. **Comments for TODOs**: Make it easy to complete manually

---

## Testing

```bash
# 1. Export the loan problem
cd /home/ross/devpublic/zelen
./target/release/zelen --export-selen /tmp/loan_auto.rs /tmp/loan.fzn

# 2. Check the auto-generated structure
head -30 /tmp/loan_auto.rs

# 3. Compare with hand-crafted version
ls -lh /home/ross/devpublic/selen/examples/loan_problem.rs

# 4. Run hand-crafted version in Selen
cd /home/ross/devpublic/selen
cargo run --example loan_problem
```

---

## Use Cases

### Use Case 1: Debug Bound Inference
1. Export problem: `zelen --export-selen test.rs problem.fzn`
2. Modify bounds in test.rs
3. Run and compare results

### Use Case 2: Reproduce Zelen Bug in Selen
1. Find problematic FlatZinc file
2. Export to Rust
3. Debug directly in Selen without parser overhead

### Use Case 3: Create Selen Test Suite
1. Export interesting FlatZinc problems
2. Add to `selen/examples/`
3. Use as regression tests

### Use Case 4: Verify Constraint Semantics
1. Export simple constraint test
2. Manually verify solution
3. Compare with MiniZinc/Gecode results

---

## Example: Loan Problem

**Input**: `/tmp/loan.fzn` (17 lines)
**Output**: `/home/ross/devpublic/selen/examples/loan_problem.rs` (185 lines)

**Result**: Standalone Selen program that:
- Creates 11 float variables (7 unbounded, 4 bounded)
- Posts 9 constraints (float_lin_eq × 5, float_times × 4)
- Solves and verifies solution
- Compares with expected Coin-BC values
- Identifies bound inference issues

**Status**: Ready to run in Selen for debugging!

---

## Documentation

See also:
- `/home/ross/devpublic/selen/UNBOUNDED_FLOAT_VARIABLES.md` - Bound inference requirements
- `/home/ross/devpublic/zelen/INTEGRATION_COMPLETE.md` - Overall integration status
- `/home/ross/devpublic/zelen/SELEN_COMPLETE_STATUS.md` - Feature verification

---

## Summary

✅ **Feature implemented and working**
✅ **Hand-crafted example ready for Selen testing**
✅ **Enables isolated debugging of float problems**
✅ **Can be enhanced for full auto-generation**

**Next Steps**: 
1. Run `loan_problem.rs` in Selen
2. Debug bound inference with real problem
3. Iterate on solutions
4. Optionally enhance auto-export to generate complete constraints
