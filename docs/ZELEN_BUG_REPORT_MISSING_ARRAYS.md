# Zelen Export Bug Report: Missing Coefficient Arrays

## Status: agprice_test.rs DOES NOT COMPILE ❌

**File**: `/tmp/agprice_test.rs`  
**Errors**: 284 compilation errors  
**Root Cause**: Parameter arrays (coefficient vectors) are not being generated

---

## The Problem

The exporter correctly identifies that parameter arrays are needed:

```rust
// Array parameter: X_INTRODUCED_212_ (initialization skipped in export)
// Array parameter: X_INTRODUCED_214_ (initialization skipped in export)
// Array parameter: X_INTRODUCED_216_ (initialization skipped in export)
// ... 11 more arrays
```

But then **never actually creates these arrays**. Later in the code, constraints try to use them:

```rust
model.float_lin_eq(&x_introduced_212_, &vec![xm, milk], 1.4);
                    ^^^^^^^^^^^^^^^^^^
                    ERROR: cannot find value `x_introduced_212_` in this scope
```

---

## What Needs to Be Fixed

### Example from MiniZinc
```minizinc
constraint (1.0/4.82)*xm + (0.4/0.297)*milk = 1.4
```

### What's Currently Generated (BROKEN)
```rust
// Array parameter: X_INTRODUCED_212_ (initialization skipped in export)
// ... later in file ...
model.float_lin_eq(&x_introduced_212_, &vec![xm, milk], 1.4);  // ❌ UNDEFINED!
```

### What Should Be Generated (CORRECT)
```rust
let x_introduced_212_ = vec![1.0/4.82, 0.4/0.297];  // ✅ DEFINE THE ARRAY!
// ... later in file ...
model.float_lin_eq(&x_introduced_212_, &vec![xm, milk], 1.4);
```

---

## Complete List of Missing Arrays

Based on the compilation errors, these 14 parameter arrays need to be defined:

1. **x_introduced_212_** - Used in: `model.float_lin_eq(&x_introduced_212_, &vec![xm, milk], 1.4)`
2. **x_introduced_214_** - Used in: `model.float_lin_eq(&x_introduced_214_, &vec![xb, butt], 3.7)`
3. **x_introduced_216_** - Used in: `model.float_lin_eq(&x_introduced_216_, &vec![cha, xca, chb], 2)`
4. **x_introduced_218_** - Used in: `model.float_lin_eq(&x_introduced_218_, &vec![chb, xcb, cha], 1)`
5. **x_introduced_220_** - Used in: `model.float_lin_le(&x_introduced_220_, &vec![xca, xb, xm, xcb], 0.6)`
6. **x_introduced_222_** - Used in: `model.float_lin_le(&x_introduced_222_, &vec![xca, xb, xm, xcb], 0.75)`
7. **x_introduced_224_** - Used in: `model.float_lin_le(&x_introduced_224_, &vec![cha, butt, milk, chb], 1.939)`
8. **x_introduced_226_** - Used in: `model.float_lin_eq(&x_introduced_226_, &vec![chb, cha, q], -0)`
9. **x_introduced_301_** - Used in: `model.float_lin_eq(&x_introduced_301_, &x_introduced_300_, -0)`
10. **x_introduced_491_** - Used in: `model.float_lin_eq(&x_introduced_491_, &x_introduced_490_, -0)`
11. **x_introduced_563_** - Used in: `model.float_lin_eq(&x_introduced_563_, &x_introduced_562_, -0)`
12. **x_introduced_753_** - Used in: `model.float_lin_eq(&x_introduced_753_, &x_introduced_752_, -0)`
13. **x_introduced_755_** - Used in: `model.float_lin_le(&x_introduced_755_, &lmilk, 1)`
14. **x_introduced_798_** - Used in: `model.float_lin_le(&x_introduced_798_, &x_introduced_797_, 1.0)`

---

## Where to Place the Definitions

The coefficient arrays should be defined **after the imports but before any model.float() calls**, typically right after `let mut model = Model::default();`

### Recommended Structure:
```rust
use selen::prelude::*;
use selen::variables::Val;

fn main() {
    let mut model = Model::default();

    // ===== PARAMETER ARRAYS (COEFFICIENT VECTORS) =====
    let x_introduced_212_ = vec![1.0/4.82, 0.4/0.297];
    let x_introduced_214_ = vec![1.0/0.32, 2.7/0.720];
    let x_introduced_216_ = vec![1.0/0.21, 1.1/1.05, -0.1/0.815];
    // ... etc for all 14 arrays
    
    // ===== VARIABLES =====
    let milk = model.float(f64::NEG_INFINITY, f64::INFINITY);
    // ... rest of the code
```

---

## How to Extract Coefficients from FlatZinc

Look for the FlatZinc constraint declarations. For example:

```
constraint float_lin_eq([0.20746887966804978, 1.3468013468013468], [xm, milk], 1.4);
```

Should generate:
```rust
let x_introduced_212_ = vec![0.20746887966804978, 1.3468013468013468];
model.float_lin_eq(&x_introduced_212_, &vec![xm, milk], 1.4);
```

The coefficient array `[0.20746887966804978, 1.3468013468013468]` corresponds to `[1.0/4.82, 0.4/0.297]` from the MiniZinc source.

---

## Verification Steps

After fixing the exporter:

1. ✅ File should compile without errors
2. ✅ Run the compiled binary and check output
3. ✅ Verify `cha` value is around 10.0 (not 0 or astronomical)
4. ✅ All variables should have finite, reasonable values
5. ✅ Revenue should be finite and positive

---

## Current Export Progress

✅ **FIXED**: Constraint structure (float_lin_eq, float_lin_le calls are correct)  
✅ **FIXED**: Variable declarations (all model.float() calls present)  
✅ **FIXED**: Array variable groupings (x_introduced_300_, etc.)  
❌ **BROKEN**: Coefficient array definitions (all marked as "initialization skipped")  

**Once the coefficient arrays are defined, the export should be complete and working!**

---

**Report Date**: October 7, 2025  
**Reported By**: Selen Testing Team  
**File**: `/tmp/agprice_test.rs`  
**Urgency**: HIGH - File does not compile
