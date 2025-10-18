% Test basic array3d initialization with integer values
int: d1 = 2;
int: d2 = 3;
int: d3 = 2;

array[1..d1, 1..d2, 1..d3] of int: cube = array3d(1..d1, 1..d2, 1..d3,
  [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

output [
  "cube[1,1,1] = ", show(cube[1,1,1]), "\n",
  "cube[2,3,2] = ", show(cube[2,3,2]), "\n",
];
