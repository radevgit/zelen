%-----------------------------------------------------------------------------%
% At least constraint for Zelen solver
% Requires at least 'n' variables in 'x' to take the value 'v'
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_at_least_int(int: n, array [int] of var int: x, int: v);
