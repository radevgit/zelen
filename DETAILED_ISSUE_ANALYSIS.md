# Detailed Issue Analysis: Zelen Model Test Results

**Date:** October 19, 2025  
**Session:** Systematic Model Testing (36 models)

---

## Issue #1: Data File Parsing (.dzn) - HIGH PRIORITY

### Description
When attempting to run models with data files (`test_model.mzn` + `test_data.dzn`), parsing fails with:
```
Parse error: UnexpectedToken { expected: "Colon", found: "Eq" }
```

### Root Cause
The Zelen CLI concatenates `.mzn` and `.dzn` files, but:
- `.mzn` files use **declaration syntax**: `int: n;`
- `.dzn` files use **assignment syntax**: `n = 5;`

When concatenated, the `.dzn` file syntax is invalid in the context of a `.mzn` parser.

### Example Files
- **Model:** `examples/models/test_model.mzn`
  ```minizinc
  int: n;  % Parameter to be set in data file
  array[1..n] of int: costs;  % Parameter array
  var 1..n: choice;
  constraint costs[choice] >= 15;
  solve minimize costs[choice];
  ```

- **Data:** `examples/models/test_data.dzn`
  ```minizinc
  n = 5;
  costs = [20, 10, 25, 15, 30];
  ```

### Expected Behavior
- Parse `.dzn` file as parameter assignments
- Substitute parameter values into `.mzn` model
- Execute merged model

### Current Behavior
- Concatenates files directly
- Parser fails on `.dzn` assignment syntax

### Solution Options

**Option A: Parse .dzn Separately**
1. Parse `.mzn` file normally
2. Parse `.dzn` file with separate parser recognizing `identifier = value;` syntax
3. Merge parameter values into parsed AST
4. Proceed with translation

**Option B: Preprocess .dzn to .mzn**
1. Read `.dzn` file
2. Convert assignments to declarations: `n = 5;` → `int: n = 5;`
3. Prepend to `.mzn` content
4. Parse merged content

**Option C: Document Limitation**
- Mark as unsupported feature for now
- Require users to merge files manually
- Plan for future implementation

### Recommendation
**Option A** (Separate parser) - Most robust and standards-compliant

---

## Issue #2: Float-Only Models Hang - CRITICAL

### Description
Models containing only float variables cause the solver to hang indefinitely (timeout after 10s):

```bash
$ timeout 10 ./target/release/zelen tests_all/models/edge_float_only.mzn
[hangs...]
```

### Test Model
```minizinc
var float: x;
var float: y;
var float: z;

constraint x + y = 10.5;
constraint z >= 0.0;

solve satisfy;
```

### Root Cause Analysis

**Not a parsing issue:** Model parses successfully, error occurs during solving.

**Likely Selen solver issue:** 
- Pure float domains may cause infinite loop in constraint propagation
- Selen might not handle unconstrained float values well
- May need bounds on variables: `var 0.0..100.0: x;`

### Impact
- Blocks execution of valid models
- User experience: appears frozen/unresponsive
- No error message or helpful feedback

### Workaround
Users can add explicit bounds:
```minizinc
var -1000.0..1000.0: x;  % Instead of unbounded
var -1000.0..1000.0: y;
var 0.0..1000.0: z;
```

### Solution Options

**Option A: Add Timeout in CLI**
- Implement solver timeout (already have `-t` flag)
- Return error message instead of hanging
- Improves user experience

**Option B: Detect and Warn**
- Parse model to detect float-only problems
- Print warning: "Float variables require explicit bounds"
- Suggest workaround

**Option C: Add Implicit Bounds**
- Detect float-only variables
- Automatically add bounds: `-1e6..1e6`
- May hide underlying issue

**Option D: Investigate Selen**
- Profile Selen's float constraint handler
- May need to file issue upstream
- Check Selen version and solver algorithms

### Recommendation
**Option A + Option B** (Timeout + Warning) - Immediate mitigation while investigating

---

## Issue #3: Enum Value References Undefined - MEDIUM PRIORITY

### Description
Using enum values in constraints fails with "Undefined variable" error:

```bash
$ ./target/release/zelen tests_all/models/test_enum_basic.mzn
Error: "Undefined variable or parameter: 'Red'"
```

### Test Model
```minizinc
enum Color = {Red, Green, Blue};
var Color: my_color;
constraint my_color != Red;  % ← Error: Red not recognized
solve satisfy;
```

### Root Cause
Enum values (`Red`, `Green`, `Blue`) are not being registered as constants in the translator context.

### Working vs Broken

**Broken:**
```minizinc
constraint my_color != Red;  % Error: undefined
```

**Workaround (Works):**
```minizinc
% Use only global constraints that don't reference enum values
constraint alldifferent([my_color, ...]);
```

### Implementation Issue
In `src/translator.rs`, enum handling needs to:
1. Create constant symbols for each enum value
2. Register them in translator context
3. Allow them in constraint expressions

### Current Code Flow
- Enum declaration parsed ✓
- Enum used as type annotation ✓
- Enum values as constants ✗ (BROKEN)

### Solution
Add to `TranslatorContext` initialization (where enum is declared):
```rust
// Register enum values as constants
for (idx, value_name) in enum_values.iter().enumerate() {
    int_params.insert(value_name.clone(), idx as i32);
}
```

Then use in `evaluate_expr()`:
```rust
Expr::Identifier(name) => {
    if let Some(value) = int_params.get(name) {
        return Ok(Expression::Constant(*value));
    }
    // ... rest of resolution logic
}
```

### Recommendation
Fix enum value registration to allow direct references in constraints

---

## Issue #4: Type Mismatch Not Detected - LOW PRIORITY

### Description
Models with type mismatches are accepted without error:

```bash
$ ./target/release/zelen tests_all/models/error_type_mismatch.mzn
[succeeds with no error]
```

### Test Model
```minizinc
var int: x;
var bool: b;
constraint x = b;  % Type mismatch: int ≠ bool
solve satisfy;
```

### Status
**By-design limitation** - Zelen intentionally permissive to allow solver flexibility

### Rationale
- Different solvers have different coercion rules
- Strict type checking would require full type inference pass
- Would add compilation overhead
- Current approach: "let solver handle it"

### Documentation
- Already documented in `ERROR_HANDLING.md`
- Listed as known limitation
- Not a bug, design choice

### Recommendation
No action needed - properly documented

---

## Summary Table

| Issue | Priority | Category | Status | Action |
|-------|----------|----------|--------|--------|
| #1: .dzn parsing | HIGH | Feature | New | Implement .dzn parser |
| #2: Float hang | CRITICAL | Performance | New | Add timeout + investigate |
| #3: Enum values | MEDIUM | Bug | New | Register enum constants |
| #4: Type mismatch | LOW | Design | Expected | Document (done) |

---

## Test Results Impact

```
Current: 32/36 passing (89%)

After Fixes:
- Fix #1 (.dzn): +1 → 33/36 (92%)
- Fix #2 (float): +1 → 34/36 (94%)
- Fix #3 (enum): +1 → 35/36 (97%)
- Fix #4: N/A (by-design)

Target: 35/36 (97%)
```

---

## Recommended Implementation Order

1. **First:** Fix enum value registration (#3)
   - Quickest fix (5-10 lines of code)
   - Affects: 1 test
   - Difficulty: Low

2. **Second:** Implement .dzn parsing (#1)
   - Medium complexity (50-100 lines)
   - Affects: 1+ tests
   - Difficulty: Medium

3. **Third:** Investigate float hang (#2)
   - Complexity depends on root cause
   - May require upstream fix in Selen
   - Can implement timeout as interim solution
   - Difficulty: High

---

## Related Files

- `src/translator.rs` - Where enum values need to be registered
- `src/parser.rs` - Where .dzn parsing would be added
- `src/main.rs` - CLI timeout handling
- `examples/models/test_model*.mzn` - Test cases
- `tests_all/models/test_enum_basic.mzn` - Enum value test
- `tests_all/models/edge_float_only.mzn` - Float timeout test

---

## Testing Commands

```bash
# Test all models
./run_model_tests.sh

# Test individual issues:
./target/release/zelen tests_all/models/test_enum_basic.mzn      # Issue #3
timeout 3 ./target/release/zelen tests_all/models/edge_float_only.mzn  # Issue #2
./target/release/zelen examples/models/test_model.mzn examples/models/test_data.dzn  # Issue #1

# Manual .dzn merge (workaround)
cat examples/models/test_model.mzn examples/models/test_data.dzn > /tmp/merged.mzn
./target/release/zelen /tmp/merged.mzn
```
