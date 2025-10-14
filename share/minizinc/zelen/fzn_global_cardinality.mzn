%-----------------------------------------------------------------------------%
% Global cardinality constraint for Zelen solver
% Handled natively by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_global_cardinality(array[int] of var int: x,
                                  array[int] of int: cover,
                                  array[int] of var int: counts);

predicate fzn_global_cardinality_low_up(array[int] of var int: x,
                                        array[int] of int: cover,
                                        array[int] of int: lbound,
                                        array[int] of int: ubound);

predicate fzn_global_cardinality_low_up_closed(array[int] of var int: x,
                                                array[int] of int: cover,
                                                array[int] of int: lbound,
                                                array[int] of int: ubound);
