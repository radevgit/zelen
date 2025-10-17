MZN Specification: https://docs.minizinc.dev/en/stable/spec.html
EBNF basics: https://docs.minizinc.dev/en/stable/spec.html#notation
High-level Model Structure: https://docs.minizinc.dev/en/stable/spec.html#high-level-model-structure 


1. What types should be supported?
    - Built-in Scalar Types: https://docs.minizinc.dev/en/stable/spec.html#built-in-scalar-types-and-type-insts
    - Compound Types https://docs.minizinc.dev/en/stable/spec.html#built-in-compound-types-and-type-insts
    

// Phase 1: Core features (2-3 weeks)
- var/par bool, int, float
- Constrained ranges: var 1..10: x
- 1D arrays with any index set
- Basic constraints: =, <, >, etc.
- Arithmetic operations
- Global constraints: alldifferent, all_different, etc.
- Solve satisfy/minimize/maximize

// Phase 2: Enhanced (2-3 weeks)  
- Multi-dimensional arrays (flatten intelligently)
- Array comprehensions [expr | i in range]
- Set operations
- Enums → map to integers
- Simple let expressions

// Phase 3: Advanced (as needed)
- User-defined predicates (inline or library)
- More comprehensions
- Annotations (for search hints)