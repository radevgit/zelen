// Main test file that imports all test modules from tests_all/
// Creates a single executable for all tests

#[path = "../tests_all/test_2d_grid.rs"]
mod test_2d_grid;

#[path = "../tests_all/test_3d_arrays.rs"]
mod test_3d_arrays;

#[path = "../tests_all/test_variable_indexing.rs"]
mod test_variable_indexing;


