%-----------------------------------------------------------------------------%
% Float reified comparison constraints for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_float_eq_reif(var float: x, var float: y, var bool: b);
predicate fzn_float_ne_reif(var float: x, var float: y, var bool: b);
predicate fzn_float_lt_reif(var float: x, var float: y, var bool: b);
predicate fzn_float_le_reif(var float: x, var float: y, var bool: b);
predicate fzn_float_gt_reif(var float: x, var float: y, var bool: b);
predicate fzn_float_ge_reif(var float: x, var float: y, var bool: b);
