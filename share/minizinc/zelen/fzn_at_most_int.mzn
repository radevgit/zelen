%-----------------------------------------------------------------------------%
% At most constraint for Zelen solver
% Requires at most 'n' variables in 'x' to take the value 'v'
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_at_most_int(int: n, array [int] of var int: x, int: v);
