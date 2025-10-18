# Zelen - Direct MiniZinc Solver backed by Selen

[![Crates.io](https://img.shields.io/crates/v/zelen.svg?color=blue)](https://crates.io/crates/zelen)
[![Documentation](https://docs.rs/zelen/badge.svg)](https://docs.rs/zelen)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

Zelen is a MiniZinc parser and solver that directly translates MiniZinc models to the [Selen](https://github.com/radevgit/selen) constraint solver, bypassing FlatZinc compilation. It supports a core subset of MiniZinc including constraint satisfaction and optimization problems.

## Features

**Variable Types:**
- Integer variables with domains: `var <min>..<max>: x`
- Boolean variables: `var bool: b`
- Float variables: `var float: f`
- Variable arrays: `array[1..n] of var int: arr`
- Parameter arrays with initialization: `array[1..5] of int: costs = [10, 20, 30, 40, 50]`

**Constraints:**
- Arithmetic: `+`, `-`, `*`, `/`, `%` (modulo)
- Comparison: `=`, `!=`, `<`, `<=`, `>`, `>=`
- Boolean logic: `not`, `/\` (and), `\/` (or), `->` (implication), `<->`
- Global: `all_different`, `element`, `min`, `max`, `sum`
- Aggregation: `forall`, `exists`
- **Nested forall loops**: `forall(i, j in 1..n)(constraint)`

**Solve Types:**
- Satisfaction: `solve satisfy;`
- Minimize: `solve minimize expr;`
- Maximize: `solve maximize expr;`

**Input Formats:**
- MiniZinc model files (`.mzn`)
- Optional data files (`.dzn`) - merged with model before parsing

## Dependencies

Zelen has minimal dependencies:

| Crate | Purpose | Version |
|-------|---------|---------|
| [selen](https://github.com/radevgit/selen) | CSP solver backend | 0.14+ |
| [clap](https://docs.rs/clap) | CLI argument parsing | 4.5+ |

## Installation

### As a Binary

Build from source:

```bash
git clone https://github.com/radevgit/zelen
cd zelen
cargo build --release
```

The binary will be at `target/release/zelen`.

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
zelen = "0.4"
selen = "0.14"
```

## Usage

### Using as a Library

Parse and solve MiniZinc models from Rust:

```rust
use zelen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        var 1..10: x;
        var 1..10: y;
        constraint x + y = 15;
        solve satisfy;
    "#;

    // Parse and translate MiniZinc to Selen model
    let model = zelen::build_model(source)?;
    
    // Solve the model
    let solution = model.solve()?;
    
    println!("Solution found!");
    println!("x = {}", solution.get_int(/* var_id */));
    
    Ok(())
}
```

**Advanced: Access variable information**

```rust
use zelen::Translator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        var 1..10: x;
        var 1..10: y;
        constraint x + y = 15;
        solve satisfy;
    "#;

    // Parse the source
    let ast = zelen::parse(source)?;
    
    // Translate to model with variable tracking
    let model_data = Translator::translate_with_vars(&ast)?;
    
    // Now we have variable names and IDs
    for (name, var_id) in &model_data.int_vars {
        println!("Variable: {} -> {:?}", name, var_id);
    }
    
    // Solve
    let solution = model_data.model.solve()?;
    
    // Print results with names
    for (name, var_id) in &model_data.int_vars {
        let value = solution.get_int(*var_id);
        println!("{} = {}", name, value);
    }
    
    Ok(())
}
```

### Command Line

```bash
# Solve a MiniZinc model
./target/release/zelen examples/models/test_cli.mzn

# Solve with data file
./target/release/zelen examples/models/test_model.mzn examples/models/test_data.dzn

# With options
./target/release/zelen -v -s examples/models/test_cli.mzn  # Verbose + statistics
./target/release/zelen -a examples/models/test_cli.mzn     # Find all solutions
```

### Command-Line Options

```
USAGE:
    zelen [OPTIONS] <MODEL> [DATA]

ARGS:
    <MODEL>    MiniZinc model file (.mzn)
    <DATA>     Optional MiniZinc data file (.dzn)

OPTIONS:
    -a, --all-solutions         Find all solutions (satisfaction problems)
    -n, --num-solutions <N>     Stop after N solutions
    -i, --intermediate          Print intermediate solutions (optimization)
    -s, --statistics            Print solver statistics
    -v, --verbose               Verbose output with progress
    -t, --time <MS>             Time limit in milliseconds
    --mem-limit <MB>            Memory limit in MB
    -h, --help                  Print help information
    -V, --version               Print version
```

### Example: 4-Queens

Model file (`queens.mzn`):
```minizinc
var 1..4: q1;
var 1..4: q2;
var 1..4: q3;
var 1..4: q4;

constraint q1 != q2;
constraint q1 != q3;
constraint q1 != q4;
constraint q2 != q3;
constraint q2 != q4;
constraint q3 != q4;

constraint q1 + 1 != q2;
constraint q2 + 1 != q3;
constraint q3 + 1 != q4;

solve satisfy;
```

Run:
```bash
./target/release/zelen queens.mzn
```

Output:
```
q1 = 2;
q2 = 4;
q3 = 1;
q4 = 3;
----------
```

## Examples

The repository includes runnable examples:

```bash
# Run an example program (with --release for better performance)
cargo run --release --example queens4      # 4-Queens solver
cargo run --release --example solve_nqueens # N-Queens solver
cargo run --release --example bool_float_demo  # Boolean and float operations
```

See `examples/` directory for source code and `examples/models/` for test MiniZinc files.

## Implementation Status

### Completed Features
- ✅ Variable declarations and arrays
- ✅ Integer, boolean, and float types
- ✅ Arithmetic and comparison operators
- ✅ Boolean logic operators
- ✅ Global constraints: `all_different`, `element`
- ✅ Aggregates: `min`, `max`, `sum`
- ✅ Forall loops (single and nested generators)
- ✅ Array initialization with literals
- ✅ Modulo operator
- ✅ Satisfy/Minimize/Maximize
- ✅ Multiple input formats (.mzn and .dzn files)
- ✅ **Enumerated types**: `enum Color = {Red, Green, Blue};` and `var Color: x;`
- ✅ **Array2D and Array3D** types with proper flattening

### Not Supported
- ❌ Set operations
- ❌ Complex comprehensions beyond forall
- ❌ Advanced global constraints (cumulative, circuit, etc.)
- ❌ Search annotations
- ❌ Some output predicates
- ❌ Include directives (globals.mzn not needed for current model set)
```

## Architecture

```
MiniZinc Source → Parser → AST → Translator → Selen Model → Selen Solver → Solution
```

**Components:**
- `parser.rs` - MiniZinc parser (recursive descent)
- `translator.rs` - Converts AST to Selen model
- `main.rs` - CLI interface with verbose output


## Relationship with Selen

Zelen uses [Selen](https://github.com/radevgit/selen) v0.14+ as its constraint solver backend. Selen provides the core CSP solving engine, while Zelen adds MiniZinc parsing and direct model translation.

## See Also

- [Selen](https://github.com/radevgit/selen) - The underlying CSP solver
- [MiniZinc](https://www.minizinc.org/) - Constraint modeling language

