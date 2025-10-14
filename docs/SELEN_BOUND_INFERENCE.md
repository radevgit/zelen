# Selen Bound Inference Implementation

**Status**: ✅ Implemented in Selen (October 2025)  
**Selen Version**: 0.9.1+  
**Purpose**: Enable Zelen to pass unbounded variables to Selen

---

## Overview

Selen now has automatic bound inference for unbounded integer and float variables. This allows Zelen to pass variables with infinite bounds directly to Selen without workarounds.

### What Was the Problem?

FlatZinc allows unbounded variable declarations like:
- `var int: x;` (unbounded integer)
- `var float: y;` (unbounded float)

Previously:
- Zelen had to use workarounds (e.g., `[-10000, 10000]` hardcoded bounds)
- Selen would reject `i32::MIN/MAX` or `f64::INFINITY` bounds
- This caused validation errors and artificial constraints

### What's New?

Selen now automatically infers reasonable bounds by:
1. **Context Analysis**: Scanning existing variables of the same type
2. **Smart Expansion**: Expanding the context by a configurable factor (default 1000x)
3. **Safe Fallback**: Using ±10,000 when no context exists
4. **Domain Protection**: Respecting the 1M element limit for integer domains

---

## Algorithm Details

### Integer Variable Inference

When Selen sees `i32::MIN` or `i32::MAX`:

1. **Detect Unbounded**: Check if `min == i32::MIN` or `max == i32::MAX`
2. **Scan Context**: Find global_min and global_max across all existing integer variables
3. **Calculate Span**: `span = global_max - global_min`
4. **Expand**: `[global_min - factor*span, global_max + factor*span]`
5. **Clamp to i32 Range**: `[i32::MIN+1, i32::MAX-1]`
6. **Domain Size Check**: If domain > 1,000,000 elements:
   - Calculate center: `(min + max) / 2`
   - Clamp to: `[center - 500,000, center + 499,999]` (exactly 1M elements) (constant is defined about that)
7. **Fallback**: If no context, use `[-10,000, 10,000]`

### Float Variable Inference

When Selen sees `f64::INFINITY`, `f64::NEG_INFINITY`, or `NaN`:

1. **Detect Unbounded**: Check if bounds are infinite or NaN
2. **Scan Context**: Find global_min and global_max across all existing float variables
3. **Calculate Span**: `span = global_max - global_min`
4. **Expand**: `[global_min - factor*span, global_max + factor*span]`
5. **Clamp to Safe Range**: `[-1e308, 1e308]`
6. **Fallback**: If no context, use `[-10,000.0, 10,000.0]`

### Key Design Choices

- **Type Isolation**: Integer and float inference are completely separate
- **1000x Default Factor**: Logarithmic middle ground (between 10x conservative and 100,000x aggressive)
- **Configurable**: Factor can be adjusted per-solver via `SolverConfig`
- **Pre-Validation**: Inference happens before validation, preventing errors

---

## Configuration

### Default Usage (1000x expansion)

```rust
use selen::prelude::*;

let mut model = Model::default();
// Unbounded variables automatically inferred with 1000x factor
let x = model.int(i32::MIN, i32::MAX); // Will infer reasonable bounds
let y = model.float(f64::NEG_INFINITY, f64::INFINITY); // Will infer reasonable bounds
```

### Custom Expansion Factor

```rust
use selen::prelude::*;

let config = SolverConfig::default()
    .with_unbounded_inference_factor(300); // More conservative

let mut model = Model::new(config);
let x = model.int(i32::MIN, i32::MAX); // Uses 300x expansion
```

### Factor Guidelines

- **100-500**: Conservative, tight bounds (good for small search spaces)
- **1000** (default): Balanced, works well for most problems
- **5000-10000**: Aggressive, large search spaces (optimization problems)

---

## Integration with Zelen

### Before (Workaround)

```rust
// mapper.rs (old code)
fn map_variable(&mut self, decl: &VarDecl) {
    match decl.domain {
        Domain::Unbounded => {
            // Workaround: hardcoded bounds
            self.model.int(-10000, 10000)
        }
        // ...
    }
}
```

### After (Direct Mapping)

```rust
// mapper.rs (new code)
fn map_variable(&mut self, decl: &VarDecl) {
    match decl.domain {
        Domain::Unbounded => {
            // Let Selen infer bounds automatically
            match decl.var_type {
                VarType::Int => self.model.int(i32::MIN, i32::MAX),
                VarType::Float => self.model.float(f64::NEG_INFINITY, f64::INFINITY),
                // ...
            }
        }
        // ...
    }
}
```

### Benefits for Zelen

1. **No More Hardcoded Bounds**: Remove the `-10000..10000` workaround
2. **Context-Aware**: Bounds adapt to each problem automatically
3. **Correct Semantics**: Truly unbounded variables, inferred from context
4. **Better Solutions**: Wider search space when appropriate

---

## Implementation Files (in Selen)

### Core Implementation
- **src/model/factory_internal.rs**: `infer_bounds()` method (~140 lines)
  - Main inference logic for both integers and floats
  - Domain size limit handling
  - Type isolation

### Configuration
- **src/utils/config.rs**: `SolverConfig::unbounded_inference_factor`
  - Configuration field and builder method
  - Default value: 1000

### Tests
- **tests/test_unbounded_variables.rs**: 14 comprehensive tests
  - Fallback inference (no context)
  - Context-based inference (small and large spans)
  - Domain size limit enforcement
  - Boundary conditions (i32::MIN/MAX, infinity, NaN)
  - Type isolation
  - Integration with solving
  - Custom configuration factors

### Documentation
- **docs/development/BOUND_INFERENCE_DESIGN.md**: Design rationale
- **docs/development/UNBOUNDED_INFERENCE_IMPLEMENTATION.md**: Usage guide

---

## Examples

### Example 1: Simple Unbounded Integer

```rust
let mut model = Model::default();
let x = model.int(i32::MIN, i32::MAX); // Infers [-10000, 10000] (no context)
model.all_different(&[x]);
```

### Example 2: Context-Based Inference

```rust
let mut model = Model::default();

// Create context with bounded variables
let a = model.int(0, 100);
let b = model.int(50, 150);

// Unbounded variable infers from context
// Context: [0, 150], span = 150
// Inferred: [0 - 1000*150, 150 + 1000*150] = [-150000, 150150]
let x = model.int(i32::MIN, i32::MAX);

model.all_different(&[a, b, x]);
```

### Example 3: Float Inference

```rust
let mut model = Model::default();

// Bounded floats establish context
let a = model.float(0.0, 10.0);
let b = model.float(5.0, 15.0);

// Unbounded float infers from context
// Context: [0.0, 15.0], span = 15.0
// Inferred: [0.0 - 1000*15.0, 15.0 + 1000*15.0] = [-15000.0, 15015.0]
let x = model.float(f64::NEG_INFINITY, f64::INFINITY);
```

### Example 4: Type Isolation

```rust
let mut model = Model::default();

// Float context
let f1 = model.float(0.0, 100.0);
let f2 = model.float(50.0, 150.0);

// Integer ignores float context, uses fallback
let x = model.int(i32::MIN, i32::MAX); // Infers [-10000, 10000]

// Float uses float context
let y = model.float(f64::NEG_INFINITY, f64::INFINITY); // Infers from f1, f2
```

---

## Migration Checklist for Zelen

- [ ] Update `mapper.rs` to remove hardcoded bounds workaround
- [ ] Change unbounded integer mapping to: `model.int(i32::MIN, i32::MAX)`
- [ ] Change unbounded float mapping to: `model.float(f64::NEG_INFINITY, f64::INFINITY)`
- [ ] Test with various FlatZinc models containing unbounded variables
- [ ] Consider exposing `unbounded_inference_factor` config to Zelen users
- [ ] Update Zelen documentation to mention automatic bound inference

---

## Testing

All bound inference functionality is thoroughly tested in Selen:

```bash
cd selen
cargo test test_unbounded
```

**Test Results**: 14 tests, all passing ✅

Test coverage includes:
- Fallback inference when no context exists
- Context-based inference with various spans
- Domain size limit enforcement (1M elements for integers)
- Boundary conditions (i32::MIN/MAX, f64::INFINITY, NaN)
- Type isolation (int/float contexts don't mix)
- Integration with actual constraint solving
- Custom configuration factors

---

## Performance Notes

- **Zero Overhead**: Inference only runs for unbounded variables
- **O(n) Scan**: Context scan is linear in number of existing variables (typically fast)
- **One-Time Cost**: Inference happens once at variable creation
- **No Runtime Impact**: Inferred bounds are fixed, no overhead during solving

---

## Limitations

1. **Requires Context**: Best results when other bounded variables exist
2. **Domain Size**: Integer variables still limited to 1M elements total
3. **Static Bounds**: Inferred bounds don't adapt if more variables added later
4. **No Cross-Type**: Integer context doesn't help float inference (by design)

---

## Future Enhancements (Potential)

- Per-variable expansion factors
- Constraint-aware inference (analyze constraints to tighten bounds)
- Automatic factor tuning based on problem characteristics
- Statistics/warnings when inference triggers
- Iterative refinement as more variables are added

---

## Questions?

For issues or questions about Selen's bound inference:
- Check Selen's documentation: `selen/docs/development/`
- Review test examples: `selen/tests/test_unbounded_variables.rs`
- See implementation: `selen/src/model/factory_internal.rs`

---

**Summary**: Selen's automatic bound inference eliminates the need for Zelen to use hardcoded bounds for unbounded variables. Simply pass `i32::MIN/MAX` or `f64::INFINITY` and let Selen infer reasonable bounds from context.
