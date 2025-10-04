# Selen Float Constraints Implementation Status

**Date**: October 4, 2025  
**Selen Location**: ../selen (local copy)

## ✅ IMPLEMENTED - Critical (P0)

### Float Linear Constraints
- ✅ `float_lin_eq(&[f64], &[VarId], f64)` - Line 751
- ✅ `float_lin_le(&[f64], &[VarId], f64)` - Line 783  
- ✅ `float_lin_ne(&[f64], &[VarId], f64)` - Line 815

### Float Linear Reified Constraints
- ✅ `float_lin_eq_reif(&[f64], &[VarId], f64, VarId)` - Line 848
- ✅ `float_lin_le_reif(&[f64], &[VarId], f64, VarId)` - Line 886
- ✅ `float_lin_ne_reif(&[f64], &[VarId], f64, VarId)` - Line 924

### Float Array Aggregations
- ✅ `array_float_minimum(&[VarId]) -> SolverResult<VarId>` - Line 1218
- ✅ `array_float_maximum(&[VarId]) -> SolverResult<VarId>` - Line 1246
- ✅ `array_float_element(VarId, &[VarId], VarId)` - Line 1284

### Float/Int Conversions (BONUS!)
- ✅ `int2float(VarId, VarId)` - Line 966
- ✅ `float2int_floor(VarId, VarId)` - Line 1006
- ✅ `float2int_ceil(VarId, VarId)` - Line 1047
- ✅ `float2int_round(VarId, VarId)` - Line 1088

---

## ❌ NOT IMPLEMENTED - But May Not Be Critical

### Simple Float Comparison Reified Constraints
These are NOT found in Selen:
- ❌ `float_eq_reif(VarId, VarId, VarId)` - reif ⇔ (x == y)
- ❌ `float_ne_reif(VarId, VarId, VarId)` - reif ⇔ (x != y)
- ❌ `float_lt_reif(VarId, VarId, VarId)` - reif ⇔ (x < y)
- ❌ `float_le_reif(VarId, VarId, VarId)` - reif ⇔ (x <= y)
- ❌ `float_gt_reif(VarId, VarId, VarId)` - reif ⇔ (x > y)
- ❌ `float_ge_reif(VarId, VarId, VarId)` - reif ⇔ (x >= y)

### Integer Linear Not-Equal
- ❌ `int_lin_ne(&[i32], &[VarId], i32)` - Integer linear ≠

---

## 🤔 Assessment: Do We Need The Missing Ones?

### Simple Float Comparison Reified

**Can be worked around** using linear versions:
```rust
// float_eq_reif(x, y, b) can be expressed as:
// b <=> (1.0*x + -1.0*y == 0.0)
model.float_lin_eq_reif(&[1.0, -1.0], &[x, y], 0.0, b);

// float_lt_reif(x, y, b) can be expressed as:
// b <=> (1.0*x + -1.0*y < 0.0)
// Which is equivalent to: b <=> (1.0*x + -1.0*y <= -epsilon)
// But epsilon handling is tricky...
```

**FlatZinc Usage Check**: Let me check if these are commonly used in FlatZinc...

Looking at the FlatZinc spec:
- **Integer** has: `int_eq_reif`, `int_ne_reif`, `int_lt_reif`, `int_le_reif`, etc.
- **Float** equivalents would be: `float_eq_reif`, `float_ne_reif`, etc.

**Recommendation**: 
- ⚠️ **NICE TO HAVE** but not critical
- Can be emulated using `float_lin_*_reif` with coefficients [1.0, -1.0]
- Would make zelen implementation cleaner
- **Priority**: P2 (Medium) - Add if you have time, but not blocking

### Integer Linear Not-Equal

**Current Workaround in Zelen**:
```rust
// int_lin_ne is implemented using intermediate variables
let scaled_vars = coeffs.zip(vars).map(|(c,v)| model.mul(v, c)).collect();
let sum = model.sum(&scaled_vars);
model.c(sum).ne(constant);
```

**Recommendation**:
- ⚠️ **OPTIMIZATION** - Current workaround works but is verbose
- Would be more efficient as native constraint
- **Priority**: P2 (Medium) - Nice optimization, not critical

---

## ✅ CONCLUSION: Ready to Integrate!

### What Selen Has Implemented:

**ALL P0 (CRITICAL) constraints are implemented:**
- ✅ Float linear constraints (eq, le, ne)
- ✅ Float linear reified constraints  
- ✅ Float array aggregations
- ✅ BONUS: Float/int conversions (very useful!)

### What's Missing (P2 - Medium Priority):

1. **Simple float comparison reified** - Can work around with linear versions
2. **int_lin_ne** - Can work around with intermediate variables

### Verdict:

🎉 **SELEN IS READY!** The critical (P0) features are all implemented.

The missing P2 features are:
- Not blocking any functionality
- Can be added later for optimization/convenience
- Current workarounds are acceptable

---

## 📋 Next Steps for Zelen

1. **Update Cargo.toml** to point to local Selen:
   ```toml
   selen = { path = "../selen" }
   ```

2. **Remove scaling workarounds** from `src/mapper/constraints/float.rs`:
   - Delete the `SCALE = 1000.0` approach
   - Call native Selen methods directly

3. **Add missing FlatZinc constraints** (simple float reified):
   - Implement using `float_lin_*_reif` workaround
   - Document that they use linear constraint decomposition

4. **Test with loan.fzn** to verify float support works!

5. **Run full test suite** to ensure nothing broke

---

## 🔧 Implementation Recommendations for Missing P2 Features

### If you want to add simple float comparison reified:

```rust
// In selen/src/model/constraints.rs

/// Reified float equality: reif_var <=> (x == y)
pub fn float_eq_reif(&mut self, x: VarId, y: VarId, reif_var: VarId) {
    // Decompose to: reif <=> (1.0*x + -1.0*y == 0.0)
    self.float_lin_eq_reif(&[1.0, -1.0], &[x, y], 0.0, reif_var);
}

/// Reified float not-equal: reif_var <=> (x != y)  
pub fn float_ne_reif(&mut self, x: VarId, y: VarId, reif_var: VarId) {
    // Decompose to: reif <=> (1.0*x + -1.0*y != 0.0)
    self.float_lin_ne_reif(&[1.0, -1.0], &[x, y], 0.0, reif_var);
}

// float_lt_reif and float_le_reif are trickier due to strict inequality
// May need special handling for floating point epsilon
```

### If you want to add int_lin_ne:

```rust
/// Integer linear not-equal: sum(coeffs[i] * vars[i]) != constant
pub fn int_lin_ne(&mut self, coefficients: &[i32], variables: &[VarId], constant: i32) {
    // Create intermediate variables for scaled values
    let scaled: Vec<VarId> = coefficients
        .iter()
        .zip(variables.iter())
        .map(|(&c, &v)| self.mul(v, Val::ValI(c)))
        .collect();
    
    // Sum and post not-equal constraint
    let sum = self.sum(&scaled);
    self.c(sum).ne(constant);
}
```

**Effort**: ~30 minutes each, very straightforward

---

## Summary

✅ **Selen implementation is EXCELLENT!** All critical features done.

🎯 **Zelen can now**:
- Remove broken scaling workarounds
- Use native float constraints
- Support float FlatZinc problems correctly

⚠️ **Minor additions recommended** (P2, not blocking):
- Simple float comparison reified (6 methods)
- int_lin_ne (1 method)

**Total**: 7 convenience methods, ~3-4 hours work, but NOT required for functionality.

---

**Ready to proceed with Zelen integration! 🚀**
