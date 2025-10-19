% Test: Large 2D array - stress test with bigger dimensions
% 10x10 grid - should handle reasonable sizes well
array[1..10, 1..10] of var 1..20: grid;

% Some constraints on corners
constraint grid[1,1] = 1;
constraint grid[10,10] = 20;
constraint grid[5,5] = 10;

solve satisfy;
