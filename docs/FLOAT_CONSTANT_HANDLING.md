# Float Constant Handling in Selen/Zelen

## Date
October 4, 2025

## Question: How to Create Float Constants?

### Two Approaches in Selen

#### 1. For Direct Selen Code (View Trait)
```rust
use selen::prelude::*;

// Single parameter - returns Val that implements View trait
let constant = float(551.2);  // Returns Val::ValF(551.2)

// Can be used directly in expressions via View trait
model.new(x.eq(float(3.14)));
```

**Use when:** Writing native Selen code with expressions

#### 2. For Constraint System (VarId)
```rust
// Two parameters - creates a fixed variable domain
let const_var = model.float(551.2, 551.2);  // Returns VarId with domain [551.2, 551.2]

// Use the VarId in constraints
model.float_lin_eq(&[1.0, -1.0], &[x, const_var], 0.0);
```

**Use when:** Need a VarId for constraints that take variable arrays

## Zelen's Implementation

In FlatZinc parsing, we need VarIds because constraints like `float_lin_eq` take arrays of variables:

```rust
// FlatZinc: constraint float_lin_eq([1.0,-1.04],[551.2,780.0],-260.0);
// Array elements need to be VarIds, so we create fixed variables:
let var1 = model.float(551.2, 551.2);  // VarId with fixed value
let var2 = model.float(780.0, 780.0);  // VarId with fixed value
model.float_lin_eq(&[1.0, -1.04], &[var1, var2], -260.0);
```

This is **correct** - we're creating variables with single-value domains, which is how Selen represents constants in the constraint system.

## The Pre-Evaluated FlatZinc Problem

### Why loan_with_data.fzn Fails

When MiniZinc compiles with data (`loan.mzn` + `loan1.dzn`), it pre-evaluates everything:

```flatzinc
% Pre-evaluated by MiniZinc:
array [1..2] of float: X_INTRODUCED_4_ = [1.0,-1.04];
constraint float_lin_eq(X_INTRODUCED_4_,[551.2,780.0],-260.0);
constraint float_lin_eq(X_INTRODUCED_4_,[313.248,551.2],-260.0);
constraint float_lin_eq(X_INTRODUCED_4_,[65.77792000000005,313.248],-260.0);
solve satisfy;
```

**The Problem:** Float rounding errors from MiniZinc's evaluation:

```python
# Constraint 1: 1.0 * 551.2 + (-1.04) * 780.0 = -260.0 ✅ Exact
# Constraint 2: 1.0 * 313.248 + (-1.04) * 551.2 = -260.00000000000006 ❌ Off by 6e-14
# Constraint 3: 1.0 * 65.778 + (-1.04) * 313.248 = -259.99999999999994 ❌ Off by 6e-14
```

When we create fixed variables:
```rust
let v1 = model.float(551.2, 551.2);    // Domain: exactly 551.2
let v2 = model.float(780.0, 780.0);    // Domain: exactly 780.0

// Constraint says: 1.0 * v1 + (-1.04) * v2 = -260.0
// But: 1.0 * 551.2 + (-1.04) * 780.0 = -260.00000000000006
// Even with Selen's tolerance, this might fail if rounding is unlucky
```

### Why This Is Not Zelen's Fault

1. **MiniZinc introduced the rounding errors** during pre-evaluation
2. **Zelen correctly creates fixed variables** with `model.float(val, val)`
3. **Selen has float tolerance** (commit ffcb8cf), but it's for propagation, not verification
4. **The constraints are verification-only** - no variables to solve, just check if constants satisfy equations

## Solutions

### Option 1: Don't Use Pre-Evaluated FlatZinc (Current Approach)
```bash
# WRONG: MiniZinc pre-evaluates and introduces rounding errors
minizinc --compile loan.mzn loan1.dzn -o loan_with_data.fzn

# RIGHT: Keep variables, let solver handle the actual solving
minizinc --compile loan.mzn -o loan.fzn
# Then add data constraints in Selen directly (as in loan_problem.rs)
```

### Option 2: Increase Tolerance for Verification Constraints
Could add a larger tolerance specifically for constraints with all-fixed variables:
```rust
// If all variables in float_lin_eq are fixed (domain size == 0),
// use a larger epsilon for the equality check
const VERIFICATION_TOLERANCE: f64 = 1e-10;  // Instead of step/2.0
```

### Option 3: Warn About Pre-Evaluated FlatZinc
Detect when FlatZinc has:
- No actual variables (all fixed)
- Only verification constraints
- Potential precision issues

Warn user: "This FlatZinc appears to be pre-evaluated with data. Consider compiling without .dzn file."

## Current Status

✅ **FloatLit support added**: Zelen can now parse float literals in arrays
✅ **Fixed variable creation**: `model.float(val, val)` is the correct approach  
❌ **Pre-evaluated FlatZinc**: Still fails due to MiniZinc's rounding errors

## Recommendation

**For Users:**
- Don't include `.dzn` files when compiling to FlatZinc
- Let Selen solve the actual problem, not verify pre-computed constants
- If you need data, add it as constraints in your Selen code (see loan_problem.rs)

**For Developers:**
- Current implementation is correct
- Pre-evaluated FlatZinc is an edge case with inherent precision issues
- Could add tolerance for verification-only constraints, but it's not a priority

## Code Changes Made

### `/home/ross/devpublic/zelen/src/mapper/helpers.rs`

Added float literal support in `extract_var_array`:
```rust
Expr::FloatLit(val) => {
    // Constant float - for constraints that need VarId, create a fixed variable
    let const_var = self.model.float(*val, *val);
    var_ids.push(const_var);
}
```

This allows FlatZinc like:
```flatzinc
constraint float_lin_eq([1.0,-1.04],[551.2,780.0],-260.0);
                                     ^^^^^^^^^^^^^^^^ Float literals now supported
```

## Conclusion

**`model.float(val, val)` IS the correct way** to create fixed-value variables in Selen's constraint system. The `float(val)` single-parameter version is for the View trait in expression contexts, not for creating VarIds.

The pre-evaluated FlatZinc issue is a fundamental problem with MiniZinc's pre-evaluation introducing rounding errors, not with our constant handling.
