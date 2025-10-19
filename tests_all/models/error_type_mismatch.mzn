% Test: Type mismatch - assigning wrong type
var int: x;
var bool: b;

% ERROR: comparing int with bool
constraint x = b;

solve satisfy;
