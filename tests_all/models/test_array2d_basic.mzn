% Test basic array2d initialization with integer values
int: n = 3;
int: m = 4;

array[1..n, 1..m] of int: matrix = array2d(1..n, 1..m, 
  [1, 2, 3, 4,
   5, 6, 7, 8,
   9, 10, 11, 12]);

output [
  "matrix[1,1] = ", show(matrix[1,1]), "\n",
  "matrix[2,3] = ", show(matrix[2,3]), "\n",
  "matrix[3,4] = ", show(matrix[3,4]), "\n",
];
