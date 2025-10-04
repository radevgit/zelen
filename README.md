# Zelen - FlatZinc Frontend for Selen CSP Solver

[![Crates.io](https://img.shields.io/crates/v/zelen.svg?color=blue)](https://crates.io/crates/zelen)
[![Documentation](https://docs.rs/zelen/badge.svg)](https://docs.rs/zelen)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

Zelen is a FlatZinc parser and integration library for the [Selen](https://github.com/radevgit/selen) constraint solver. It allows you to solve constraint satisfaction and optimization problems written in the FlatZinc format, which is the intermediate language used by [MiniZinc](https://www.minizinc.org/).

## Features

- ✅ **Complete FlatZinc parser** - Parses FlatZinc models into an AST
- ✅ **Seamless Selen integration** - Maps FlatZinc constraints to Selen's constraint model
- ✅ **Integer & Float constraints** - Full support for both integer and floating-point constraint programming
- ✅ **Extensive constraint support** - Arithmetic, comparison, linear, boolean, global constraints (alldiff, table, etc.)
- ✅ **Array handling** - Full support for arrays and array indexing (including float literals)
- ✅ **Reification** - Support for reified constraints
- ✅ **Type conversions** - int2float, float2int (floor/ceil/round)
- ✅ **Optimization** - Handles both satisfaction and optimization problems (minimize/maximize)
- ✅ **Export feature** - Generate standalone Selen test programs for debugging
- ✅ **High compatibility** - Successfully parses 96%+ of real-world FlatZinc files

## Installation

### As a Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
zelen = "0.1"
```

### As a Command-Line Solver

Build and install the `zelen` binary:

```bash
cargo install zelen
```

Or build from source:

```bash
git clone https://github.com/radevgit/zelen
cd zelen
cargo build --release
# Binary will be in target/release/zelen
```

#### Using with MiniZinc

To use Zelen as a solver backend for MiniZinc:

1. Build the zelen binary (see above)

2. Copy the solver configuration file to MiniZinc's solver directory:

```bash
# Copy the template
cp zelen.msc ~/.minizinc/solvers/

# Edit ~/.minizinc/solvers/zelen.msc and replace the placeholder path
# Change: "executable": "/full/path/to/zelen/target/release/zelen"
# To your actual path, e.g.: "executable": "/home/user/zelen/target/release/zelen"
```

You can find your full path with:

```bash
cd /path/to/zelen && pwd
# Then use that path in the .msc file as: /that/path/target/release/zelen
```

3. Verify MiniZinc can find the solver:

```bash
minizinc --solvers
# Should list "Zelen" as an available solver
```

4. Solve MiniZinc models with Zelen:

```bash
# From MiniZinc source
minizinc --solver zelen model.mzn

# From FlatZinc directly
minizinc --solver zelen model.fzn

# With data file
minizinc --solver zelen -m model.fzn -d data.dzn

# Find all solutions
minizinc --solver zelen -a model.mzn
```

### Converting MiniZinc to FlatZinc

To manually convert a MiniZinc model to FlatZinc format:

```bash
# Compile MiniZinc to FlatZinc (requires specifying a solver)
minizinc --solver gecode --compile model.mzn -o model.fzn

# Then solve with zelen directly
./target/release/zelen model.fzn
```

Note: The `--solver gecode` flag is needed during compilation to properly flatten the model using Gecode's constraint library definitions.

#### Command-Line Options

The `zelen` binary implements the FlatZinc standard command-line interface:

```bash
zelen [OPTIONS] <FILE>

Options:
  -a, --all-solutions      Find all solutions (satisfaction problems)
  -n, --num-solutions <N>  Stop after N solutions
  -i, --intermediate       Print intermediate solutions (optimization)
  -f, --free-search        Free search (ignore search annotations, not yet supported)
  -s, --statistics         Print statistics
  -v, --verbose            Verbose output
  -p, --parallel <N>       Use N parallel threads (not yet supported)
  -r, --random-seed <N>    Random seed (not yet supported)
  -t, --time <MS>          Time limit in milliseconds (0 = use Selen default of 60000ms)
      --mem-limit <MB>     Memory limit in MB (0 = use Selen default of 2000MB)
  -h, --help               Print help
  -V, --version            Print version
```

Example usage:

```bash
# Find one solution
zelen problem.fzn

# Find all solutions with statistics
zelen -a -s problem.fzn

# Find up to 5 solutions with verbose output
zelen -n 5 -v problem.fzn
```

## Quick Start

### Recommended API: FlatZincSolver

The easiest way to use Zelen is with the `FlatZincSolver` - it provides automatic FlatZinc-compliant output:

```rust
use zelen::prelude::*;

let fzn = r#"
    var 1..10: x;
    var 1..10: y;
    constraint int_eq(x, 5);
    constraint int_plus(x, y, 12);
    solve satisfy;
"#;

let mut solver = FlatZincSolver::new();
solver.load_str(fzn)?;
solver.solve()?;

// Automatic FlatZinc-compliant output with statistics
print!("{}", solver.to_flatzinc());
// Outputs:
// x = 5;
// y = 7;
// ----------
// ==========
// %%%mzn-stat: solutions=1
// %%%mzn-stat: nodes=0
// %%%mzn-stat: propagations=0
// %%%mzn-stat: solveTime=0.001
// %%%mzn-stat-end
```

### Low-Level API: Direct Model Integration

For more control, use the Model integration API:

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

### Configurable Output and Statistics

Control statistics and solution enumeration:

```rust
use zelen::prelude::*;

let fzn = "var 1..10: x; solve satisfy;";

// Configure solver options
let mut solver = FlatZincSolver::new();
solver.with_statistics(true);       // Enable/disable statistics
solver.max_solutions(3);            // Find up to 3 solutions
solver.find_all_solutions();        // Find all solutions

solver.load_str(fzn)?;
solver.solve()?;

// Get formatted output
let output = solver.to_flatzinc();  // Returns String
solver.print_flatzinc();            // Prints directly

// Access solutions programmatically
let count = solver.solution_count();
let solution = solver.get_solution(0);
```

### FlatZinc Specification Compliance

Zelen follows the [FlatZinc specification](https://docs.minizinc.dev/en/stable/fzn-spec.html#output) exactly:

**Output Format:**
- Variable assignments: `varname = value;`
- Solution separator: `----------`
- Search complete: `==========`
- Unsatisfiable: `=====UNSATISFIABLE=====`

**Statistics Format** (optional, configurable):
```
%%%mzn-stat: solutions=1
%%%mzn-stat: nodes=10
%%%mzn-stat: failures=0
%%%mzn-stat: propagations=21
%%%mzn-stat: variables=4
%%%mzn-stat: propagators=1
%%%mzn-stat: solveTime=0.001
%%%mzn-stat: peakMem=1.00
%%%mzn-stat-end
```

All statistics are automatically extracted from Selen's solver:
- **Standard** (FlatZinc spec): solutions, nodes, failures, solveTime (seconds), peakMem (MB)
- **Extended**: propagations, variables, propagators

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

The repository includes comprehensive examples demonstrating different aspects of the library:

### Basic Usage
- **`simple_usage.rs`** - Basic constraint solving with FlatZincContext API
- **`clean_api.rs`** - High-level FlatZincSolver API with automatic output formatting
- **`solver_demo.rs`** - Demonstrates solving various constraint problem types

### FlatZinc Integration
- **`flatzinc_simple.rs`** - Simple FlatZinc model solving
- **`flatzinc_output.rs`** - FlatZinc-compliant output formatting

### Multiple Solutions & Configuration
- **`multiple_solutions.rs`** - Enumerate multiple solutions with configurable limits
- **`spec_compliance.rs`** - FlatZinc specification compliance demonstration
- **`optimization_test.rs`** - Minimize/maximize with optimal and intermediate solutions

### Statistics & Monitoring
- **`enhanced_statistics.rs`** - All available solver statistics from Selen
- **`statistics_units.rs`** - Statistics unit verification (seconds, megabytes)

Run any example with:
```bash
cargo run --example <name>
# For instance:
cargo run --example clean_api
cargo run --example multiple_solutions
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

