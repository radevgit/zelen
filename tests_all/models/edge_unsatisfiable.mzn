% Test: Unsatisfiable problem - conflicting constraints
% These constraints are mutually exclusive
var 1..10: x;

constraint x > 5;   % x must be 6-10
constraint x < 5;   % x must be 1-4

solve satisfy;
