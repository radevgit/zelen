# FlatZinc Documentation

- Flattening: https://docs.minizinc.dev/en/stable/flattening.html
- FlatZinc Specification: https://docs.minizinc.dev/en/stable/fzn-spec.html

Examples: https://www.hakank.org/minizinc/



## 4. Implementation Plan: Steps for FlatZinc Import

### Step 1: Scaffolding and Setup
- module files (`mod.rs`, `parser.rs`, `ast.rs`, `error.rs`).
- Define `ZincImportError` error type.
- Add public API stubs for `import_flatzinc_file` and `import_flatzinc_str`.

### Step 2: Tokenizer/Lexer
- Implement a simple tokenizer for FlatZinc syntax:
	- Recognize identifiers, numbers, strings, symbols, and comments.
	- Output a stream of tokens for the parser.

### Step 3: Parser
- Implement a recursive-descent parser for FlatZinc:
	- Parse variable declarations, constraints, parameters, and solve annotations.
	- Build an AST or intermediate representation.
	- Handle arrays and basic types (int, bool, set).

### Step 4: Mapping to Selen Model
- Map FlatZinc AST to Selen's `Model`:
	- Create variables and arrays.
	- Post constraints (support core FlatZinc constraints first).
	- Set objective (satisfy, minimize, maximize).

### Step 5: Error Handling
- Implement robust error handling and reporting for parsing and mapping errors.
- Return `ZincImportError` with descriptive messages.

### Step 6: Testing and Examples
- Add unit tests for tokenizer, parser, and mapping.
- Add integration tests with sample `.fzn` files.
- Provide usage examples in `/examples`.

### Step 7: Extensibility and Documentation
- Document supported FlatZinc features and limitations.
- Design for future support of MiniZinc or `.dzn` data files.
## 3. Import API Design and Integration

### Directory for Implementation
- All FlatZinc parser and import code should be placed in `/src/zinc`.

### API Design

- Provide a public API for importing FlatZinc files into a Selen model.
- Example API signature:

```rust
// In /src/zinc/mod.rs
pub fn import_flatzinc_file<P: AsRef<std::path::Path>>(path: P) -> Result<Model, ZincImportError>;

// Or, for string input:
pub fn import_flatzinc_str(input: &str) -> Result<Model, ZincImportError>;
```

- `Model` is the main Selen model type.
- `ZincImportError` is a custom error type for parsing/mapping errors.

### Integration Points

- The import function should:
	- Parse the FlatZinc file/string into an internal AST or intermediate representation.
	- Map FlatZinc variables, constraints, and solve annotations to Selen's model API.
	- Return a fully constructed `Model` ready for solving.
- Add tests and usage examples in `/tests` and `/examples`.

### Extensibility
- The API should be designed so that future support for MiniZinc or `.dzn` data files can be added with minimal changes (e.g., via additional functions or feature flags).
## 2a. Survey of Existing Parsers and Design Notes

- **Rust ecosystem:** No mature Rust crates exist for FlatZinc or MiniZinc parsing as of 2025. Most solvers in other languages (C++, Python, Java) implement their own parser or use the official C++ library (libminizinc).
- **Reference implementations:**
	- [MiniZinc/libminizinc](https://github.com/MiniZinc/libminizinc): Official C++ library with FlatZinc parser (useful for grammar/structure reference).
	- [Chuffed](https://github.com/chuffed/chuffed): C++ solver with FlatZinc parser.
	- [Google OR-Tools](https://github.com/google/or-tools): C++ FlatZinc parser.
- **Design takeaways:**
	- FlatZinc is line-oriented and regular, making it feasible to hand-write a parser in Rust.
	- The official FlatZinc BNF grammar is a good starting point for tokenizer and parser design.
	- Most solvers use a simple recursive-descent parser or state machine for FlatZinc.
- **No external dependencies:** All parsing and lexing will be implemented manually in Rust, using only the standard library.

---

## 2b. Crate Organization: Standalone vs Integrated Parser

**Option 1: Separate Crate**
- Pros:
	- Parser can be reused in other projects or solvers.
	- Clear separation of concerns; easier to test and document parser independently.
	- Encourages clean API boundaries.
- Cons:
	- Slightly more maintenance overhead (versioning, publishing, documentation).
	- May be overkill if parser is tightly coupled to Selen's internal model.

**Option 2: Integrated in Selen Crate**
- Pros:
	- Simpler project structure; no need for cross-crate dependencies.
	- Easier access to Selen's internal types and APIs.
	- Faster iteration for project-specific needs.
- Cons:
	- Harder to reuse parser in other projects.
	- Parser code may become entangled with solver logic.

**Recommendation:**
- If you anticipate reusing the FlatZinc parser in other Rust projects or want to encourage community adoption, a separate crate is preferable.
- If the parser will be tightly integrated with Selen's internal model and not reused elsewhere, keep it as a module within this crate for simplicity.
# MiniZinc Import: Detailed Implementation Plan

## 1. Scope and Requirements

- **Goal:** Enable parsing and importing of MiniZinc (.mzn) model files (and optionally .dzn data files) into the Selen CSP solver, mapping them to internal model structures.
- **Directory:** Implementation is scoped to `docs/development/` (for planning/design) and the relevant Rust source directory for code.
- **Constraints:** No external dependencies (no crates for parsing, lexing, or MiniZinc).

---


## 2. MiniZinc and FlatZinc Standards and References

- **MiniZinc Language Reference:**  
	- [MiniZinc Language Reference](https://docs.minizinc.dev/en/stable/)
	- [MiniZinc Grammar (BNF)](https://github.com/MiniZinc/libminizinc/blob/master/doc/grammar/minizinc.bnf)
- **FlatZinc Specification:**  
	- [FlatZinc Specification](https://docs.minizinc.dev/en/stable/fzn-spec.html)
- **File Types:**  
	- `.mzn` — Model files (constraints, variables, parameters)
	- `.dzn` — Data files (parameter assignments)
- **Key Language Features:**  
	- Variable declarations (int, bool, set, array)
	- Constraints (global, arithmetic, logical)
	- Parameters and data separation
	- Solve annotations (satisfy, minimize, maximize)
	- Comments (`% ...`)
- **Subset Recommendation:**  
	- Start with a subset: integer/boolean variables, basic constraints, arrays, and parameter assignment.

---

## 3. Implementation Complexity

- **Parsing:**  
	- Must hand-write a recursive-descent parser or a simple tokenizer and parser for the MiniZinc subset.
	- Handle comments, whitespace, identifiers, literals, arrays, and basic expressions.
- **Mapping:**  
	- Map MiniZinc constructs to Selen’s internal model (variables, constraints, objectives).
- **Error Handling:**  
	- Provide clear error messages for unsupported or malformed input.
- **Extensibility:**  
	- Design parser to allow future support for more MiniZinc features.

**Estimated Complexity:**  
- **Minimal Subset:** Moderate (basic parser, mapping, error handling)
- **Full MiniZinc:** High (complex grammar, global constraints, advanced types)

---

## 4. Implementation Plan

### Step 1: Research and Design

- Review MiniZinc language reference and grammar.
- Identify the minimal viable subset to support (variables, constraints, arrays, basic arithmetic).
- Document mapping from MiniZinc constructs to Selen’s API.

### Step 2: Write a MiniZinc Tokenizer

- Implement a tokenizer for MiniZinc syntax:
	- Recognize keywords, identifiers, numbers, symbols, comments, and whitespace.
	- Output a stream of tokens for the parser.

### Step 3: Implement a Recursive-Descent Parser

- Parse MiniZinc model files into an AST (abstract syntax tree).
- Support:
	- Variable declarations (int, bool, array)
	- Parameter assignments
	- Constraint statements
	- Solve annotations (optional, for future)
- Ignore unsupported features with clear errors.

### Step 4: Map AST to Selen Model

- Translate parsed MiniZinc AST into Selen’s internal model:
	- Create variables, post constraints, set objectives.
- Handle arrays and parameter substitution.

### Step 5: Integrate and Test

- Add import API (e.g., `Model::import_minizinc(path: &str) -> Result<Model, Error>`).
- Write unit tests with sample MiniZinc files.
- Document supported features and limitations.

---


## 5. References and Resources

- [MiniZinc Language Reference](https://docs.minizinc.dev/en/stable/)
- [MiniZinc BNF Grammar](https://github.com/MiniZinc/libminizinc/blob/master/doc/grammar/minizinc.bnf)
- [FlatZinc Specification](https://docs.minizinc.dev/en/stable/fzn-spec.html)
- [MiniZinc Example Models](https://www.minizinc.org/software/)
- [MiniZinc Standard Library](https://docs.minizinc.dev/en/stable/lib-globals.html)

---

## 6. No-Dependency Considerations

- All parsing and lexing must be implemented manually in Rust.
- Avoid using crates like `nom`, `pest`, or `lalrpop`.
- Use Rust’s standard library only.

---

## 7. Example: Minimal Supported MiniZinc

```minizinc
int: n;
array[1..n] of var 1..n: x;
constraint all_different(x);
solve satisfy;
```

---

## 8. Future Extensions

- Support for `.dzn` data files.
- More global constraints.
- Objective functions (minimize/maximize).
- Full MiniZinc grammar coverage.

Integrated project: https://github.com/glklimmer/oxiflex