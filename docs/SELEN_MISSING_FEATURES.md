# Missing Features in Selen for Full FlatZinc Support

**Context**: Zelen was tested against ~900 real FlatZinc examples and achieved 95% coverage with integer constraints. However, float constraint support is incomplete in Selen.

**Date**: October 4, 2025  
**Zelen Version**: 0.1.1  
**Selen Version**: 0.9.1

---

## 1. Float Linear Constraints (CRITICAL)

### Missing from Selen's Model API:

Currently Selen has:
- ✅ `int_lin_eq(&[i32], &[VarId], i32)` - Integer linear equality
- ✅ `int_lin_le(&[i32], &[VarId], i32)` - Integer linear ≤

### NEEDED in Selen:

```rust
// In selen/src/model/constraints.rs
impl Model {
    /// Linear equality constraint with float coefficients
    /// sum(coefficients[i] * variables[i]) == constant
    pub fn float_lin_eq(&mut self, coefficients: &[f64], variables: &[VarId], constant: f64);
    
    /// Linear inequality constraint with float coefficients
    /// sum(coefficients[i] * variables[i]) <= constant
    pub fn float_lin_le(&mut self, coefficients: &[f64], variables: &[VarId], constant: f64);
    
    /// Linear inequality constraint with float coefficients (not-equal)
    /// sum(coefficients[i] * variables[i]) != constant
    pub fn float_lin_ne(&mut self, coefficients: &[f64], variables: &[VarId], constant: f64);
    
    /// Reified float linear equality
    /// reif_var <=> sum(coefficients[i] * variables[i]) == constant
    pub fn float_lin_eq_reif(&mut self, coefficients: &[f64], variables: &[VarId], constant: f64, reif_var: VarId);
    
    /// Reified float linear inequality
    /// reif_var <=> sum(coefficients[i] * variables[i]) <= constant
    pub fn float_lin_le_reif(&mut self, coefficients: &[f64], variables: &[VarId], constant: f64, reif_var: VarId);
}
```

### Why Critical:

- **FlatZinc Spec Section 4.2.3** lists `float_lin_eq` and `float_lin_le` as standard builtins
- **Used extensively** in optimization problems (loan calculations, physics simulations, etc.)
- **Cannot be decomposed** efficiently - needs native solver support
- **Current workaround** (scaling floats to integers by 1000x) is:
  - ❌ Loses precision
  - ❌ Causes overflow for large values
  - ❌ Incorrect semantics

### Example FlatZinc requiring these:

```flatzinc
% From loan.fzn - financial calculation
array [1..3] of float: coeffs = [1.0, -1.0, 1.0];
var float: B1;
var float: X;
var float: R;
constraint float_lin_eq(coeffs, [B1, X, R], 0.0);
```

**Status**: Currently causing `=====UNSATISFIABLE=====` or wrong results due to scaling workaround.

---

## 2. Float Comparison Reified Constraints

### Missing from Selen:

```rust
// In selen/src/model/constraints.rs
impl Model {
    /// Reified float equality: reif_var <=> (x == y)
    pub fn float_eq_reif(&mut self, x: VarId, y: VarId, reif_var: VarId);
    
    /// Reified float not-equal: reif_var <=> (x != y)
    pub fn float_ne_reif(&mut self, x: VarId, y: VarId, reif_var: VarId);
    
    /// Reified float less-than: reif_var <=> (x < y)
    pub fn float_lt_reif(&mut self, x: VarId, y: VarId, reif_var: VarId);
    
    /// Reified float less-equal: reif_var <=> (x <= y)
    pub fn float_le_reif(&mut self, x: VarId, y: VarId, reif_var: VarId);
    
    /// Reified float greater-than: reif_var <=> (x > y)
    pub fn float_gt_reif(&mut self, x: VarId, y: VarId, reif_var: VarId);
    
    /// Reified float greater-equal: reif_var <=> (x >= y)
    pub fn float_ge_reif(&mut self, x: VarId, y: VarId, reif_var: VarId);
}
```

### Why Needed:

- Used in conditional constraints with floats
- Required for proper float constraint reification
- Common in optimization problems

---

## 3. Integer Linear Constraints - Missing Variant

### Missing from Selen:

```rust
impl Model {
    /// Integer linear not-equal constraint
    /// sum(coefficients[i] * variables[i]) != constant
    pub fn int_lin_ne(&mut self, coefficients: &[i32], variables: &[VarId], constant: i32);
}
```

### Current Workaround in Zelen:

```rust
// Works but verbose - requires creating intermediate variables
let scaled_vars: Vec<VarId> = coeffs
    .iter()
    .zip(vars.iter())
    .map(|(&coeff, &var)| self.model.mul(var, Val::ValI(coeff)))
    .collect();
let sum_var = self.model.sum(&scaled_vars);
self.model.c(sum_var).ne(constant);
```

**Better**: Native `int_lin_ne` would be more efficient.

---

## 4. Array Float Aggregation Constraints

### Missing from Selen:

```rust
impl Model {
    /// Float array minimum: result = min(array)
    pub fn array_float_minimum(&mut self, result: VarId, array: &[VarId]);
    
    /// Float array maximum: result = max(array)
    pub fn array_float_maximum(&mut self, result: VarId, array: &[VarId]);
}
```

### Current Status:

- ✅ `array_int_minimum` exists
- ✅ `array_int_maximum` exists
- ❌ Float versions missing

**FlatZinc Spec Reference**: Section 4.2.3 lists these as standard builtins added in MiniZinc 2.0.

---

## 5. Implementation Notes for Selen

### Float Variable Representation

Current Selen implementation:
```rust
// selen/src/variables/domain/float_interval.rs exists
pub fn float(&mut self, min: f64, max: f64) -> VarId
```

This suggests Selen uses **interval-based float domains**. The missing constraints should:

1. **Use interval arithmetic** for propagation
2. **Maintain precision** - no arbitrary scaling
3. **Handle special float cases**:
   - NaN handling
   - Infinity bounds
   - Rounding modes for constraint propagation

### Integration Points

The float linear constraints should integrate with:

```rust
// From selen/src/optimization/float_direct.rs (exists)
// This file suggests float optimization is already partially supported
```

### Performance Considerations

- Float linear constraints are more expensive than integer
- May need **relaxation-based propagation** for efficiency
- Consider **lazy evaluation** for large coefficient arrays

---

## 6. Testing Requirements

Once implemented in Selen, verify with:

### Test Suite from Zelen:

We tested against **~900 FlatZinc files** including:
- MiniZinc tutorial examples
- Optimization problems
- Scheduling problems
- Integer constraint satisfaction

### Float-Specific Tests Needed:

1. **loan.fzn** - Financial calculations (currently fails)
2. **Physics simulations** - Kinematics equations
3. **Resource allocation** - Fractional resources
4. **Continuous optimization** - Minimize/maximize float objectives

### Verification Command:

```bash
# After implementing in Selen
cd zelen
cargo test --release
./target/release/zelen /tmp/loan.fzn  # Should show solution, not UNSATISFIABLE
```

---

## 7. Priority Ranking

### P0 - CRITICAL (Blocks float support):
1. ✅ **float_lin_eq** - Most common float constraint
2. ✅ **float_lin_le** - Required for optimization bounds
3. ✅ **float_lin_ne** - Needed for exclusion constraints

### P1 - HIGH (Common use cases):
4. **float_lin_eq_reif** - Conditional float constraints
5. **float_lin_le_reif** - Conditional bounds
6. **array_float_minimum/maximum** - Float aggregations

### P2 - MEDIUM (Less common):
7. **float_eq_reif, float_ne_reif, float_lt_reif** - Other reified comparisons
8. **int_lin_ne** - Can work around, but inefficient

---

## 8. Current Zelen Workaround Status

### What Works (using scaling):
- ❌ **float_eq, float_ne, float_lt, float_le** - Use runtime API (`.eq()`, `.ne()`, `.lt()`, `.le()`)
  - Works because these are simple comparisons
  - No scaling needed
  
- ⚠️ **float_lin_eq, float_lin_le, float_lin_ne** - Scale by 1000x to integers
  - **BROKEN**: Loses precision, causes overflow
  - **INCORRECT**: Not proper float semantics

- ❌ **float_plus, float_minus, float_times, float_div** - Use Selen's `add()`, `sub()`, `mul()`, `div()`
  - These work if variables are float type
  - But composed with scaled linear constraints = broken

### What Fails:
- `/tmp/loan.fzn` - Returns UNSATISFIABLE (wrong)
- Any float optimization problem
- Physics simulations
- Financial calculations

---

## 9. Documentation Updates Needed in Selen

Once implemented, update:

```rust
// selen/src/model/constraints.rs
/// # Float Constraints
///
/// Selen supports float variables with interval-based domains.
/// Float linear constraints maintain precision through interval arithmetic.
///
/// ## Example
/// ```rust
/// let x = model.float(0.0, 10.0);
/// let y = model.float(0.0, 10.0);
/// model.float_lin_eq(&[2.5, 1.5], &[x, y], 10.0);  // 2.5*x + 1.5*y = 10
/// ```
```

---

## 10. API Design Recommendation

### Consistent Naming with Integer Constraints:

```rust
// Integer (existing):
model.int_lin_eq(...)
model.int_lin_le(...)
model.int_lin_eq_reif(...)

// Float (proposed - SAME PATTERN):
model.float_lin_eq(...)
model.float_lin_le(...)
model.float_lin_eq_reif(...)
```

### Type Safety:

```rust
// Coefficients should match variable types
pub fn int_lin_eq(&mut self, coefficients: &[i32], ...);   // i32 coeffs for int vars
pub fn float_lin_eq(&mut self, coefficients: &[f64], ...); // f64 coeffs for float vars
```

### Return Error Handling:

```rust
// Consider returning Result for error cases:
pub fn float_lin_eq(&mut self, ...) -> Result<(), ConstraintError> {
    if coefficients.len() != variables.len() {
        return Err(ConstraintError::DimensionMismatch);
    }
    if coefficients.iter().any(|c| c.is_nan()) {
        return Err(ConstraintError::InvalidCoefficient);
    }
    // ... implementation
}
```

---

## Summary

**Zelen Status**: 
- ✅ 95% integer constraint coverage (~900 tests passing)
- ❌ Float constraints incomplete (blocked by Selen limitations)
- ⚠️ Current float workarounds are incorrect

**Selen Requirements**:
- **3 critical methods** needed: `float_lin_eq`, `float_lin_le`, `float_lin_ne`
- **5 high-priority methods** for full float support
- **1 optimization** for integers: native `int_lin_ne`

**Impact**: 
- Once implemented in Selen, Zelen will immediately support float problems correctly
- No changes needed in Zelen parser or mapper (already implemented)
- Just wire up to native Selen methods instead of scaling workaround

**Estimate**: 
- P0 constraints: ~2-3 days implementation + testing in Selen
- Full float support: ~1 week including reified versions

---

## Contact

When these are implemented in Selen, please provide:
1. Selen version number with float support
2. API documentation for the new methods
3. Any performance considerations or limitations

Zelen will be updated to use native methods immediately.
