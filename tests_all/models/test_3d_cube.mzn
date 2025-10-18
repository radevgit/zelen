% Simple 3D array test
% Demonstrates 3D multi-dimensional array indexing with constant indices

int: d1 = 2;
int: d2 = 3;
int: d3 = 2;

% 3D array: cube[i,j,k] for i in 1..d1, j in 1..d2, k in 1..d3
array[1..d1, 1..d2, 1..d3] of var 1..6: cube;

% All cells must have different values
constraint alldifferent(cube);

% Some 3D indexing constraints
constraint cube[1,1,1] != cube[1,1,2];
constraint cube[1,1,1] != cube[1,2,1];
constraint cube[1,1,1] != cube[2,1,1];

% More constraints
constraint cube[1,2,2] < cube[2,3,2];
constraint cube[2,1,1] > cube[1,3,1];

solve satisfy;

output ["cube = ", show(cube), "\n"];
