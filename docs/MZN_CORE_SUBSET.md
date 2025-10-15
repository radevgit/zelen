# MiniZinc Core Subset Specification

**Project**: Zelen - Direct MiniZinc Support  
**Date**: October 15, 2025  
**Status**: Draft v1.0

## Overview

This document defines the **core subset** of MiniZinc that Zelen will support directly, bypassing FlatZinc compilation. The goal is to support 80% of practical constraint models with 20% of the language complexity.

### Design Principles

1. **Preserve Structure**: Keep arrays, logical groupings, and semantic meaning
2. **Incremental Implementation**: Start small, expand based on real needs
3. **Clear Semantics**: Every feature has well-defined mapping to Selen
4. **Practical Focus**: Prioritize features used in real models
5. **Fail Fast**: Reject unsupported features with clear error messages

## Phase 1: Core Features (MVP)

### 1.1 Type System

#### Supported Types

**Scalar Types:**
```minizinc
% Boolean variables
var bool: x;
par bool: is_valid = true;

% Integer variables (unconstrained)
var int: count;
par int: n = 10;

% Float variables (unconstrained)
var float: price;
par float: pi = 3.14159;
```

**Constrained Types:**
```minizinc
% Integer ranges
var 1..10: digit;
var 0..n: index;

% Set domains
var {1, 3, 5, 7, 9}: odd_digit;

% Float ranges
var 0.0..1.0: probability;
```

**Array Types:**
```minizinc
% 1D arrays with integer index sets
array[1..n] of var int: x;
array[1..5] of int: constants = [1, 2, 3, 4, 5];

% Arrays with constrained elements
array[1..n] of var 1..10: digits;

% Implicitly-indexed arrays (list of)
array[int] of var bool: flags;
```

#### NOT Supported in Phase 1

- Multi-dimensional arrays (flatten to 1D)
- Enumerated types (use integers)
- Tuple/Record types
- Option types (`opt int`)
- Set variables (`var set of int`)
- String variables (only for output)

### 1.2 Expressions

#### Arithmetic Expressions
```minizinc
% Basic operations
x + y
x - y
x * y
x div y    % Integer division
x mod y    % Modulo
-x         % Unary minus

% Comparisons
x < y
x <= y
x > y
x >= y
x == y     % or x = y
x != y
```

#### Boolean Expressions
```minizinc
% Logical operations
a /\ b           % AND
a \/ b           % OR
a -> b           % Implication
a <-> b          % Bi-implication
not a            % Negation
a xor b          % Exclusive OR
```

#### Array Operations
```minizinc
% Array access
x[i]
x[i+1]

% Array literals
[1, 2, 3, 4, 5]
[x, y, z]

% Array functions
sum(x)           % Sum of elements
product(x)       % Product of elements
min(x)           % Minimum element
max(x)           % Maximum element
length(x)        % Array length
```

#### Set Operations (on fixed sets)
```minizinc
% Set literals
{1, 2, 3}
1..10

% Set membership
x in 1..10
x in {2, 4, 6, 8}

% Set operations (for domains)
card(1..n)       % Cardinality
min(1..n)        % Minimum
max(1..n)        % Maximum
```

### 1.3 Constraints

#### Basic Constraints
```minizinc
% Relational constraints
constraint x < y;
constraint x + y == 10;
constraint sum(arr) <= 100;

% Boolean constraints
constraint flag1 \/ flag2;
constraint enabled -> (x > 0);
```

#### Global Constraints (Priority Order)

**High Priority** (Week 1-2):
```minizinc
% All different
constraint alldifferent(x);
constraint all_different(x);

% Element constraint
constraint x[i] == value;
```

**Medium Priority** (Week 3-4):
```minizinc
% Cumulative (resource constraints)
constraint cumulative(start, duration, resource, capacity);

% Table constraint (extensional)
constraint table(x, allowed_tuples);
```

**Lower Priority** (As needed):
```minizinc
% Sorting
constraint sort(x, y);

% Counting
constraint count(x, value) == n;

% Global cardinality
constraint global_cardinality(x, cover, counts);
```

### 1.4 Solve Items

```minizinc
% Satisfaction problem
solve satisfy;

% Optimization problems
solve minimize cost;
solve maximize profit;

% With annotations (Phase 2)
solve :: int_search(x, input_order, indomain_min) 
      satisfy;
```

### 1.5 Output Items

```minizinc
% Simple output
output ["x = ", show(x), "\n"];

% Array output
output ["Solution: ", show(queens), "\n"];

% String interpolation
output ["The value is \(x)\n"];
```

### 1.6 Model Structure

```minizinc
% Parameters (fixed at instance-time)
int: n = 10;
array[1..n] of int: weights;

% Decision variables
array[1..n] of var 1..n: queens;

% Constraints
constraint alldifferent(queens);
constraint forall(i in 1..n, j in i+1..n) (
    queens[i] != queens[j] + (j - i) /\
    queens[i] != queens[j] - (j - i)
);

% Solve
solve satisfy;

% Output
output ["queens = \(queens)\n"];
```

## Phase 2: Enhanced Features (After MVP)

### 2.1 Multi-Dimensional Arrays

```minizinc
% 2D arrays (map to 1D internally)
array[1..n, 1..m] of var int: grid;

% Access: grid[i,j] → internal_array[i*m + j]
constraint grid[2,3] == 5;
```

**Implementation Strategy:**
- Parse as multi-dimensional
- Flatten to 1D arrays internally
- Translate index expressions: `[i,j]` → `[i*dim2 + j]`

### 2.2 Array Comprehensions

```minizinc
% Simple comprehensions
array[int] of var int: doubled = [2*i | i in 1..n];

% With conditions
array[int] of var int: evens = [i | i in 1..n where i mod 2 == 0];

% Generator expressions
constraint forall(i in 1..n) (x[i] > 0);
constraint sum(i in 1..n)(cost[i] * x[i]) <= budget;
```

### 2.3 Enumerated Types

```minizinc
% Enum declaration
enum Color = {Red, Green, Blue};

% Enum variables (map to integers internally)
var Color: my_color;

% Usage
constraint my_color != Red;

% Arrays of enums
array[1..n] of var Color: colors;
```

**Implementation Strategy:**
- Map enum to integer range: `Red=1, Green=2, Blue=3`
- Track mapping for output formatting
- Support `enum2int()` and `to_enum()` functions

### 2.4 Let Expressions

```minizinc
% Local variables
constraint let {
    var int: temp = x + y;
} in temp * 2 > z;

% Multiple locals
constraint let {
    int: half = n div 2;
    var int: mid = x[half];
} in mid > 0;
```

**Implementation Strategy:**
- Introduce fresh variables in parent scope
- Substitute references in body expression
- Handle constraints in let properly

### 2.5 User-Defined Predicates

```minizinc
% Predicate definition
predicate adjacent(var int: x, var int: y) =
    abs(x - y) == 1;

% Usage
constraint adjacent(pos[1], pos[2]);
```

**Implementation Strategy:**
- Inline simple predicates
- Build library of common predicates
- Support recursion carefully (detect cycles)

## Phase 3: Advanced Features (Future)

### 3.1 Set Comprehensions
```minizinc
set of int: evens = {2*i | i in 1..n};
```

### 3.2 Annotations
```minizinc
% Search annotations
solve :: int_search(x, first_fail, indomain_min)
      satisfy;

% Variable annotations
var 1..n: x :: is_defined_var;
```

### 3.3 Option Types
```minizinc
var opt 1..n: maybe_value;
constraint occurs(maybe_value) -> (deopt(maybe_value) > 5);
```

## Mapping to Selen

### Type Mapping

| MiniZinc | Selen | Notes |
|----------|-------|-------|
| `var bool` | `model.bool()` | Boolean variable |
| `var int` | `model.int(i32::MIN, i32::MAX)` | Unbounded integer |
| `var 1..10` | `model.int(1, 10)` | Bounded integer |
| `var float` | `model.float(f64::MIN, f64::MAX)` | Unbounded float |
| `var 0.0..1.0` | `model.float(0.0, 1.0)` | Bounded float |
| `array[1..n] of var int` | `model.ints(n, i32::MIN, i32::MAX)` | Integer array |

### Constraint Mapping

| MiniZinc | Selen | Notes |
|----------|-------|-------|
| `x < y` | `model.less_than(&x, &y)` | Comparison |
| `x + y == z` | `model.lin_eq(&[1,1,-1], &[x,y,z], 0)` | Linear equality |
| `x * y == z` | `model.times(&x, &y, &z)` | Multiplication |
| `alldifferent(x)` | `model.all_different(&x)` | Global constraint |
| `sum(x) <= c` | `model.lin_le(&[1;n], &x, c)` | Linear inequality |

## Error Handling

### Unsupported Features

When encountering unsupported features, emit clear error messages:

```rust
UnsupportedFeature {
    feature: "multi-dimensional arrays",
    location: "line 15, column 8",
    workaround: "Flatten to 1D: array[1..n*m] of var int",
    phase: "Phase 2"
}
```

### Type Errors

```rust
TypeError {
    expected: "var int",
    found: "set of int",
    location: "line 23, column 12",
    hint: "Set variables not supported in Phase 1"
}
```

### Syntax Errors

```rust
SyntaxError {
    message: "Expected ';' after constraint",
    location: "line 42, column 30",
    context: "constraint x < y"
}
```

## Testing Strategy

### Unit Tests

Test each component in isolation:
- Parser: MiniZinc → AST
- Type checker: AST → Typed AST
- Compiler: Typed AST → Selen code

### Integration Tests

Test complete models:
```rust
#[test]
fn test_nqueens_4() {
    let mzn = r#"
        int: n = 4;
        array[1..n] of var 1..n: queens;
        constraint alldifferent(queens);
        solve satisfy;
    "#;
    
    let compiled = compile_mzn(mzn).unwrap();
    let solution = run_selen(compiled).unwrap();
    assert_eq!(solution.len(), 2); // 2 solutions for 4-queens
}
```

### Benchmark Models

Standard CSP problems:
1. **N-Queens** (various sizes)
2. **Sudoku** (9x9 grid)
3. **Graph Coloring** (various graphs)
4. **Job Shop Scheduling** (simple instances)
5. **Magic Square** (order 3, 4, 5)

## Implementation Roadmap

### Week 1-2: Parser & Type System
- [ ] Lexer (tokenization)
- [ ] Parser (core subset grammar)
- [ ] AST data structures
- [ ] Basic type checker
- [ ] Error reporting

### Week 3-4: Compiler & Code Generation
- [ ] AST → Selen code generator
- [ ] Variable mapping
- [ ] Constraint translation
- [ ] Array handling
- [ ] Solve items

### Week 5-6: Global Constraints
- [ ] `alldifferent` / `all_different`
- [ ] `element` constraint
- [ ] `cumulative` (if needed)
- [ ] `table` constraint
- [ ] Array operations

### Week 7-8: Testing & Refinement
- [ ] Unit tests
- [ ] Integration tests
- [ ] Benchmark suite
- [ ] Documentation
- [ ] Error message polish

## Example: N-Queens Model

### Input (MiniZinc)
```minizinc
% N-Queens Problem
int: n = 8;

% Decision variables: queen position in each row
array[1..n] of var 1..n: queens;

% All queens in different columns
constraint alldifferent(queens);

% No two queens on same diagonal
constraint forall(i in 1..n, j in i+1..n) (
    queens[i] != queens[j] + (j - i) /\
    queens[i] != queens[j] - (j - i)
);

solve satisfy;

output ["queens = ", show(queens), "\n"];
```

### Output (Selen - Generated Code)
```rust
use selen::prelude::*;

fn main() {
    let mut model = Model::new();
    
    // Parameters
    let n: i32 = 8;
    
    // Decision variables
    let queens = model.ints(n as usize, 1, n);
    
    // All queens in different columns
    model.all_different(&queens);
    
    // No two queens on same diagonal
    for i in 0..n {
        for j in (i+1)..n {
            let offset = j - i;
            model.not_equal(&queens[i as usize], 
                          &model.add(&queens[j as usize], offset));
            model.not_equal(&queens[i as usize], 
                          &model.sub(&queens[j as usize], offset));
        }
    }
    
    // Solve
    let mut solver = model.solve();
    
    // Find and print solution
    if let Some(solution) = solver.next() {
        print!("queens = [");
        for i in 0..n {
            print!("{}", solution.get_int(&queens[i as usize]));
            if i < n - 1 { print!(", "); }
        }
        println!("]");
    } else {
        println!("=====UNSATISFIABLE=====");
    }
}
```

## Success Metrics

### Phase 1 Complete When:
- ✅ Can parse and compile N-Queens
- ✅ Can parse and compile Sudoku
- ✅ Can parse and compile Magic Square
- ✅ All benchmark models run correctly
- ✅ Generated code is readable
- ✅ Error messages are clear and helpful
- ✅ Performance is comparable to FlatZinc path

### Quality Metrics:
- **Code Coverage**: >80% for core modules
- **Error Rate**: <5% false negatives (accepting invalid MiniZinc)
- **Performance**: Within 10% of hand-written Selen
- **Maintainability**: New constraint takes <2 hours to add

## References

- [MiniZinc Specification](https://docs.minizinc.dev/en/stable/spec.html)
- [MiniZinc Tutorial](https://docs.minizinc.dev/en/stable/part_2_tutorial.html)
- [Selen API Documentation](../README.md)
- [FlatZinc Specification](https://docs.minizinc.dev/en/stable/fzn-spec.html) (for comparison)

## Appendix A: Grammar Subset (EBNF)

```ebnf
(* Core MiniZinc Subset Grammar *)

model ::= item*

item ::= var_decl ";"
       | constraint ";"
       | solve ";"
       | output ";"

var_decl ::= type_inst ":" ident [ "=" expr ]

type_inst ::= [ "var" | "par" ] base_type
            | "array" "[" index_set "]" "of" type_inst

base_type ::= "bool"
            | "int"
            | "float"
            | int_range
            | set_literal

int_range ::= int_expr ".." int_expr

constraint ::= "constraint" expr

solve ::= "solve" "satisfy"
        | "solve" "minimize" expr
        | "solve" "maximize" expr

output ::= "output" "[" string_expr_list "]"

expr ::= int_expr
       | bool_expr
       | array_expr
       | call_expr
       | ident
       | literal

(* More detailed rules in parser implementation *)
```

## Appendix B: Limitations & Workarounds

| Limitation | Workaround | Phase |
|------------|------------|-------|
| Multi-dim arrays | Use 1D with index calculations | Phase 2 |
| Enums | Use integers (1, 2, 3...) | Phase 2 |
| Set variables | Represent as boolean arrays | Phase 3 |
| User predicates | Inline manually | Phase 2 |
| Complex comprehensions | Expand to loops | Phase 2 |
| Option types | Use sentinel values (-1, etc.) | Phase 3 |

## Appendix C: FAQ

**Q: Why not support full MiniZinc?**  
A: Full MiniZinc is very complex. This subset covers most practical models while keeping implementation tractable.

**Q: How do I use features not in the subset?**  
A: Either wait for later phases, use FlatZinc fallback, or manually rewrite your model.

**Q: Will my FlatZinc models still work?**  
A: Yes! FlatZinc support remains as fallback for unsupported features.

**Q: What about MiniZinc library functions?**  
A: Phase 1 includes only built-in operations. Phase 2 will add common library predicates.

**Q: How is performance compared to FlatZinc?**  
A: Should be similar or better, as we avoid flattening overhead and preserve structure.

---

*This is a living document. Update as implementation progresses and requirements evolve.*
