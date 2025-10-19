% Test: Undefined variable reference
var 1..10: x;
var 1..10: y;

% ERROR: using undefined variable 'z'
constraint x + z = 15;

solve satisfy;
