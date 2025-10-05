%-----------------------------------------------------------------------------%
% Boolean constraints for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

% Boolean reified comparisons
predicate fzn_bool_eq_reif(var bool: x, var bool: y, var bool: b);
predicate fzn_bool_le_reif(var bool: x, var bool: y, var bool: b);

% Boolean clause (CNF)
predicate fzn_bool_clause(array[int] of var bool: pos, array[int] of var bool: neg);
