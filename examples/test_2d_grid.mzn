% Simple 2D grid test
% 3x3 grid with variables 1..9
% All different values

int: n = 3;
int: m = 3;

% 2D grid: grid[i,j] for i in 1..n, j in 1..m
array[1..n, 1..m] of var 1..9: grid;

% All cells must be different
constraint alldifferent(grid);

% Simple constraint: grid[1,1] != grid[2,2]
constraint grid[1,1] != grid[2,2];

solve satisfy;

output ["grid = ", show(grid), "\n"];
