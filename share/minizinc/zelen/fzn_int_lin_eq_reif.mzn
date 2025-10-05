%-----------------------------------------------------------------------------%
% Integer linear reified constraints for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_int_lin_eq_reif(array[int] of int: coeffs, array[int] of var int: vars, int: c, var bool: b);
predicate fzn_int_lin_le_reif(array[int] of int: coeffs, array[int] of var int: vars, int: c, var bool: b);
predicate fzn_int_lin_ne_reif(array[int] of int: coeffs, array[int] of var int: vars, int: c, var bool: b);
