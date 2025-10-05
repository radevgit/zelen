%-----------------------------------------------------------------------------%
% Nvalue constraint for Zelen solver
% n is the number of distinct values in array x
% Handled by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_nvalue(var int: n, array[int] of var int: x);
