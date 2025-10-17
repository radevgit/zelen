% Simple test model for CLI
var 1..10: x;
var 1..10: y;

constraint x + y = 15;

solve satisfy;
