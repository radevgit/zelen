% Test: Float-only variables - edge case with only floats
var float: x;
var float: y;
var float: z;

constraint x + y = 10.5;
constraint z >= 0.0;

solve satisfy;
