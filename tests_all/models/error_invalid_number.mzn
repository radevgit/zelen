% Test: Invalid number format
var 1..10: x;

constraint x = 12a3;  % ERROR: invalid number syntax

solve satisfy;
