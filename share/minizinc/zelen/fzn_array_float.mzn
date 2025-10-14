%-----------------------------------------------------------------------------%
% Float array operations for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

% Array minimum/maximum
predicate fzn_minimum_float(var float: m, array[int] of var float: x);
predicate fzn_array_float_minimum(var float: m, array[int] of var float: x);
predicate fzn_maximum_float(var float: m, array[int] of var float: x);
predicate fzn_array_float_maximum(var float: m, array[int] of var float: x);

% Array element access
predicate fzn_array_float_element(var int: idx, array[int] of float: x, var float: c);
predicate fzn_array_var_float_element(var int: idx, array[int] of var float: x, var float: c);
