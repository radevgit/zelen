# Phase 3: Variable Multi-Dimensional Array Indexing

## Overview
Extend multi-dimensional array support to handle variable indices, enabling dynamic access patterns.

## Example
```minizinc
int: n = 3;
int: m = 3;
array[1..n, 1..m] of var 1..9: grid;
var 1..n: i;
var 1..m: j;
constraint grid[i,j] != 5;  // Access with variable indices
```

## Implementation Strategy

### 1. Index Flattening Computation
Transform multi-dimensional variable indices to a single 1D index:
- Input: variable indices [i, j, k] with dimensions [d1, d2, d3]
- Convert to 0-based: [i-1, j-1, k-1]
- Compute: `flat = (i-1)*d2*d3 + (j-1)*d3 + (k-1)`
- Use Selen operations: `mul`, `add`, `sub`

### 2. Auxiliary Variable Approach
```
flat_index = new_int(0, d1*d2*d3-1)
constraint flat_index = (i-1)*d2*d3 + (j-1)*d3 + (k-1)
result = element(array, flat_index)
```

### 3. Code Changes Needed
**File: `src/translator.rs`**
- Modify ArrayAccess handler's variable index branch
- When `all_const == false` and `indices.len() > 1`:
  1. Convert each index variable from 1-based to 0-based
  2. Create auxiliary variable for flattened index
  3. Build constraints for flattening computation
  4. Use element constraint with flattened index

## Test Status
- ✅ 6 Phase 1 tests passing (constant indices)
- 🔄 2 Phase 3 tests pending (variable indices)

## Files Involved
- `tests_all/test_variable_indexing.rs` - Test cases
- `src/translator.rs` - Main implementation location
- `tests/main_tests.rs` - Test orchestration

## Next Steps
1. Implement the index flattening computation in translate_expr
2. Handle multi-dimensional constraints with Selen operations
3. Pass tests: `test_2d_grid_variable_indexing`, `test_3d_cube_variable_indexing`
4. Extend to higher-dimensional arrays
