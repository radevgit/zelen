% Test: Empty constraints - satisfaction with no constraints
% Should find solution immediately for any assignment
var 1..5: x;
var 1..5: y;

% No constraints! Just satisfy
solve satisfy;
