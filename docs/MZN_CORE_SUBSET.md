# MiniZinc Core Subset Specification

**Project**: Zelen - Direct MiniZinc Support  
**Date**: October 15, 2025  
**Status**: Phase 1 MVP Complete ✅

## Quick Summary

### ✅ What Works Now (Phase 1+ Complete)
- Parse MiniZinc to AST (lexer + recursive descent parser)
- Translate AST directly to Selen Model objects
- Integer variables with domains: `var 1..10: x`
- Boolean variables: `var bool: flag`
- Float variables with domains: `var 0.0..1.0: probability`
- Variable arrays (int, bool, float): `array[1..n] of var 1..n: x`
- Parameters (int, bool, float): `int: n = 5;`, `bool: enabled = true;`, `float: pi = 3.14159;`
- Binary constraints: `x < y`, `x + y <= 10`
- Arithmetic in constraints: `+`, `-`, `*`, `/`, `mod`
- **Boolean logical operations**: `/\` (AND), `\/` (OR), `not` (NOT), `->` (implies), `<->` (iff)
- **Float arithmetic in constraints**: All arithmetic operators work with floats
- **Array indexing in constraints**: `x[i] == value`, `x[1] < 5`
- Global constraint: `alldifferent(queens)`
- Direct execution and solution extraction
- 34 unit tests passing, 7 working examples

### ❌ What's Missing (Phase 2)
- Array aggregates: `sum(x)`, `product(x)`, etc.
- Forall loops: `forall(i in 1..n) (...)`
- Element constraint with variable indices: `x[y] == z` (where y is a variable)
- Optimization: `minimize`/`maximize`
- Output formatting
- String types and operations
- Set types and operations

### 📊 Test Results
```
✅ 34/34 unit tests passing
✅ Parser handles 6/7 examples (comprehensions Phase 2)
✅ Translator solves simple N-Queens (column constraints)
✅ Boolean logic fully working (AND, OR, NOT, IMPLIES, IFF)
✅ Examples: solve_nqueens, queens4, simple_constraints, compiler_demo, bool_float_demo, boolean_logic_demo
```

## Overview

This document defines the **core subset** of MiniZinc that Zelen supports directly, bypassing FlatZinc compilation. The goal is to support 80% of practical constraint models with 20% of the language complexity.

### Architecture

**New Approach** (Implemented ✅):
```
MiniZinc Source → AST → Selen Model → Execute & Solve
```

**Previous Approach** (Deprecated):
```
MiniZinc → AST → String (Rust code) → Compile & Run
```

The new architecture builds Selen Model objects directly, enabling:
- ✅ Immediate execution without code generation
- ✅ Runtime model manipulation
- ✅ Direct solution access
- ✅ Better error messages
- ✅ Simpler implementation

### Design Principles

1. **Preserve Structure**: Keep arrays, logical groupings, and semantic meaning ✅
2. **Incremental Implementation**: Start small, expand based on real needs ✅
3. **Clear Semantics**: Every feature has well-defined mapping to Selen ✅
4. **Practical Focus**: Prioritize features used in real models ✅
5. **Fail Fast**: Reject unsupported features with clear error messages ✅

## Phase 1: Core Features (MVP) ✅

### 1.1 Type System

#### Supported Types ✅

**Scalar Types:**
```minizinc
% Integer variables (unconstrained) - ✅ IMPLEMENTED
var int: count;
par int: n = 10;                      % ✅ Parameters work

% Integer variables with domains - ✅ IMPLEMENTED  
var 1..10: digit;
var 0..n: index;                      % ✅ Expression domains work

% Boolean variables - ✅ IMPLEMENTED
var bool: flag;
par bool: enabled = true;             % ✅ Boolean parameters work

% Float variables with domains - ✅ IMPLEMENTED
var float: unbounded;                 % Unconstrained float
var 0.0..1.0: probability;            % ✅ Float domains work
par float: pi = 3.14159;              % ✅ Float parameters work
```

**Status:**
- ✅ `var int` → `model.int(i32::MIN, i32::MAX)`
- ✅ `var 1..10` → `model.int(1, 10)`
- ✅ `par int: n = 5` → Compile-time evaluation
- ✅ `var 1..n` → Domain expressions evaluated with parameters
- ✅ `var bool` → `model.bool()`
- ✅ `par bool: b = true` → Compile-time evaluation
- ✅ `var float` → `model.float(f64::MIN, f64::MAX)`
- ✅ `var 0.0..1.0` → `model.float(0.0, 1.0)`
- ✅ `par float: f = 3.14` → Compile-time evaluation
- ❌ Set domains `{1, 3, 5, 7, 9}` (not yet implemented)

**Array Types:** ✅
```minizinc
% 1D arrays with integer index sets - ✅ IMPLEMENTED
array[1..n] of var int: x;
array[1..5] of int: constants = [1, 2, 3, 4, 5];

% Arrays with constrained elements - ✅ IMPLEMENTED
array[1..n] of var 1..10: digits;

% Boolean and float arrays - ✅ IMPLEMENTED
array[1..5] of var bool: flags;
array[1..n] of var 0.0..1.0: probabilities;

% Implicitly-indexed arrays - ✅ IMPLEMENTED
array[int] of var 1..4: flags;
array[bool] of var 0..10: choices;
```

**Status:**
- ✅ `array[1..n] of var 1..10` → `model.ints(n, 1, 10)`
- ✅ `array[1..n] of var bool` → `model.bools(n)`
- ✅ `array[1..n] of var 0.0..1.0` → `model.floats(n, 0.0, 1.0)`
- ✅ Index set size calculation from expressions
- ✅ Constrained element domains for all types
- ❌ Parameter arrays (not yet implemented)
- ❌ Array initialization expressions

#### NOT Supported in Phase 1

- ❌ Set domains `var {1, 3, 5, 7, 9}` - Phase 2
- ❌ Multi-dimensional arrays - Phase 2
- ❌ Enumerated types - Phase 2
- ❌ Tuple/Record types - Phase 3
- ❌ Option types (`opt int`) - Phase 3
- ❌ Set variables (`var set of int`) - Phase 3
- ❌ String variables (only for output) - Phase 3

### 1.2 Expressions

#### Arithmetic Expressions ✅
```minizinc
% Basic operations - ✅ IMPLEMENTED
x + y              % ✅ model.add(x, y)
x - y              % ✅ model.sub(x, y)
x * y              % ✅ model.mul(x, y)
x div y            % ✅ model.div(x, y)
-x                 % ✅ Unary minus

% Comparisons - ✅ IMPLEMENTED in constraints
x < y              % ✅ model.new(x.lt(y))
x <= y             % ✅ model.new(x.le(y))
x > y              % ✅ model.new(x.gt(y))
x >= y             % ✅ model.new(x.ge(y))
x == y             % ✅ model.new(x.eq(y))
x != y             % ✅ model.new(x.ne(y))
```

**Status:**
- ✅ Arithmetic in constraints: `constraint x + y < 15`
- ✅ Nested expressions: `constraint (x + 1) * 2 < y`
- ✅ Integer literals as constants
- ✅ Variable references
- ✅ Parameter references (evaluated at translation time)
- ❌ `x mod y` (not yet implemented)
- ❌ Arithmetic expressions in variable declarations (e.g., `var x+1..y`)

#### Boolean Expressions ❌
```minizinc
% Logical operations - NOT YET IMPLEMENTED
a /\ b           % AND
a \/ b           % OR
a -> b           % Implication
a <-> b          % Bi-implication
not a            % Negation
a xor b          % Exclusive OR
```

**Status:** Phase 2

#### Array Operations ❌
```minizinc
% Array access - NOT YET IMPLEMENTED
x[i]
x[i+1]

% Array literals - PARSED but not in constraints yet
[1, 2, 3, 4, 5]
[x, y, z]

% Array functions - NOT YET IMPLEMENTED
sum(x)           % Sum of elements
product(x)       % Product of elements
min(x)           % Minimum element
max(x)           % Maximum element
length(x)        % Array length
```

**Status:** Phase 2

#### Set Operations (on fixed sets) ❌
```minizinc
% Set literals - NOT YET IMPLEMENTED
{1, 2, 3}
1..10            % ✅ Used in domains only

% Set membership - NOT YET IMPLEMENTED
x in 1..10
x in {2, 4, 6, 8}

% Set operations - NOT YET IMPLEMENTED
card(1..n)       % Cardinality
min(1..n)        % Minimum
max(1..n)        % Maximum
```

**Status:** Phase 2

### 1.3 Constraints

#### Basic Constraints ✅
```minizinc
% Relational constraints - ✅ IMPLEMENTED
constraint x < y;               % ✅ model.new(x.lt(y))
constraint x + y == 10;         % ✅ Arithmetic + comparison
constraint x <= y + 5;          % ✅ Complex expressions

% Examples that work:
constraint x < y;               % ✅
constraint x + y < 15;          % ✅
constraint x * 2 >= y;          % ✅
constraint (x + 1) - y != 0;    % ✅
```

**Status:**
- ✅ Binary comparisons: `<`, `<=`, `>`, `>=`, `==`, `!=`
- ✅ Arithmetic in constraints: `+`, `-`, `*`, `/`
- ✅ Nested expressions
- ✅ Variable and parameter references
- ❌ Boolean constraints (`flag1 \/ flag2`) - Phase 2
- ❌ Implication (`enabled -> (x > 0)`) - Phase 2
- ❌ Array aggregates (`sum(arr) <= 100`) - Phase 2

#### Global Constraints (Priority Order)

**High Priority** ✅
```minizinc
% All different - ✅ IMPLEMENTED
constraint alldifferent(x);     % ✅ model.alldiff(&x)
constraint all_different(x);    % ✅ Alias supported
```

**Status:**
- ✅ `alldifferent` / `all_different` on arrays
- ❌ `element` constraint - Phase 2
- ❌ Array indexing in constraints - Phase 2

**Medium Priority** ❌
```minizinc
% NOT YET IMPLEMENTED
constraint cumulative(start, duration, resource, capacity);
constraint table(x, allowed_tuples);
```

**Status:** Phase 2

**Lower Priority** ❌
```minizinc
% NOT YET IMPLEMENTED
constraint sort(x, y);
constraint count(x, value) == n;
constraint global_cardinality(x, cover, counts);
```

**Status:** Phase 2-3

### 1.4 Solve Items

```minizinc
% Satisfaction problem - ✅ IMPLEMENTED
solve satisfy;

% Optimization problems - ❌ NOT YET (parsed but not translated)
solve minimize cost;
solve maximize profit;

% With annotations - ❌ Phase 2
solve :: int_search(x, input_order, indomain_min) 
      satisfy;
```

**Status:**
- ✅ `solve satisfy` → Default solving
- ❌ `solve minimize/maximize` → Phase 2 (Selen supports it, need to wire up)
- ❌ Search annotations → Phase 2

### 1.5 Output Items

```minizinc
% Output items - ❌ PARSED but IGNORED
output ["x = ", show(x), "\n"];
output ["Solution: ", show(queens), "\n"];
output ["The value is \(x)\n"];
```

**Status:**
- ✅ Parsed (doesn't cause errors)
- ❌ Not used (solution extraction done via API)
- ❌ Output formatting → Phase 2

**Current Approach:**
Solutions are accessed programmatically:
```rust
let translated = Translator::translate_with_vars(&ast)?;
match translated.model.solve() {
    Ok(solution) => {
        if let Some(&x) = translated.int_vars.get("x") {
            println!("x = {:?}", solution[x]);
        }
    }
}
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

## Mapping to Selen (Actual Implementation)

### Type Mapping ✅

| MiniZinc | Selen | Status | Notes |
|----------|-------|--------|-------|
| `var int` | `model.int(i32::MIN, i32::MAX)` | ✅ | Unbounded integer |
| `var 1..10` | `model.int(1, 10)` | ✅ | Bounded integer |
| `var 1..n` | `model.int(1, n_value)` | ✅ | Evaluated at translation time |
| `array[1..n] of var int` | `model.ints(n, i32::MIN, i32::MAX)` | ✅ | Integer array |
| `array[1..n] of var 1..10` | `model.ints(n, 1, 10)` | ✅ | Bounded integer array |
| `var bool` | `model.bool()` | ❌ | Phase 2 |
| `var float` | `model.float(f64::MIN, f64::MAX)` | ❌ | Phase 2 |
| `var 0.0..1.0` | `model.float(0.0, 1.0)` | ❌ | Phase 2 |

### Constraint Mapping ✅

| MiniZinc | Selen | Status | Notes |
|----------|-------|--------|-------|
| `x < y` | `model.new(x.lt(y))` | ✅ | Comparison |
| `x <= y` | `model.new(x.le(y))` | ✅ | Less or equal |
| `x > y` | `model.new(x.gt(y))` | ✅ | Greater than |
| `x >= y` | `model.new(x.ge(y))` | ✅ | Greater or equal |
| `x == y` | `model.new(x.eq(y))` | ✅ | Equality |
| `x != y` | `model.new(x.ne(y))` | ✅ | Not equal |
| `x + y` | `model.add(x, y)` | ✅ | Addition (returns new VarId) |
| `x - y` | `model.sub(x, y)` | ✅ | Subtraction |
| `x * y` | `model.mul(x, y)` | ✅ | Multiplication |
| `x / y` | `model.div(x, y)` | ✅ | Division |
| `alldifferent(x)` | `model.alldiff(&x)` | ✅ | Global constraint |
| `x[i] == value` | - | ❌ | Phase 2 (element) |
| `sum(x) <= c` | - | ❌ | Phase 2 (linear) |

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

## Implementation Status

### Phase 1: Parser & Type System ✅
- ✅ Lexer (tokenization) - 22 tokens, comments, strings
- ✅ Parser (core subset grammar) - Recursive descent with precedence climbing
- ✅ AST data structures - Model, Item, Expr, TypeInst, etc.
- ✅ Error reporting - Line/column with caret pointers
- ⚠️ Basic type checker - Minimal (type inference TODO)

### Phase 1: Translator & Execution ✅
- ✅ AST → Selen Model translator (not code generation!)
- ✅ Variable mapping - HashMap<String, VarId>
- ✅ Constraint translation - Binary ops and alldifferent
- ✅ Array handling - Vec<VarId> arrays
- ✅ Solve items - Basic satisfy support
- ✅ Solution extraction - TranslatedModel with variable mappings

### Phase 1: Constraints ✅ (Partial)
- ✅ `alldifferent` / `all_different`
- ✅ Binary comparison constraints (`<`, `<=`, `>`, `>=`, `==`, `!=`)
- ✅ Arithmetic in constraints (`+`, `-`, `*`, `/`)
- ❌ `element` constraint - Phase 2
- ❌ `cumulative` - Phase 2
- ❌ `table` constraint - Phase 2
- ❌ Array operations (`sum`, `product`, etc.) - Phase 2

### Phase 1: Testing & Examples ✅
- ✅ Unit tests - 22 tests passing
- ✅ Integration tests - Parser demo, solver demos
- ✅ Example programs:
  - ✅ `solve_nqueens.rs` - Shows array solution extraction
  - ✅ `queens4.rs` - Visual chessboard output
  - ✅ `compiler_demo.rs` - Translation workflow
  - ✅ `simple_constraints.rs` - Constraint examples
  - ✅ `parser_demo.rs` - Parser testing
- ✅ Documentation - This file!
- ✅ Error messages - Clear with source location

## Example: N-Queens Model (Current Implementation)

### Input (MiniZinc) ✅
```minizinc
% N-Queens Problem - WORKS (column constraints only)
int: n = 4;

% Decision variables: queen position in each row
array[1..n] of var 1..n: queens;

% All queens in different columns
constraint alldifferent(queens);

% Diagonal constraints NOT YET SUPPORTED - Phase 2
% constraint forall(i in 1..n, j in i+1..n) (
%     queens[i] != queens[j] + (j - i) /\
%     queens[i] != queens[j] - (j - i)
% );

solve satisfy;

output ["queens = ", show(queens), "\n"];
```

### Rust Usage (Actual API) ✅
```rust
use zelen::{parse, Translator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        int: n = 4;
        array[1..n] of var 1..n: queens;
        constraint alldifferent(queens);
        solve satisfy;
    "#;

    // Parse MiniZinc to AST
    let ast = parse(source)?;
    
    // Translate AST to Selen Model (direct, not code generation!)
    let translated = Translator::translate_with_vars(&ast)?;
    
    // Solve the model
    match translated.model.solve() {
        Ok(solution) => {
            // Extract solution values using variable mappings
            if let Some(queens) = translated.int_var_arrays.get("queens") {
                print!("queens = [");
                for (i, var_id) in queens.iter().enumerate() {
                    if i > 0 { print!(", "); }
                    if let selen::variables::Val::ValI(val) = solution[*var_id] {
                        print!("{}", val);
                    }
                }
                println!("]");
            }
            
            println!("Stats: {} propagations, {} nodes, {:?}",
                solution.stats.propagation_count,
                solution.stats.node_count,
                solution.stats.solve_time
            );
        }
        Err(e) => {
            println!("No solution: {:?}", e);
        }
    }
    
    Ok(())
}
```

### What Works Now ✅
- ✅ Parse MiniZinc directly
- ✅ Build Selen Model objects (not strings!)
- ✅ Execute immediately
- ✅ Extract solution values
- ✅ Access solve statistics

### What Doesn't Work Yet ❌
- ❌ Diagonal constraints (need `forall` loops)
- ❌ Array indexing in constraints (`queens[i]`)
- ❌ Output formatting (manual extraction instead)
- ❌ Optimization objectives
- ❌ Boolean operations in constraints

## Success Metrics

### Phase 1 Status ✅ (MVP Complete)
- ✅ Can parse N-Queens (column constraints only)
- ✅ Can translate and solve directly (no code generation!)
- ✅ Can handle arrays with variable domains
- ✅ Can evaluate parameter expressions
- ✅ Error messages are clear with source locations
- ✅ Architecture is solid and extensible
- ⚠️ Sudoku requires array indexing (Phase 2)
- ⚠️ Full N-Queens requires forall loops (Phase 2)
- ⚠️ Magic Square requires array operations (Phase 2)

### Quality Metrics Achieved:
- **Tests Passing**: 22/22 unit tests ✅
- **Error Handling**: Clear errors with line/column/caret ✅
- **Architecture**: Direct execution (no string generation) ✅
- **Examples**: 5 working examples demonstrating features ✅
- **Maintainability**: Clean separation (parser/translator/examples) ✅

### What Works:
1. ✅ Integer variables with domains
2. ✅ Integer arrays with constrained elements
3. ✅ Parameters with compile-time evaluation
4. ✅ Binary comparison constraints
5. ✅ Arithmetic expressions in constraints
6. ✅ Alldifferent global constraint
7. ✅ Direct model execution
8. ✅ Solution value extraction

### Next Steps (Phase 2):
1. ❌ Array indexing in constraints (`x[i]`)
2. ❌ Forall loops for diagonal constraints
3. ❌ Boolean variables and operations
4. ❌ Array aggregate functions (`sum`, `product`, etc.)
5. ❌ Element constraint
6. ❌ Optimization (minimize/maximize)
7. ❌ Output item formatting

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
