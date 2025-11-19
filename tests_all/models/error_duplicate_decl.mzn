% Test: Duplicate variable declaration
var 1..10: x;
var 1..20: x;  % ERROR: variable 'x' already declared

solve satisfy;
