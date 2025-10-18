% Simple 4x4 Latin square puzzle
% A simplified version of Sudoku that works with Phase 1 features
% Demonstrates 2D array multi-dimensional indexing

int: n = 4;

% 4x4 grid where each cell is a value from 1 to 4
array[1..n, 1..n] of var 1..n: grid;

% All cells must have different values from 1 to 4
constraint alldifferent(grid);

% Additional constraints using 2D indexing to break symmetry
% Row 1: first two cells are different
constraint grid[1,1] != grid[1,2];

% Column 1: first two cells are different
constraint grid[1,1] != grid[2,1];

% Diagonal cells are different
constraint grid[1,1] != grid[2,2];
constraint grid[2,2] != grid[3,3];
constraint grid[3,3] != grid[4,4];

% Corner constraints
constraint grid[1,1] < grid[1,4];
constraint grid[4,1] < grid[4,4];

solve satisfy;

output ["grid = ", show(grid), "\n"];
