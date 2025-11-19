% Test: Degenerate domain - single-value variable (minimal domain)
% All variables must be single value
var 5..5: x;
var 10..10: y;
var 1..1: z;

constraint x + y = 15;  % x=5, y=10 only valid values

solve satisfy;
