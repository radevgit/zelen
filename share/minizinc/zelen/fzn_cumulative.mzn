%-----------------------------------------------------------------------------%
% Cumulative constraint for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_cumulative(array[int] of var int: s,
                         array[int] of var int: d,
                         array[int] of var int: r,
                         var int: b);
