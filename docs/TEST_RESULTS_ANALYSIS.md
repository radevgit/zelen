# Zelen Test Results Analysis

**Test Date**: October 5, 2025  
**Total Problems Tested**: 50  
**Success Rate**: 82% (41 passed, 9 failed)

## Summary

After implementing mznlib reorganization and fixing float literal support in array initializers, Zelen successfully solves 82% of the tested MiniZinc problems from the hakank benchmark suite.

---

## Failures Analysis

### ERROR Failures (6 problems) - Parser/Loader Issues

#### 1. **abc_endview.mzn**
- **Error**: `UnsupportedFeature { feature: "Constraint: fzn_global_cardinality", line: 142 }`
- **Category**: Missing constraint implementation
- **Severity**: Medium - global_cardinality is a common constraint
- **Notes**: This is a global constraint that needs to be implemented in both mznlib and Selen

#### 2. **a_card_trick_1.mzn**
- **Error**: `UnsupportedFeature { feature: "Variable type: SetOfInt", line: 110 }`
- **Category**: Missing feature - Set variables
- **Severity**: High - Set variables are part of standard FlatZinc
- **Notes**: Selen doesn't support set variables yet

#### 3. **agprice.mzn**
- **Error**: `MapError { message: "Unsupported expression type: FloatLit(-0.0)" }`
- **Category**: Parser bug - negative zero
- **Severity**: Low - edge case
- **Notes**: Float literal `-0.0` is not handled properly (vs `0.0`)

#### 4. **alldifferent_explain.mzn**
- **Error**: `UnsupportedFeature { feature: "Variable type: SetOfInt", line: 6 }`
- **Category**: Missing feature - Set variables
- **Severity**: High - Set variables are part of standard FlatZinc
- **Notes**: Same as a_card_trick_1.mzn

#### 5. **alldifferent_partition.mzn**
- **Error**: `ParseError { message: "Expected Int or range after 'set of', found LeftBrace", line: 6, column: 12 }`
- **Category**: Parser limitation - Set literals
- **Severity**: Medium - Parser doesn't handle set type declarations properly
- **Notes**: Probably trying to parse `set of {1,2,3}` or similar

#### 6. **all_interval.mzn**
- **Error**: `symbol error: variable 'n' must be defined (did you forget to specify a data file?)`
- **Category**: Test infrastructure issue - Missing data file
- **Severity**: N/A - Not a Zelen issue
- **Notes**: This is a MiniZinc error before Zelen is invoked; problem requires a data file

---

### FAILED Failures (3 problems) - Solver/Runtime Issues

#### 7. **225_divisor.mzn**
- **Issue**: Timeout (30s)
- **Warning**: `Variable 'y' has very large domain [1, 11111111111] with size 11111111111. Using inferred bounds [-100, 101] instead.`
- **Category**: Performance - Large domain
- **Severity**: Low - Problem has extreme domain sizes
- **Notes**: Solver times out, possibly due to complexity or domain inference issues

#### 8. **abbott.mzn**
- **Issue**: Timeout (30s) - No output
- **Category**: Performance or infinite loop
- **Severity**: Medium - Problem hangs without any output
- **Notes**: Solver appears to hang with no progress

#### 9. **alien.mzn**
- **Issue**: Timeout (30s) - No output
- **Category**: Performance or infinite loop
- **Severity**: Medium - Problem hangs without any output
- **Notes**: Solver appears to hang with no progress

---

## Categorized by Root Cause

### Missing Features (3 failures)
1. **Set Variables** (2 problems):
   - a_card_trick_1.mzn
   - alldifferent_explain.mzn
   - *Impact*: High - Set variables are standard FlatZinc feature

2. **Global Constraint: fzn_global_cardinality** (1 problem):
   - abc_endview.mzn
   - *Impact*: Medium - Common constraint

### Parser Issues (2 failures)
1. **Negative Zero Float** (1 problem):
   - agprice.mzn
   - *Impact*: Low - Edge case with `-0.0`

2. **Set Type Declaration Parsing** (1 problem):
   - alldifferent_partition.mzn
   - *Impact*: Medium - Parser limitation

### Solver Performance (2 failures)
1. **Timeouts** (2 problems):
   - abbott.mzn
   - alien.mzn
   - *Impact*: Medium - Need investigation

2. **Large Domain Handling** (1 problem):
   - 225_divisor.mzn
   - *Impact*: Low - Extreme case

### Test Infrastructure (1 failure)
1. **Missing Data File** (1 problem):
   - all_interval.mzn
   - *Impact*: None - Not a Zelen issue

---

## Recent Fixes

### Float Literals in Array Initializers ✅
- **Problem**: Arrays of `var float` couldn't contain float literal constants
- **Example**: `array [1..3] of var float: x = [a, 72.0, b];`
- **Fix**: Added `Expr::FloatLit` handler in mapper.rs to create fixed float variables
- **File**: `/home/ross/devpublic/zelen/src/mapper.rs` (lines 203-207)
- **Impact**: Fixed 1RM.mzn, improved success rate from 80% to 82%

---

## Recommendations

### High Priority
1. **Fix negative zero handling**: Add special case for `-0.0` in float literal parsing
2. **Investigate timeout issues**: Profile abbott.mzn and alien.mzn to find infinite loops or performance bottlenecks

### Medium Priority
3. **Implement fzn_global_cardinality**: Add to both mznlib and Selen
4. **Fix set type parsing**: Improve parser to handle set type declarations

### Low Priority
5. **Set variable support**: Major feature - requires significant work in Selen
6. **Large domain optimization**: Improve domain inference for extreme cases like 225_divisor.mzn

### Infrastructure
7. **Test script**: Update test_problems.sh to detect when .dzn files are required

---

## Success Stories

The following types of problems solve successfully:
- ✅ Integer constraint problems (alldifferent, linear equations, etc.)
- ✅ Float constraint problems (linear equations with floats)
- ✅ Boolean constraint problems (clause, reifications)
- ✅ Array operations (minimum, maximum, element)
- ✅ Global constraints (alldifferent, cumulative, nvalue, lex_lesseq, etc.)
- ✅ Optimization problems (minimize/maximize)
- ✅ UNSAT detection (3_jugs.mzn correctly reports unsatisfiable)
- ✅ Mixed integer/float/bool problems
- ✅ Problems with complex array operations
- ✅ Problems requiring reified constraints

---

## Conclusion

With 82% success rate on a diverse benchmark suite, Zelen demonstrates solid support for core MiniZinc/FlatZinc features. The main limitations are:
- Set variables (fundamental missing feature)
- Some global constraints (global_cardinality)
- Parser edge cases (negative zero, set declarations)
- Performance on specific hard problems

The mznlib reorganization and recent float literal fix show that the architecture is solid and issues can be addressed incrementally.
