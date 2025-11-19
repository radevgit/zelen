% Test: Boolean only - all boolean variables, no integers
var bool: a;
var bool: b;
var bool: c;

% Boolean constraints
constraint a /\ b;  % a AND b
constraint c;       % c must be true

solve satisfy;
