%-----------------------------------------------------------------------------%
% Integer array operations for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

% Array minimum/maximum
predicate fzn_minimum_int(var int: m, array[int] of var int: x);
predicate fzn_array_int_minimum(var int: m, array[int] of var int: x);
predicate fzn_maximum_int(var int: m, array[int] of var int: x);
predicate fzn_array_int_maximum(var int: m, array[int] of var int: x);

% Array element access
predicate fzn_array_int_element(var int: idx, array[int] of int: x, var int: c);
predicate fzn_array_var_int_element(var int: idx, array[int] of var int: x, var int: c);
