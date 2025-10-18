% Test error handling: array2d with value count mismatch
int: n = 3;
int: m = 3;

% This should fail - 3x3 array needs 9 values but only 8 provided
array[1..n, 1..m] of int: matrix = array2d(1..n, 1..m,
  [1, 2, 3, 4, 5, 6, 7, 8]);
