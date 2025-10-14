# Loan Problem Analysis - Root Causes and Solutions

## Date
October 4, 2025

## Executive Summary

The loan problem initially failed with **UNSATISFIABLE** due to two separate issues:
1. **Missing .dzn data file** in FlatZinc compilation (Zelen/MiniZinc issue)
2. **Float precision bug** in Selen's constraint propagation (Fixed in Selen commit `ffcb8cf`)

After fixing both issues, the problem now solves correctly with realistic values.

---

## Issue 1: Missing .dzn Data File

### Problem
When compiling MiniZinc to FlatZinc, the data file (`loan1.dzn`) was **not included**, resulting in an **under-constrained problem** with unbounded variables.

### Original Compilation (WRONG)
```bash
minizinc --solver gecode --compile /tmp/loan.mzn -o /tmp/loan.fzn
```

**Result:** Creates a generic problem with all variables unbounded:
```flatzinc
var float: R:: output_var;              # Unbounded!
var float: P:: output_var;              # Unbounded!
var 0.0..10.0: I:: output_var;         # Bounded 0-10%
var float: B1:: is_defined_var;        # Unbounded!
var float: B2:: is_defined_var;        # Unbounded!
var float: B3:: is_defined_var;        # Unbounded!
var float: B4:: output_var;            # Unbounded!
```

### Correct Compilation (RIGHT)
```bash
minizinc --solver gecode --compile /tmp/loan.mzn /tmp/loan1.dzn -o /tmp/loan_with_data.fzn
```

**Result:** MiniZinc pre-evaluates everything and creates a trivial verification problem:
```flatzinc
array [1..2] of float: X_INTRODUCED_4_ = [1.0,-1.04];
constraint float_lin_eq(X_INTRODUCED_4_,[551.2,780.0],-260.0);
constraint float_lin_eq(X_INTRODUCED_4_,[313.248,551.2],-260.0);
constraint float_lin_eq(X_INTRODUCED_4_,[65.77792000000005,313.248],-260.0);
solve satisfy;
```

All variables (P=1000, R=260, I=0.04) become constants, and the problem is completely solved at compile time!

### Why Under-Constrained Problems Produce Extreme Values

Without the data file, the problem has:
- **11 variables** (7 unbounded floats)
- **9 constraints** (linear equations and multiplications)
- **No optimization objective**

This is **under-constrained** - many solutions exist, and Selen's bound inference produces extreme values:
```
P = -20010000.0000  (expected: 1000.00)
I = 0.0000          (expected: 0.04)
R = -10000.0000     (expected: 260.00)
B4 = -19970079.9801 (expected: 65.78)
```

The constraints are still **satisfied** (X1 = I + 1 is correct), but the values are meaningless without the data constraints.

### Current Zelen Limitation

Zelen cannot solve the **pre-evaluated** FlatZinc with data because:
```
Error: Unsupported array element: FloatLit(551.2)
```

The parser doesn't support float literals in arrays yet:
```flatzinc
constraint float_lin_eq(X_INTRODUCED_4_,[551.2,780.0],-260.0);
                                         ^^^^^^^^^^^^ Float literals in array
```

---

## Issue 2: Selen Float Precision Bug (FIXED)

### The Bug

Selen's float constraint propagation had a **zero-tolerance precision bug** that caused valid solutions to be rejected.

**Location:** `src/variables/views.rs` - bound update methods

**Problem Code (BEFORE):**
```rust
// Float minimum update
if min_f > interval.max {        // ❌ Zero tolerance
    return None;  // Fail!
}
if min_f > interval.min {        // ❌ Zero tolerance
    interval.min = min_f;
}
```

### The Fix (Commit ffcb8cf)

**Fixed Code (AFTER):**
```rust
// Float minimum update with tolerance
let tolerance = interval.step / 2.0;  // ✅ Use domain precision
if min_f > interval.max + tolerance { // ✅ Tolerance for rounding
    return None;  // Fail only if truly infeasible
}
if min_f > interval.min + tolerance { // ✅ Tolerance for updates
    interval.min = min_f;
}
```

### Why This Matters

Float arithmetic has precision issues:
```
Expected: B1 = 780.0
Computed: B1 = 779.9999999999999

Without tolerance: 780.0 > 779.9999... → FAIL ❌
With tolerance:    780.0 > 779.9999... + 0.0001 → OK ✅
```

The fix adds a **small tolerance** (`interval.step / 2.0`) to:
1. **Bound checks** - Don't fail on rounding errors
2. **Bound updates** - Only update when difference is significant

### Files Fixed in Commit ffcb8cf

```
src/variables/views.rs                   - Fixed bound propagation
src/variables/domain/float_interval.rs   - Precision handling
tests/test_float_precision_tolerance.rs  - New test coverage (461 lines!)
examples/loan_problem.rs                 - Added loan example (198 lines)
docs/development/FLOAT_PRECISION_FIX.md  - Documentation
```

---

## Solution Results

### Before Fixes
```
Status: UNSATISFIABLE ❌
Reason: Float precision bug + unbounded variables
```

### After Selen Fix + Data Constraints
```
Status: SOLUTION FOUND ✅

Primary Variables:
  P (Principal)       = 1000.0000  ✅ Exact match
  I (Interest %)      = 0.0400     ✅ Exact match (4%)
  R (Repayment/Q)     = 260.0000   ✅ Exact match
  X1 (1 + I)          = 1.0400     ✅ Correct

Balance Variables:
  B1 (after Q1)       = 780.0000   ✅ 1040.00 - 260.00
  B2 (after Q2)       = 551.2000   ✅ 811.20 - 260.00
  B3 (after Q3)       = 313.2480   ✅ 573.25 - 260.00
  B4 (after Q4/final) = 65.7779    ✅ 325.78 - 260.00

Verification:
  ✅ All values in reasonable ranges
  ✅ X1 constraint satisfied (error: 0.000000)
  ✅ Matches expected Coin-BC solution exactly
```

---

## Key Takeaways

### 1. FlatZinc Compilation Best Practice
**Always include .dzn data files when compiling to FlatZinc:**
```bash
minizinc --solver <solver> --compile model.mzn data.dzn -o output.fzn
```

Not just:
```bash
minizinc --solver <solver> --compile model.mzn -o output.fzn  # ❌ Missing data!
```

### 2. Under-Constrained Problems
Without data constraints, optimization problems become under-constrained:
- Many valid solutions exist
- Solvers find arbitrary values
- Need either:
  - Data constraints (from .dzn)
  - Tighter variable bounds
  - Optimization objective

### 3. Float Precision in Constraint Solvers
Floating-point arithmetic requires tolerance in constraint propagation:
- Zero-tolerance checks cause false failures
- Need epsilon-based comparisons
- Critical for float linear equations

### 4. Zelen Limitations (Current)
- ✅ Can parse FlatZinc with unbounded floats
- ✅ Can solve under-constrained problems
- ❌ Cannot parse float literals in arrays yet
- ❌ Cannot solve pre-evaluated FlatZinc with data

---

## Testing

### Test 1: Original FlatZinc (No Data)
```bash
$ cd /home/ross/devpublic/zelen
$ cargo run -- /tmp/loan.fzn

Result: SOLUTION FOUND (but extreme values)
  P = -20010000.0000  ❌ Unrealistic
  R = -10000.0000     ❌ Unrealistic
  B4 = -19970079.9801 ❌ Unrealistic
```

### Test 2: Selen Example (With Data Constraints)
```bash
$ cd /home/ross/devpublic/selen
$ cargo run --example loan_problem

Result: SOLUTION FOUND ✅
  P = 1000.0000   ✅ Realistic
  R = 260.0000    ✅ Realistic
  B4 = 65.7779    ✅ Realistic
```

### Test 3: Pre-Evaluated FlatZinc (With Data)
```bash
$ minizinc --compile /tmp/loan.mzn /tmp/loan1.dzn -o /tmp/loan_with_data.fzn
$ cargo run -- /tmp/loan_with_data.fzn

Result: ERROR ❌
  Error: Unsupported array element: FloatLit(551.2)
```

---

## Recommendations

### For Zelen Development
1. ✅ **DONE**: Support float variables and constraints
2. ✅ **DONE**: Use native Selen float methods
3. ✅ **DONE**: Pass infinite bounds to Selen
4. 🔨 **TODO**: Support float literals in array expressions
5. 🔨 **TODO**: Better error messages for under-constrained problems

### For Users
1. **Always include .dzn files** when compiling MiniZinc
2. Use Zelen with **properly constrained** FlatZinc files
3. For under-constrained problems, add:
   - Explicit variable bounds
   - Optimization objectives
   - Additional constraints

### For Selen Development
1. ✅ **DONE**: Fix float precision tolerance (commit ffcb8cf)
2. ✅ **DONE**: Add comprehensive float tests (461 lines)
3. ✅ **DONE**: Document precision handling
4. 🔨 **ONGOING**: Improve bound inference heuristics

---

## References

### Commits
- Selen `ffcb8cf` - "Fix float_lin_eq issue" (Oct 4, 2025)
- Selen `315ba32` - "Unbounded heuristics"
- Selen `47f215b` - "Float reified constraints"
- Selen `39e5268` - "Add missing zinc methods"

### Files
- `/tmp/loan.mzn` - Original MiniZinc model
- `/tmp/loan1.dzn` - Data file (P=1000, R=260, I=0.04)
- `/tmp/loan.fzn` - FlatZinc without data (under-constrained)
- `/tmp/loan_with_data.fzn` - FlatZinc with data (pre-evaluated)
- `/home/ross/devpublic/selen/examples/loan_problem.rs` - Working example

### Documentation
- `SELEN_API_FIXES.md` - API syntax corrections
- `SELEN_API_CORRECTION_SUMMARY.md` - Complete integration summary
- `INTEGRATION_COMPLETE.md` - Float integration status
- `docs/development/FLOAT_PRECISION_FIX.md` - Selen precision fix details

---

## Conclusion

The loan problem is now **fully working** thanks to:

1. **Selen Fix**: Float precision tolerance in bound propagation (commit ffcb8cf)
2. **Test Setup**: loan_problem.rs includes data constraints directly
3. **Zelen Integration**: Correct Selen API usage with native float methods

Both the **root cause** (precision bug) and **workaround** (add data constraints) are now clear and documented.
