%-----------------------------------------------------------------------------%
% Boolean linear constraints for Zelen solver
% These handle weighted sums of boolean variables
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_bool_lin_eq(array[int] of int: coeffs, array[int] of var bool: vars, int: c);
predicate fzn_bool_lin_le(array[int] of int: coeffs, array[int] of var bool: vars, int: c);
predicate fzn_bool_lin_ne(array[int] of int: coeffs, array[int] of var bool: vars, int: c);

% Reified versions
predicate fzn_bool_lin_eq_reif(array[int] of int: coeffs, array[int] of var bool: vars, int: c, var bool: b);
predicate fzn_bool_lin_le_reif(array[int] of int: coeffs, array[int] of var bool: vars, int: c, var bool: b);
predicate fzn_bool_lin_ne_reif(array[int] of int: coeffs, array[int] of var bool: vars, int: c, var bool: b);
