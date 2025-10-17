# Zelen Project Structure - Organized

## Root Directory
```
/home/ross/devpublic/zelen/
├── Cargo.toml           # Project configuration
├── Cargo.lock           # Dependency lock file
├── LICENSE              # License file
├── README.md            # Project readme
├── src/                 # Source code
│   ├── main.rs         # CLI entry point with .mzn and .dzn support
│   ├── lib.rs          # Library root
│   ├── parser.rs       # MiniZinc parser
│   ├── translator.rs   # MiniZinc to Selen translator (with forall & array init)
│   ├── ast.rs          # Abstract syntax tree definitions
│   ├── error.rs        # Error types
│   ├── compiler.rs     # Compiler utilities
│   └── mapper.rs       # FlatZinc mapper (legacy)
├── examples/           # Runnable examples
│   ├── models/         # MiniZinc model files (.mzn, .dzn)
│   │   ├── test_cli.mzn
│   │   ├── test_data.dzn
│   │   ├── test_data2.dzn
│   │   ├── test_model.mzn
│   │   └── test_model2.mzn
│   ├── bool_float_demo.rs       # Example: Boolean and float operations
│   ├── boolean_logic_demo.rs    # Example: Boolean logic
│   ├── compiler_demo.rs         # Example: Compiler usage
│   ├── parser_demo.rs           # Example: Parser usage
│   ├── queens4.rs               # Example: N-Queens problem
│   ├── simple_constraints.rs    # Example: Simple constraints
│   └── solve_nqueens.rs         # Example: N-Queens solver
├── tests/              # Test files and integration tests
│   ├── element_test.rs          # Test: Element constraints
│   ├── phase2_demo.rs           # Test: Phase 2 features
│   ├── phase3_demo.rs           # Test: Phase 3 features
│   ├── selen_modulo_test.rs     # Test: Modulo operation
│   ├── selen_modulo_two_vars.rs # Test: Modulo with two variables
│   ├── test_forall.rs           # Test: Forall loops
│   ├── test_parser.rs           # Test: Parser
│   ├── test_translate.rs        # Test: Translator
│   └── verify_modulo_fix.rs     # Test: Modulo fix verification
├── docs/               # Documentation
├── target/             # Build artifacts (generated)
└── zinc/               # MiniZinc library files
```

## Organization Summary

### `/examples/` - Runnable Example Programs
- **Purpose**: Demonstrate how to use Zelen features
- **Contents**: Example Rust programs and MiniZinc model files
- **Subdirectory**: `models/` contains MiniZinc model files (.mzn, .dzn)

### `/tests/` - Test Files
- **Purpose**: Integration tests and development tests
- **Contents**: Test programs that verify functionality

### Root Directory (`/`) - Clean
- ✅ No .mzn or .dzn files
- ✅ No test .rs files
- ✅ Only configuration and documentation

## CLI Usage with Models

```bash
# Using model file only
cargo run -- -v examples/models/test_cli.mzn

# Using model and data files
cargo run -- -v examples/models/test_model2.mzn examples/models/test_data2.dzn

# With options
cargo run -- -v -s examples/models/test_cli.mzn
```

## Features Implemented

### Phase 4 (Complete)
- ✅ Forall loops (single and nested generators)
- ✅ Array initialization expressions
- ✅ CLI with model and data file support

### Phase 3 (Complete)
- ✅ Modulo operator
- ✅ Element constraints

### Phase 2 (Complete)
- ✅ Array operations
- ✅ Float arithmetic

### Phase 1 (Complete)
- ✅ Basic constraints
- ✅ Variable types
- ✅ Simple expressions

## Testing

Run unit tests:
```bash
cargo test --lib
```

Run example programs:
```bash
cargo run --example queens4
cargo run --example solve_nqueens
```

Run test files (integration tests):
```bash
cargo test --test test_forall
```
