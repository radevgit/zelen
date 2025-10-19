# Error Handling and Validation

This document describes how Zelen handles errors and validates input. All error scenarios are tested with dedicated test models in `tests_all/models/error_*.mzn`.

## Error Categories

### 1. Lexical Errors (Lexer Phase)

**Description:** Invalid token sequences

**Examples:**
- `UnexpectedChar(char)` - Invalid character in source
- `UnterminatedString` - String missing closing quote
- `InvalidNumber(String)` - Malformed number literal

**Test Case:** `error_invalid_number.mzn`
```
constraint x = 12a3;  % Invalid: digits followed by letters
Error: UnexpectedToken (detected at parsing, not lexing)
```

**Status:** ✅ Detected and reported

### 2. Parser Errors (Parser Phase)

**Description:** Syntax violations

**Examples:**
- `UnexpectedToken { expected, found }` - Wrong token type
- `UnexpectedEof` - File ends unexpectedly
- `InvalidExpression(String)` - Malformed expression
- `InvalidTypeInst(String)` - Invalid type declaration

**Test Cases:**
- `error_invalid_number.mzn` - Invalid number syntax
  ```
  Error: UnexpectedToken { expected: "Semicolon", found: "Ident(\"a3\")" }
  ```

**Status:** ✅ Robust parsing with clear diagnostics

### 3. Semantic Errors (Translator Phase)

**Description:** Type mismatches, undefined references, duplicate declarations

#### 3a. Variable Reference Errors

**Example:** Undefined variable
```minizinc
var 1..10: x;
constraint x + z = 15;  % ERROR: 'z' not defined
```

**Test Case:** `error_undefined_var.mzn`
```
Error: Message("Undefined variable or parameter: 'z'")
```

**Status:** ✅ Properly detected with clear variable name

#### 3b. Duplicate Declaration Errors

**Example:** Variable declared twice
```minizinc
var 1..10: x;
var 1..20: x;  % ERROR: 'x' already declared
```

**Test Case:** `error_duplicate_decl.mzn`
```
Error: DuplicateDeclaration("x")
```

**Status:** ✅ Detected and reported - fixed in v0.5.1

#### 3c. Type Mismatch Errors

**Example:** Assigning wrong type to variable
```minizinc
var int: x;
var bool: b;
constraint x = b;  % Mixing int and bool
```

**Test Case:** `error_type_mismatch.mzn`
```
Result: x = 0; b = 0;  (Type coercion - no error)
```

**Status:** ⚠️ Not enforced - by design. Zelen allows type coercion where the underlying CSP solver can make it work. This is a deliberate trade-off for flexibility.

**Why:** Zelen's type system is intentionally permissive to:
- Allow flexibility in modeling
- Leverage the underlying Selen solver's flexible typing
- Reduce barriers for rapid prototyping

**Note:** Strict MiniZinc type checking would normally catch this, but would require a full type inference/checking pass during translation.

### 4. Array-Related Errors

**Description:** Array size mismatches, dimension problems

#### 4a. Array Size Mismatch

**Example:** Declared 3 elements, provided 2
```minizinc
array[1..3] of int: data = [1, 2];  % ERROR: size mismatch
```

**Test Case:** `error_array_size_mismatch.mzn`
```
Error: ArraySizeMismatch { declared: 3, provided: 2 }
```

**Status:** ✅ Detected with exact counts

#### 4b. Array2D/Array3D Dimension Errors

**Error Types:**
- `Array2DSizeMismatch { declared_rows, declared_cols, provided_rows, provided_cols }`
- `Array3DSizeMismatch { declared_d1, declared_d2, declared_d3, ... }`
- `Array2DInvalidContext` - 2D array in wrong context
- `Array2DValuesMustBeLiteral` - Non-literal values in 2D initialization

**Status:** ✅ Comprehensive checking for multi-dimensional arrays

### 5. Unsupported Feature Errors

**Type:** `UnsupportedFeature { feature, phase, workaround }`

**Examples:**
- Set operations: `set of int`
- Complex comprehensions beyond `forall`
- Advanced global constraints: `cumulative`, `circuit`
- Search annotations

**Status:** ✅ Graceful error with explanatory messages

### 6. Solver Errors

**Description:** Constraint satisfaction failures

**Type:** `std::result::Result<Solution, SolverError>` from Selen backend

**Example:** Unsatisfiable problem
```minizinc
var 1..10: x;
constraint x > 5;
constraint x < 5;  % Conflicting constraints
```

**Test Case:** `edge_unsatisfiable.mzn`
```
Output: =====UNSATISFIABLE=====
```

**Status:** ✅ Handled gracefully

## Error Reporting Features

### 1. Source Location Tracking

All errors include `Span` information:
```rust
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,        // byte range in source
    pub source: Option<String>,  // original source code
}
```

**Example Output:**
```
Error at line 3, col 12 in "constraint x = z"
      Undefined variable: 'z'
```

### 2. Source Context

Parser errors include source code snippet for debugging:
```
Error: Parse error: Error { 
    kind: UnexpectedToken { expected: "Semicolon", found: "Ident(\"a3\")" },
    ...
    source: Some("var 1..10: x;\nconstraint x = 12a3;  % ERROR...
}
```

### 3. Error Types are `#[non_exhaustive]`

Allows safe addition of new error variants in minor versions:
```rust
#[non_exhaustive]
pub enum ErrorKind {
    // existing variants...
}
```

## Known Limitations

| Error Type | Status | Notes |
|---|---|---|
| Undefined variables | ✅ Detected | Works correctly |
| Array size mismatch | ✅ Detected | Works correctly |
| Invalid syntax | ✅ Detected | Works correctly |
| Duplicate declarations | ✅ Detected | Fixed in v0.5.1! |
| Type mismatches | ⚠️ Not detected | Type coercion allowed (limitation) |
| Type checking | ⚠️ Permissive | Int/bool/float conversions allowed |

## Error Recovery

Current behavior: **Fail-fast**

When an error is encountered:
1. Parser stops processing
2. Error is returned with full span information
3. Source code snippet is provided (if available)
4. No partial models are created

**Future:** Consider partial recovery for more resilient parsing

## Testing

### Error Test Models

Located in `tests_all/models/error_*.mzn`:

```bash
# Test error scenarios manually
./target/debug/zelen tests_all/models/error_undefined_var.mzn
./target/debug/zelen tests_all/models/error_array_size_mismatch.mzn
./target/debug/zelen tests_all/models/error_invalid_number.mzn
```

### Test Results (0.5.0)

| Test Case | Expected | Actual | Status |
|---|---|---|---|
| Undefined variable | Error | Error: "Undefined variable or parameter" | ✅ Pass |
| Array size mismatch | Error | Error: ArraySizeMismatch | ✅ Pass |
| Invalid number | Error | Error: UnexpectedToken | ✅ Pass |
| Duplicate declaration | Error | Error: DuplicateDeclaration | ✅ Pass (Fixed!) |
| Type mismatch | Should error | Allowed (limitation) | ⚠️ Limitation |

## Best Practices for Error Handling

### In Library Code

```rust
use zelen;

match zelen::parse(source) {
    Ok(ast) => {
        // Continue with translation
        match zelen::translate(&ast) {
            Ok(model) => {
                // Solve...
            }
            Err(e) => {
                eprintln!("Translation error: {}", e);
                if let Some(src) = &e.source {
                    eprintln!("Source: {}", src);
                }
            }
        }
    }
    Err(e) => {
        eprintln!("Parse error: {}", e);
        if let Some(src) = &e.source {
            eprintln!("Source:\n{}", src);
        }
    }
}
```

### In CLI Applications

The Zelen CLI automatically:
1. Formats errors with source location
2. Prints source snippets when available
3. Exits with code 1 on error
4. Provides helpful error messages

## See Also

- [API_STABILITY.md](API_STABILITY.md) - API error contract
- [CHANGELOG.md](CHANGELOG.md) - Error handling changes per version
- [Error types](src/error.rs) - Full error definitions
