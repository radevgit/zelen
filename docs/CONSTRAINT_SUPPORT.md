# Constraint Support Analysis for Zelen/Selen

## Currently Mapped in Zelen (from src/mapper.rs)

### Comparison Constraints
- int_eq, int_ne, int_lt, int_le, int_gt, int_ge
- float_eq, float_ne, float_lt, float_le (implied: float_gt, float_ge)
- bool_eq, bool_le

### Linear Constraints
- int_lin_eq, int_lin_le, int_lin_ne
- float_lin_eq, float_lin_le, float_lin_ne

### Reified Constraints
- int_eq_reif, int_ne_reif, int_lt_reif, int_le_reif, int_gt_reif, int_ge_reif
- int_lin_eq_reif, int_lin_le_reif, int_lin_ne_reif
- float_eq_reif, float_ne_reif, float_lt_reif, float_le_reif, float_gt_reif, float_ge_reif
- float_lin_eq_reif, float_lin_le_reif, float_lin_ne_reif
- bool_eq_reif, bool_le_reif
- set_in_reif

### Arithmetic Operations
- int_abs, int_plus, int_minus, int_times, int_div, int_mod
- int_max, int_min
- float_abs, float_plus, float_minus, float_times, float_div
- float_max, float_min

### Boolean Constraints
- bool_not, bool_xor
- bool_clause
- array_bool_and, array_bool_or
- bool2int

### Array Constraints
- array_int_minimum, array_int_maximum (mapped as minimum_int, maximum_int)
- array_int_element, array_var_int_element
- array_bool_element, array_var_bool_element

### Global Constraints
- all_different (fzn_all_different_int)
- table_int, table_bool
- sort
- nvalue
- lex_less_int, lex_lesseq_int
- global_cardinality, global_cardinality_low_up_closed
- cumulative, var_fzn_cumulative

### Counting Constraints
- count_eq (aliased as count)

### Set Constraints
- set_in, set_in_reif

### Type Conversions
- int2float, float2int

## Confirmed Selen API Methods (from ../selen/src/constraints/api/)

### Arithmetic (api/arithmetic.rs)
- add(x, y) -> VarId
- sub(x, y) -> VarId
- mul(x, y) -> VarId
- div(x, y) -> VarId
- modulo(x, y) -> VarId
- abs(x) -> VarId
- min(&vars) -> VarId
- max(&vars) -> VarId
- sum(&vars) -> VarId
- sum_iter(iterator) -> VarId

### Boolean (api/boolean.rs)
- bool_and(&operands) -> VarId
- bool_or(&operands) -> VarId
- bool_not(operand) -> VarId
- bool_clause(&pos, &neg)

### Linear (api/linear.rs)
**Integer:**
- int_lin_eq(&coeffs, &vars, constant)
- int_lin_le(&coeffs, &vars, constant)
- int_lin_ne(&coeffs, &vars, constant)
- int_lin_eq_reif(&coeffs, &vars, constant, reif_var)
- int_lin_le_reif(&coeffs, &vars, constant, reif_var)
- int_lin_ne_reif(&coeffs, &vars, constant, reif_var)

**Float:**
- float_lin_eq(&coeffs, &vars, constant)
- float_lin_le(&coeffs, &vars, constant)
- float_lin_ne(&coeffs, &vars, constant)
- float_lin_eq_reif(&coeffs, &vars, constant, reif_var)
- float_lin_le_reif(&coeffs, &vars, constant, reif_var)
- float_lin_ne_reif(&coeffs, &vars, constant, reif_var)

**Boolean:**
- bool_lin_eq(&coeffs, &vars, constant)
- bool_lin_le(&coeffs, &vars, constant)
- bool_lin_ne(&coeffs, &vars, constant)
- bool_lin_eq_reif(&coeffs, &vars, constant, reif_var)
- bool_lin_le_reif(&coeffs, &vars, constant, reif_var)
- bool_lin_ne_reif(&coeffs, &vars, constant, reif_var)

### Reified (api/reified.rs)
**Integer:**
- int_eq_reif(x, y, b)
- int_ne_reif(x, y, b)
- int_lt_reif(x, y, b)
- int_le_reif(x, y, b)
- int_gt_reif(x, y, b)
- int_ge_reif(x, y, b)

**Float:**
- float_eq_reif(x, y, b)
- float_ne_reif(x, y, b)
- float_lt_reif(x, y, b)
- float_le_reif(x, y, b)
- float_gt_reif(x, y, b)
- float_ge_reif(x, y, b)

### Conversion (api/conversion.rs)
- int2float(int_var, float_var)
- float2int_floor(float_var, int_var)
- float2int_ceil(float_var, int_var)
- float2int_round(float_var, int_var)

### Array (api/array.rs)
- array_float_minimum(&array) -> VarId
- array_float_maximum(&array) -> VarId
- array_float_element(index, &array, result)

### Global Constraints (from propagators)
**Confirmed from usage:**
- alldiff(&vars)  ✓ USED
- count(&vars, target_value, count_var)  ✓ USED

**Found in source (constraints/props/mod.rs):**
- at_least_constraint(vars, target_value, count)
- at_most_constraint (implied)
- table_constraint(vars, tuples)
- count_constraint (used via count() wrapper)

## Selen Methods Likely Available (need confirmation from ../selen)
These are commonly available in CP solvers and may be in Selen:
- at_least(n, &vars, value)
- at_most(n, &vars, value)
- among(n, &vars, &values)
- circuit(&vars)
- inverse(&x, &y)
- element constraints (we have them)
- bool_lin_* (boolean linear constraints)
- More global constraints...

## MiniZinc Standard Library fzn_* Predicates (sample)
Key predicates from /snap/minizinc/1157/share/minizinc/std/:
- fzn_all_different_int
- fzn_all_equal_int
- fzn_among
- fzn_at_least_int, fzn_at_most_int
- fzn_circuit
- fzn_count_eq, fzn_count_eq_reif
- fzn_cumulative
- fzn_global_cardinality*
- fzn_if_then_else_int/float/bool
- fzn_inverse
- fzn_lex_less_int, fzn_lex_lesseq_int
- fzn_nvalue
- fzn_sort
- fzn_table_int, fzn_table_bool
- Many reified versions (_reif)

## Action Items

### 1. Check Selen Source
Look at ../selen/src/constraints/api/ to find all available methods:
- Native global constraints (alldiff, circuit, inverse, etc.)
- at_least, at_most, among
- bool_lin_* family
- Other propagators

### 2. Create mznlib Files
For each Selen method, create corresponding fzn_*.mzn in mznlib/:
- Declare the predicate (no body needed - native)
- This tells MiniZinc to use our implementation directly

### 3. Implement Missing Mappers
For methods in Selen not yet in mapper.rs:
- Add dispatcher entry in map_constraint()
- Create mapper function
- Test with MiniZinc problems

### 4. Priority Constraints to Add
Based on common usage:
- bool_lin_eq/ne/le (boolean sums)
- at_least_int, at_most_int (if native)
- among (if native)
- circuit (if native)
- inverse (if native)
- all_equal_int (can decompose but native is better)

## Current mznlib Files
```
fzn_all_different_int.mzn
fzn_array_int_element.mzn
fzn_bool_clause.mzn
fzn_count_eq.mzn
fzn_cumulative.mzn
fzn_float_eq_reif.mzn
fzn_float_lin_eq.mzn
fzn_float_lin_eq_reif.mzn
fzn_global_cardinality.mzn
fzn_int_eq_reif.mzn
fzn_int_lin_eq.mzn
fzn_int_lin_eq_reif.mzn
fzn_lex_less_int.mzn
fzn_minimum_int.mzn
fzn_nvalue.mzn
fzn_set_in.mzn
fzn_sort.mzn
fzn_table_int.mzn
redefinitions.mzn
```

## Next Steps
1. Access ../selen source to catalog all available constraint methods
2. Create comprehensive mznlib/ with all supported predicates
3. Add mappers for any Selen methods we haven't exposed yet
4. Test with hakank problems to ensure native implementations are used
