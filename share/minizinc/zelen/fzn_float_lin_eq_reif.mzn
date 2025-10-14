%-----------------------------------------------------------------------------%
% Float linear reified constraints for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_float_lin_eq_reif(array[int] of float: coeffs, array[int] of var float: vars, float: c, var bool: b);
predicate fzn_float_lin_le_reif(array[int] of float: coeffs, array[int] of var float: vars, float: c, var bool: b);
predicate fzn_float_lin_ne_reif(array[int] of float: coeffs, array[int] of var float: vars, float: c, var bool: b);
