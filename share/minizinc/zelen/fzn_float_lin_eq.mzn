%-----------------------------------------------------------------------------%
% Float linear constraints for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_float_lin_eq(array[int] of float: coeffs, array[int] of var float: vars, float: c);
predicate fzn_float_lin_le(array[int] of float: coeffs, array[int] of var float: vars, float: c);
predicate fzn_float_lin_ne(array[int] of float: coeffs, array[int] of var float: vars, float: c);
