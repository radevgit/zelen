# Zelen - FlatZinc Frontend for Selen CSP Solver

[![Crates.io](https://img.shields.io/crates/v/zelen.svg?color=blue)](https://crates.io/crates/zelen)
[![Documentation](https://docs.rs/zelen/badge.svg)](https://docs.rs/zelen)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

Zelen is a FlatZinc parser and integration library for the [Selen](https://github.com/radevgit/selen) constraint solver. It allows you to solve constraint satisfaction and optimization problems written in the FlatZinc format, which is the intermediate language used by [MiniZinc](https://www.minizinc.org/).

## Features

- ✅ **Complete FlatZinc parser** - Parses FlatZinc models into an AST
- ✅ **Seamless Selen integration** - Maps FlatZinc constraints to Selen's constraint model
- ✅ **Extensive constraint support** - Arithmetic, comparison, linear, boolean, global constraints (alldiff, table, etc.)
- ✅ **Array handling** - Full support for arrays and array indexing
- ✅ **Reification** - Support for reified constraints
- ✅ **Optimization** - Handles both satisfaction and optimization problems (minimize/maximize)
- ✅ **High compatibility** - Successfully parses 96%+ of real-world FlatZinc files

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zelen = "0.1"
```

## Quick Start

### From FlatZinc String

```rust
use zelen::prelude::*;

let fzn = r#"
    var 1..10: x;
    var 1..10: y;
    constraint int_eq(x, y);
    constraint int_lt(x, 5);
    solve satisfy;
"#;

let mut model = Model::default();
model.from_flatzinc_str(fzn)?;

match model.solve() {
    Ok(solution) => println!("Solution found: {:?}", solution),
    Err(_) => println!("No solution exists"),
}
```

### From FlatZinc File

```rust
use zelen::prelude::*;

let mut model = Model::default();
model.from_flatzinc_file("problem.fzn")?;

let solution = model.solve()?;
println!("Solution: {:?}", solution);
```

### N-Queens Example

```rust
use zelen::prelude::*;

// 4-Queens problem in FlatZinc
let fzn = r#"
    array[1..4] of var 1..4: queens;
    constraint all_different(queens);
    constraint all_different([queens[i] + i | i in 1..4]);
    constraint all_different([queens[i] - i | i in 1..4]);
    solve satisfy;
"#;

let mut model = Model::default();
model.from_flatzinc_str(fzn)?;

if let Ok(solution) = model.solve() {
    println!("Found a solution for 4-Queens!");
}
```

### Optimization Example

```rust
use zelen::prelude::*;

let fzn = r#"
    var 1..100: x;
    var 1..100: y;
    constraint int_plus(x, y, 50);
    solve minimize x;
"#;

let mut model = Model::default();
model.from_flatzinc_str(fzn)?;

if let Ok(solution) = model.solve() {
    println!("Optimal solution found");
}
```

### FlatZinc-Compliant Output

Zelen provides a formatter to output solutions according to the FlatZinc specification:

```rust
use zelen::prelude::*;
use zelen::output::format_solution;
use std::collections::HashMap;

let fzn = r#"
    var 1..10: x;
    var 1..10: y;
    constraint int_eq(x, 5);
    constraint int_eq(y, 3);
    solve satisfy;
"#;

let mut model = Model::default();
model.from_flatzinc_str(fzn)?;

match model.solve() {
    Ok(solution) => {
        // Get variable names from the parser (you'd track these during parsing)
        let var_names = HashMap::from([
            (x_var_id, "x".to_string()),
            (y_var_id, "y".to_string()),
        ]);
        
        // Format according to FlatZinc spec
        let output = format_solution(&solution, &var_names);
        print!("{}", output);
        // Outputs:
        // x = 5;
        // y = 3;
        // ----------
    }
    Err(_) => {
        println!("{}", zelen::output::format_no_solution());
        // Outputs: =====UNSATISFIABLE=====
    }
}
```

The output format follows the [FlatZinc specification](https://docs.minizinc.dev/en/stable/fzn-spec.html#output):
- Each variable assignment on its own line: `varname = value;`
- Separator line `----------` marks the end of a solution
- `=====UNSATISFIABLE=====` when no solution exists
- `=====UNKNOWN=====` when satisfiability cannot be determined

## Using with MiniZinc

You can use Zelen to solve MiniZinc models by first compiling them to FlatZinc:

```bash
# Compile MiniZinc model to FlatZinc
minizinc --solver gecode -c model.mzn -d data.dzn -o problem.fzn

# Then solve with your Rust program using Zelen
cargo run --release -- problem.fzn
```

## Supported Constraints

### Comparison Constraints
- `int_eq`, `int_ne`, `int_lt`, `int_le`, `int_gt`, `int_ge`
- Reified versions: `int_eq_reif`, `int_ne_reif`, etc.

### Arithmetic Constraints  
- `int_abs`, `int_plus`, `int_minus`, `int_times`, `int_div`, `int_mod`
- `int_min`, `int_max`

### Linear Constraints
- `int_lin_eq`, `int_lin_le`, `int_lin_ne`
- Reified: `int_lin_eq_reif`, `int_lin_le_reif`

### Boolean Constraints
- `bool_eq`, `bool_le`, `bool_not`, `bool_xor`
- `bool_clause`, `array_bool_and`, `array_bool_or`
- `bool2int`

### Global Constraints
- `all_different` - All variables must take different values
- `table_int`, `table_bool` - Table/extensional constraints
- `lex_less`, `lex_lesseq` - Lexicographic ordering
- `nvalue` - Count distinct values
- `global_cardinality` - Cardinality constraints
- `cumulative` - Resource scheduling

### Array Constraints
- `array_int_minimum`, `array_int_maximum`
- `array_int_element`, `array_bool_element`
- `count_eq` - Count occurrences

### Set Constraints
- `set_in`, `set_in_reif` - Set membership

## Architecture

Zelen follows a three-stage pipeline:

1. **Tokenization** (`tokenizer.rs`) - Lexical analysis of FlatZinc source
2. **Parsing** (`parser.rs`) - Recursive descent parser building an AST
3. **Mapping** (`mapper.rs`) - Maps AST to Selen's constraint model

```
FlatZinc Source → Tokens → AST → Selen Model → Solution
```

## Performance

Zelen has been tested on 851 real-world FlatZinc files from the OR-Tools test suite:
- **819 files (96.2%)** parse and solve successfully
- **32 files (3.8%)** use unsupported features (mostly set constraints)

## Examples

Check the `examples/` directory for more complete examples:

```bash
# Basic FlatZinc parsing and solving
cargo run --example flatzinc_simple

# FlatZinc-compliant output formatting
cargo run --example flatzinc_output

# Complete solver demo with multiple problem types
cargo run --example solver_demo
```

## Testing

Run the test suite:

```bash
# Unit and integration tests
cargo test

# Run slower batch tests (tests 819 FlatZinc files)
cargo test -- --ignored
```

## Relationship with Selen

Zelen depends on [Selen](https://github.com/radevgit/selen) v0.9+ as its underlying constraint solver. While Selen provides the core CSP solving capabilities, Zelen adds the FlatZinc parsing and integration layer, making it easy to use Selen with MiniZinc models.


## License

Licensed under the MIT license. See [LICENSE](LICENSE) for details.

## See Also

- [Selen](https://github.com/radevgit/selen) - The underlying CSP solver
- [MiniZinc](https://www.minizinc.org/) - Constraint modeling language
- [FlatZinc Specification](https://docs.minizinc.dev/en/stable/fzn-spec.html)

