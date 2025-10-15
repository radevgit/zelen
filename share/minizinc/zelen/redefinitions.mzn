%-----------------------------------------------------------------------------%
% Redefinitions for Zelen solver
% This file redefines high-level predicates to use native Zelen/Selen implementations
%-----------------------------------------------------------------------------%

include "fzn_global_cardinality.mzn";
include "fzn_array_bool.mzn";
include "fzn_bool.mzn";
include "fzn_int.mzn";
include "fzn_array_int.mzn";

% Array boolean operations - redirect to fzn_ versions
predicate array_bool_and(array[int] of var bool: as, var bool: r) =
    fzn_array_bool_and(as, r);

predicate array_bool_or(array[int] of var bool: as, var bool: r) =
    fzn_array_bool_or(as, r);

% Boolean clause - redirect to fzn_ version
predicate bool_clause(array[int] of var bool: pos, array[int] of var bool: neg) =
    fzn_bool_clause(pos, neg);

% Integer comparison with reification - redirect to fzn_ versions
predicate int_eq_reif(var int: x, var int: y, var bool: b) =
    fzn_int_eq_reif(x, y, b);

predicate int_ne_reif(var int: x, var int: y, var bool: b) =
    fzn_int_ne_reif(x, y, b);

predicate int_le_reif(var int: x, var int: y, var bool: b) =
    fzn_int_le_reif(x, y, b);

predicate int_lt_reif(var int: x, var int: y, var bool: b) =
    fzn_int_lt_reif(x, y, b);

predicate int_ge_reif(var int: x, var int: y, var bool: b) =
    fzn_int_ge_reif(x, y, b);

predicate int_gt_reif(var int: x, var int: y, var bool: b) =
    fzn_int_gt_reif(x, y, b);

% Array element access - redirect to fzn_ versions
predicate array_int_element(var int: idx, array[int] of int: x, var int: c) =
    fzn_array_int_element(idx, x, c);

predicate array_var_int_element(var int: idx, array[int] of var int: x, var int: c) =
    fzn_array_var_int_element(idx, x, c);

predicate array_bool_element(var int: idx, array[int] of bool: x, var bool: c) =
    fzn_array_bool_element(idx, x, c);

predicate array_var_bool_element(var int: idx, array[int] of var bool: x, var bool: c) =
    fzn_array_var_bool_element(idx, x, c);

% Boolean linear constraints (sum of booleans)
predicate bool_lin_eq(array[int] of int: a, array[int] of var bool: x, var int: c);
predicate bool_lin_ne(array[int] of int: a, array[int] of var bool: x, var int: c);
predicate bool_lin_le(array[int] of int: a, array[int] of var bool: x, var int: c);
predicate bool_lin_lt(array[int] of int: a, array[int] of var bool: x, var int: c);
predicate bool_lin_ge(array[int] of int: a, array[int] of var bool: x, var int: c);
predicate bool_lin_gt(array[int] of int: a, array[int] of var bool: x, var int: c);

% Boolean sums (convenience predicates)
predicate bool_sum_eq(array[int] of var bool: x, var int: c) =
    bool_lin_eq([1 | i in index_set(x)], x, c);
predicate bool_sum_ne(array[int] of var bool: x, var int: c) =
    bool_lin_ne([1 | i in index_set(x)], x, c);
predicate bool_sum_le(array[int] of var bool: x, var int: c) =
    bool_lin_le([1 | i in index_set(x)], x, c);
predicate bool_sum_lt(array[int] of var bool: x, var int: c) =
    bool_lin_lt([1 | i in index_set(x)], x, c);
predicate bool_sum_ge(array[int] of var bool: x, var int: c) =
    bool_lin_ge([1 | i in index_set(x)], x, c);
predicate bool_sum_gt(array[int] of var bool: x, var int: c) =
    bool_lin_gt([1 | i in index_set(x)], x, c);

% Global cardinality constraint
% Redirect to fzn_global_cardinality which is handled natively
predicate global_cardinality(array[int] of var int: x,
                             array[int] of int: cover,
                             array[int] of var int: counts) =
    fzn_global_cardinality(x, cover, counts);

predicate global_cardinality_low_up(array[int] of var int: x,
                                    array[int] of int: cover,
                                    array[int] of int: lbound,
                                    array[int] of int: ubound) =
    fzn_global_cardinality_low_up(x, cover, lbound, ubound);

predicate global_cardinality_low_up_closed(array[int] of var int: x,
                                           array[int] of int: cover,
                                           array[int] of int: lbound,
                                           array[int] of int: ubound) =
    fzn_global_cardinality_low_up_closed(x, cover, lbound, ubound);
