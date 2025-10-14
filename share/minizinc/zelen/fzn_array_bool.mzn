%-----------------------------------------------------------------------------%
% Boolean array operations for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

% Array element access
predicate fzn_array_bool_element(var int: idx, array[int] of bool: x, var bool: c);
predicate fzn_array_var_bool_element(var int: idx, array[int] of var bool: x, var bool: c);

% Array aggregation
predicate fzn_array_bool_and(array[int] of var bool: as, var bool: r);
predicate fzn_array_bool_or(array[int] of var bool: as, var bool: r);
