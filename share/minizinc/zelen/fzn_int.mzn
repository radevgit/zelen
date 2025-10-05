%-----------------------------------------------------------------------------%
% Integer reified comparison constraints for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_int_eq_reif(var int: x, var int: y, var bool: b);
predicate fzn_int_ne_reif(var int: x, var int: y, var bool: b);
predicate fzn_int_lt_reif(var int: x, var int: y, var bool: b);
predicate fzn_int_le_reif(var int: x, var int: y, var bool: b);
predicate fzn_int_gt_reif(var int: x, var int: y, var bool: b);
predicate fzn_int_ge_reif(var int: x, var int: y, var bool: b);
