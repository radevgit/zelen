# Zelen Implementation TODO

## Current Status Analysis

### What Selen Supports (from source code inspection)
**Variable Types:**
- `int(min, max)` - Integer variables with bounds
- `float(min, max)` - Float variables with bounds  
- `bool()` - Boolean variables
- `intset(values)` - Integer set variables

**Constraints (Model methods):**
- Arithmetic: `add`, `sub`, `mul`, `div`, `modulo`, `abs`
- Aggregation: `min`, `max`, `sum`, `sum_iter`
- Boolean: `bool_and`, `bool_or`, `bool_not`
- Reified comparisons: `int_eq_reif`, `int_ne_reif`, `int_lt_reif`, `int_le_reif`, `int_gt_reif`, `int_ge_reif`
- Linear: `int_lin_eq`, `int_lin_le`
- Boolean: `bool_clause`

### What Zelen Currently Implements

**Parser (src/parser.rs):**
- ✅ `Type::Int` - basic int type
- ✅ `Type::Bool` - basic bool type
- ✅ `Type::Float` - basic float type
- ✅ `Type::IntRange(min, max)` - int ranges
- ✅ `Type::IntSet(values)` - int sets
- ✅ `Type::FloatRange(min, max)` - **DEFINED IN AST BUT NOT PARSED!**
- ✅ `Type::SetOfInt` - set of int
- ✅ `Type::Array` - arrays

**Mapper (src/mapper/constraints/):**
Integer constraints:
- ✅ `int_eq`, `int_ne`, `int_lt`, `int_le`, `int_gt`, `int_ge`
- ✅ `int_lin_eq`, `int_lin_le`, `int_lin_ne`
- ✅ `int_lin_eq_reif`, `int_lin_le_reif`
- ✅ `int_eq_reif`, `int_ne_reif`, `int_lt_reif`, `int_le_reif`, `int_gt_reif`, `int_ge_reif`
- ✅ `int_abs`, `int_plus`, `int_minus`, `int_times`, `int_div`, `int_mod`
- ✅ `int_max`, `int_min`

Boolean constraints:
- ✅ `bool_eq`, `bool_le`, `bool_not`, `bool_xor`
- ✅ `bool_le_reif`, `bool_eq_reif`
- ✅ `bool_clause`

Array constraints:
- ✅ `array_int_minimum`, `array_int_maximum`
- ✅ `array_bool_and`, `array_bool_or`
- ✅ `array_var_int_element`, `array_int_element`
- ✅ `array_var_bool_element`, `array_bool_element`

Global constraints:
- ✅ `all_different`
- ✅ `sort`
- ✅ `table_int`, `table_bool`
- ✅ `lex_less`, `lex_lesseq`
- ✅ `nvalue`
- ✅ `cumulative`
- ✅ `count_eq`
- ✅ `global_cardinality`, `global_cardinality_low_up_closed`

Other:
- ✅ `bool2int`
- ✅ `set_in`, `set_in_reif`

## Missing Implementations

### 1. Parser - Float Range Support
**File:** `src/parser.rs`
**Issue:** Parser cannot handle `var 0.0..10.0: x` syntax
**Location:** `parse_type()` function around line 200

**Required Changes:**
```rust
TokenType::FloatLiteral(min) => {
    let min_val = *min;
    self.advance();
    self.expect(TokenType::DoubleDot)?;
    if let TokenType::FloatLiteral(max) = self.peek() {
        let max_val = *max;
        self.advance();
        Type::FloatRange(min_val, max_val)
    } else {
        return Err(FlatZincError::ParseError {
            message: "Expected float for range upper bound".to_string(),
            line: loc.line,
            column: loc.column,
        });
    }
}
```

### 2. Mapper - Float Constraints
**File:** `src/mapper/constraints/float.rs` (NEW FILE NEEDED)

According to FlatZinc spec, standard float constraints that should be supported:
- `float_eq(var float: a, var float: b)`
- `float_ne(var float: a, var float: b)`
- `float_lt(var float: a, var float: b)`
- `float_le(var float: a, var float: b)`
- `float_lin_eq(array[int] of float: coeffs, array[int] of var float: vars, float: constant)`
- `float_lin_le(array[int] of float: coeffs, array[int] of var float: vars, float: constant)`
- `float_lin_ne(array[int] of float: coeffs, array[int] of var float: vars, float: constant)`
- `float_plus(var float: a, var float: b, var float: c)` - c = a + b
- `float_minus(var float: a, var float: b, var float: c)` - c = a - b
- `float_times(var float: a, var float: b, var float: c)` - c = a * b
- `float_div(var float: a, var float: b, var float: c)` - c = a / b
- `float_abs(var float: a, var float: b)` - b = |a|
- `float_max(var float: a, var float: b, var float: c)` - c = max(a, b)
- `float_min(var float: a, var float: b, var float: c)` - c = min(a, b)

Reified versions:
- `float_eq_reif`, `float_ne_reif`, `float_lt_reif`, `float_le_reif`
- `float_lin_eq_reif`, `float_lin_le_reif`

### 3. Mapper - Float Variable Creation
**File:** `src/mapper.rs`
**Location:** `map_var_decl()` function around line 90-100

**Current Code:**
```rust
Type::Float => self.model.float(f64::NEG_INFINITY, f64::INFINITY),
```

**Issue:** This is correct! But need to also handle FloatRange:
```rust
Type::FloatRange(min, max) => self.model.float(min, max),
```

### 4. Output - Float Formatting
**File:** `src/output.rs`
**Issue:** Float values need proper formatting in solutions
**Check:** Line ~100-130 in `format_solution()`

Need to ensure float values are formatted correctly (not as integers).

### 5. Missing Standard FlatZinc Constraints

According to the FlatZinc spec (Section 4.3.4.1), these are additional standard constraints we should check:

**Set constraints:**
- `set_eq`, `set_ne`, `set_subset`, `set_card`
- `set_union`, `set_intersect`, `set_diff`, `set_symdiff`

**Array aggregations:**
- `array_float_minimum`, `array_float_maximum`
- We have `array_int_minimum` and `array_int_maximum` ✅

**Reified boolean:**
- `bool_clause_reif` - we have `bool_clause` ✅
- `bool_xor_reif` - we have `bool_xor` but not reified version

**String operations:**
- String constraints are typically handled during compilation, not in FlatZinc

## Implementation Plan

### Phase 1: Float Support (HIGH PRIORITY - needed for /tmp/loan.fzn)
1. ✅ Add FloatRange parsing in `parse_type()`
2. ✅ Add FloatRange handling in `map_var_decl()`
3. ✅ Create `src/mapper/constraints/float.rs` with all float constraints
4. ✅ Add float constraint mappings to `map_constraint()` match statement
5. ✅ Test with `/tmp/loan.fzn`

### Phase 2: Test Against MiniZinc Examples
1. Find/create directory `/tmp/zinc/` with FlatZinc examples
2. Convert various MiniZinc examples to FlatZinc using different solvers
3. Test each example with zelen
4. Document which constraints are used and which fail
5. Implement missing constraints iteratively

### Phase 3: Advanced Constraints
1. Implement remaining set constraints if Selen supports them
2. Add missing reified versions
3. Optimize performance for large problems

## Testing Strategy

1. **Unit tests** for each new constraint type
2. **Integration tests** with real FlatZinc files from MiniZinc examples
3. **Comparison testing** - compare zelen output with Gecode/Chuffed on same problems
4. **Performance benchmarks** - ensure constraints perform reasonably

## Notes

- Selen appears to be primarily an **integer CSP solver** with float support added
- Float constraints in Selen likely work by **discretization** (converting to integers internally)
- Some FlatZinc constraints may not map directly to Selen's API and need decomposition
- **Priority**: Focus on most commonly used constraints first (int, bool, basic float)
