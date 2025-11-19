% Test: Deeply nested 3D array - boundary case
% Minimal 2x2x2 array to test 3D parsing and basic constraints
array[1..2, 1..2, 1..2] of var 1..10: cube;

% Just test that 3D arrays parse and declare properly
% No indexing constraints (those may not be supported yet)

solve satisfy;
