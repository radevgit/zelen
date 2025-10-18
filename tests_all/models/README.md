# Test Models

This directory contains MiniZinc test models for validating the Zelen translator.

## Models by Feature

### 2D/3D Arrays

- **test_2d_grid.mzn** - Basic 2D grid constraint problem with variable arrays
- **test_3d_cube.mzn** - 3D cube constraint problem with 3D variable arrays
- **test_array2d_basic.mzn** - Basic `array2d()` initializer with integer values
- **test_array2d_floats.mzn** - `array2d()` initializer with float values
- **test_array3d_basic.mzn** - Basic `array3d()` initializer with integer values
- **test_array2d_error_mismatch.mzn** - Error case: array2d with value count mismatch

## Running Tests

All models are tested via the test suite in `../tests_all/test_array2d_array3d.rs`:

```bash
cd /home/ross/devpublic/zelen
cargo test --test main_tests test_array2d_array3d
```

## Model Status

- ✅ All 2D/3D array tests passing
- ✅ Error handling with enum-based error messages
- ✅ Range expressions in array initializers (e.g., `array2d(1..n, 1..m, [...])`)
