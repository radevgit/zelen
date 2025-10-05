%-----------------------------------------------------------------------------%
% Set membership constraints for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_set_in(var int: x, set of int: s);
predicate fzn_set_in_reif(var int: x, set of int: s, var bool: b);
