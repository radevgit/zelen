# API Stability and Compatibility

This document describes Zelen's API stability guarantees for external library users.

## Stability Commitment

Zelen follows [Semantic Versioning](https://semver.org/) (SemVer):
- **MAJOR.MINOR.PATCH** (e.g., 0.5.0)
- **MAJOR**: Breaking API changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, no new features

## Current Version: 0.5.x (Beta)

**Status:** API is stabilizing but may have breaking changes before v1.0

While we strive for backward compatibility, version 0.x reserves the right to make breaking changes. However, we commit to:
- Documenting all breaking changes in CHANGELOG.md
- Providing upgrade guides when APIs change
- Minimizing breaking changes between releases
- Giving advance notice for major restructuring

## Public API Surface

### Stable APIs (v0.5+)

These APIs are committed to remain backward compatible through at least v1.0:

#### Top-Level Functions
```rust
pub fn parse(source: &str) -> Result<ast::Model>
pub fn translate(ast: &ast::Model) -> Result<selen::prelude::Model>
pub fn build_model(source: &str) -> Result<selen::prelude::Model>
pub fn build_model_with_config(source: &str, config: SolverConfig) -> Result<selen::prelude::Model>
pub fn solve(source: &str) -> Result<Result<selen::core::Solution, selen::core::SolverError>>
pub fn solve_with_config(source: &str, config: SolverConfig) -> Result<Vec<selen::core::Solution>>
```

**Guarantee:** These function signatures will not change in v0.x or v1.0.

#### Configuration Types
```rust
pub struct SolverConfig {
    pub time_limit_ms: Option<u64>,
    pub memory_limit_mb: Option<u64>,
    pub all_solutions: bool,
    pub max_solutions: Option<usize>,
}
```

**Guarantee:** Existing fields will not be removed. New fields may be added with default values.

#### Main Translator Interface
```rust
impl Translator {
    pub fn translate(ast: &Model) -> Result<Model>
    pub fn translate_with_vars(ast: &Model) -> Result<TranslatedModel>
    pub fn translate_with_config(ast: &Model, config: selen::utils::config::SolverConfig) -> Result<Model>
}
```

**Guarantee:** These methods will not be removed in v0.x.

### Semi-Stable APIs (May Change)

These structures may be extended with new fields in minor versions, but existing fields are stable:

#### TranslatedModel

```rust
pub struct TranslatedModel {
    pub model: selen::model::Model,
    pub int_vars: HashMap<String, VarId>,
    pub int_var_arrays: HashMap<String, Vec<VarId>>,
    pub bool_vars: HashMap<String, VarId>,
    pub bool_var_arrays: HashMap<String, Vec<VarId>>,
    pub float_vars: HashMap<String, VarId>,
    pub float_var_arrays: HashMap<String, Vec<VarId>>,
    pub objective_type: ObjectiveType,
    pub objective_var: Option<VarId>,
    pub output_items: Vec<ast::Expr>,
    pub search_option: Option<ast::SearchOption>,
    pub enum_vars: HashMap<String, (String, Vec<String>)>,
}
```

**Guarantee:** 
- Existing fields will not be removed or change type
- New fields may be added (always backward compatible)
- To allow safe extension, `TranslatedModel` is marked `#[non_exhaustive]` (as of v0.5+)

**Migration Note:** If you directly pattern-match on `TranslatedModel`, you must use `..` for forward compatibility:
```rust
// ✅ CORRECT - Will work with new fields
let TranslatedModel { model, int_vars, .. } = translated;

// ❌ INCORRECT - Will break if fields are added
let TranslatedModel { model, int_vars } = translated;
```

#### Error Types

```rust
pub enum ErrorKind {
    // Lexer errors
    UnexpectedChar(char),
    UnterminatedString,
    InvalidNumber(String),
    
    // Parser errors
    UnexpectedToken { expected: String, found: String },
    UnexpectedEof,
    InvalidExpression(String),
    InvalidTypeInst(String),
    
    // Semantic errors
    UnsupportedFeature { feature: String, phase: String, workaround: Option<String> },
    TypeError { expected: String, found: String },
    DuplicateDeclaration(String),
    UndefinedVariable(String),
    
    // Array-related errors
    ArraySizeMismatch { declared: usize, provided: usize },
    Array2DSizeMismatch { /* ... */ },
    Array3DSizeMismatch { /* ... */ },
    // ... etc
    
    Message(String),
}
```

**Guarantee:**
- Existing error variants will not be removed
- Existing fields on error variants will not change type
- New error variants may be added (marked `#[non_exhaustive]` as of v0.5+)
- New fields may be added to existing variants (backward compatible)

**Migration Note:** Match on `Error` with a catch-all pattern:
```rust
// ✅ CORRECT - Will work with new error types
match parse(source) {
    Ok(ast) => { /* ... */ },
    Err(e) => match &e.kind {
        ErrorKind::UndefinedVariable(name) => { /* ... */ },
        _ => { /* Handle other errors */ },
    }
}

// ❌ INCORRECT - Will fail if new error types are added
match parse(source) {
    Ok(ast) => { /* ... */ },
    Err(e) => match &e.kind {
        ErrorKind::UndefinedVariable(name) => { /* ... */ },
        ErrorKind::UnsupportedFeature { .. } => { /* ... */ },
        // ERROR: non-exhaustive patterns!
    }
}
```

### Unstable Internal APIs (Subject to Change)

The following are **not part of the public API contract** and may change without notice:

- Internal modules marked `pub(crate)`
- Implementation details not re-exported at crate root
- Private functions and types in public modules
- Exact structure of AST types (intended for direct use via parser only)

If you need access to unstable APIs, please open an issue - we may stabilize them!

## Breaking Changes Between Versions

### Changes from 0.4 → 0.5
- Added `enum_vars` field to `TranslatedModel` (backward compatible due to `#[non_exhaustive]`)
- Added `with_max_solutions()` to `SolverConfig` (backward compatible - uses default())

### Planned for 1.0 (if any)
- None planned. The API is designed to be stable through v1.0+

## Forward Compatibility Guidelines

### For Library Users

**If you want to be future-proof:**

1. **Never assume struct fields** - Always use constructors or accessors
   ```rust
   // ✅ Safe
   let config = SolverConfig::default()
       .with_time_limit_ms(5000);
   
   // ❌ Risky - will break if fields are added
   let config = SolverConfig {
       time_limit_ms: Some(5000),
       memory_limit_mb: None,
       all_solutions: false,
       max_solutions: None,
   };
   ```

2. **Use catch-all patterns in matches**
   ```rust
   // ✅ Safe
   match parse(source) {
       Ok(ast) => { /* ... */ },
       Err(e) => { /* Handle error */ },
   }
   
   // ✅ Also safe
   match &error.kind {
       ErrorKind::UndefinedVariable(name) => { /* ... */ },
       other => { /* Handle rest */ },
   }
   ```

3. **Document your MSRV (Minimum Supported Rust Version)**
   - Zelen requires Rust 1.88+
   - Bumping MSRV requires a MINOR version bump

4. **Pin major version in Cargo.toml**
   ```toml
   # Safe - auto-updates to compatible versions
   zelen = "0.5"
   
   # Explicitly handle potential breaking changes at 1.0
   zelen = ">=0.5,<1.0"  # for pre-1.0
   zelen = "1"           # for 1.0+
   ```

### For Library Maintainers (Us)

**We commit to:**
1. Documenting all breaking changes in CHANGELOG.md with "**BREAKING**" prefix
2. Providing migration guides for breaking changes
3. Using `#[non_exhaustive]` on enums and structs that may grow
4. Not breaking public APIs without a MAJOR version bump
5. Deprecating APIs for at least one minor version before removal
6. Giving at least 2 weeks notice before breaking changes in pre-release versions

## Questions?

For API stability concerns or requests to stabilize internal APIs, please:
1. Open an issue on GitHub
2. Describe your use case
3. We'll evaluate for stabilization

## See Also

- [Semantic Versioning](https://semver.org/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [CHANGELOG.md](CHANGELOG.md) - Detailed version history
