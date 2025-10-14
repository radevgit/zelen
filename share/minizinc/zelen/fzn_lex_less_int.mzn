%-----------------------------------------------------------------------------%
% Lexicographic ordering constraints for Zelen solver
% x <_lex y (strict), x ≤_lex y (non-strict)
% Handled by the Selen CSP solver
%-----------------------------------------------------------------------------%

predicate fzn_lex_less_int(array[int] of var int: x, array[int] of var int: y);

predicate fzn_lex_lesseq_int(array[int] of var int: x, array[int] of var int: y);
