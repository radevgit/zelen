%-----------------------------------------------------------------------------%
% Integer linear constraints for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_int_lin_eq(array[int] of int: coeffs, array[int] of var int: vars, int: c);
predicate fzn_int_lin_le(array[int] of int: coeffs, array[int] of var int: vars, int: c);
predicate fzn_int_lin_ne(array[int] of int: coeffs, array[int] of var int: vars, int: c);
