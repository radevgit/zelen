%-----------------------------------------------------------------------------%
% Exactly constraint for Zelen solver
% Requires exactly 'n' variables in 'x' to take the value 'v'
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_exactly_int(int: n, array [int] of var int: x, int: v);
