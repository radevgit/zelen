# TODO: Complete Selen Integration

## 🎉 Latest Test Results (October 5, 2025)
**Successfully solved 48+ out of 60 test problems (80%+ success rate)**

Test batches:
- Batch 1 (problems 1-20): 13/20 solved ✓
- Batch 2 (problems 21-40): 17/20 solved ✓ 
- Batch 3 (problems 41-60): 18/20 solved ✓
- Final verification (problems 1-15): 13/15 solved ✓ (including previously failing problems!)

Key achievements:
- ✅ **age_changing.mzn** now solves correctly (h=53, m=48) - was returning UNSATISFIABLE before!
- ✅ float_lin_*_reif constraints fully working
- ✅ bool_lin_* constraints fully working (6 new mappers)
- ✅ float array operations working (3 new mappers)
- ✅ **int array operations added to Selen API** (array_int_minimum/maximum/element)
- ✅ mznlib integration successful (21 native predicates)
- ✅ **Restructured to share/minizinc/zelen/** for proper MiniZinc integration

Total test suite: 1180 MiniZinc problems in zinc/hakank/

## ✅ Already Implemented and Declared in mznlib

### Linear Constraints
- ✅ int_lin_eq, int_lin_le, int_lin_ne + _reif versions
- ✅ float_lin_eq, float_lin_le, float_lin_ne + _reif versions
- ✅ bool_lin_eq, bool_lin_le, bool_lin_ne + _reif versions ✨ NEWLY IMPLEMENTED!

### Comparison Reified
- ✅ int_eq_reif, int_ne_reif, int_lt_reif, int_le_reif, int_gt_reif, int_ge_reif
- ✅ float_eq_reif, float_ne_reif, float_lt_reif, float_le_reif, float_gt_reif, float_ge_reif
- ✅ bool_eq_reif, bool_le_reif

### Array Operations
- ✅ array_int_element, array_var_int_element
- ✅ array_bool_element, array_var_bool_element
- ✅ array_int_minimum, array_int_maximum
- ✅ array_float_element, array_float_minimum, array_float_maximum ✨ NEWLY IMPLEMENTED!

### Boolean
- ✅ bool_clause, array_bool_and, array_bool_or

### Global
- ✅ all_different (alldiff)
- ❌ count_eq - REMOVED from mznlib (Selen only supports constant values, std requires var int)
- ❌ table_int, table_bool - REMOVED from mznlib (2D arrays not allowed in FlatZinc)
- ✅ sort
- ✅ nvalue
- ✅ lex_less_int, lex_lesseq_int
- ✅ global_cardinality, global_cardinality_low_up_closed
- ✅ cumulative

### Set
- ✅ set_in, set_in_reif

### Conversions
- ✅ int2float, float2int

## 🔧 Known Limitations

### Signature Incompatibilities
- ❌ **count_eq**: Selen requires constant `target_value`, but FlatZinc std allows `var int`
  - Solution: Let MiniZinc decompose count constraints
- ❌ **table_int/table_bool**: Use 2D arrays in signature, but FlatZinc only allows 1D arrays
  - Solution: Let MiniZinc decompose table constraints

## ✅ Implementation Complete!

All Selen API methods that can be exposed to MiniZinc have been implemented:
- ✅ Bool linear constraints (6 mappers)
- ✅ Float array operations (3 mappers) 
- ✅ Int array operations (3 Selen API methods added)
- ✅ mznlib reorganized into 19 well-organized files

## 📊 Final Statistics

- **Selen API methods**: 54+ methods cataloged
- **Zelen mappers**: 80+ constraint mapping functions
- **mznlib files**: 19 organized predicate declaration files
- **Test success rate**: 80%+ (61/75 problems solved)
- **Build time**: 28.5s (release build)

## 📁 mznlib Organization (19 files)

### Array Operations (3 files)
- **fzn_array_int.mzn** - minimum, maximum, element
- **fzn_array_float.mzn** - minimum, maximum, element
- **fzn_array_bool.mzn** - element, and, or

### Type Operations (3 files)
- **fzn_int.mzn** - 6 reified comparisons (eq, ne, lt, le, gt, ge)
- **fzn_float.mzn** - 6 reified comparisons
- **fzn_bool.mzn** - reified comparisons + clause

### Linear Constraints (5 files)
- **fzn_int_lin_eq.mzn** + **fzn_int_lin_eq_reif.mzn**
- **fzn_float_lin_eq.mzn** + **fzn_float_lin_eq_reif.mzn**
- **fzn_bool_lin_eq.mzn** (includes reified)

### Global Constraints (6 files)
- fzn_all_different_int.mzn
- fzn_cumulative.mzn
- fzn_global_cardinality.mzn
- fzn_lex_less_int.mzn
- fzn_nvalue.mzn
- fzn_sort.mzn

### Set + Wrappers (2 files)
- fzn_set_in.mzn
- redefinitions.mzn

## 🔍 Future Enhancements

### If Selen Adds More Propagators
- circuit (Hamiltonian circuits)
- inverse (inverse permutations)
- diffn (non-overlapping rectangles)
- regular (regular expressions)
- And more advanced global constraints...

### count_eq and table_int
These require changes in Selen:
- **count_eq**: Would need var int support (currently constant only)
- **table_int**: Would need 1D array interface (or let MiniZinc continue decomposing)
