%-----------------------------------------------------------------------------%
% Element constraint for Zelen solver
% array[index] == value
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_array_int_element(var int: idx, array [int] of int: x, var int: c);
predicate fzn_array_var_int_element(var int: idx, array [int] of var int: x, var int: c);
