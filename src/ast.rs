//! Abstract Syntax Tree for MiniZinc Core Subset
//!
//! Represents the structure of a parsed MiniZinc model.

use std::fmt;

/// A complete MiniZinc model
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub items: Vec<Item>,
}

/// Top-level items in a MiniZinc model
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// Enum definition: `enum Color = {Red, Green, Blue};`
    EnumDef(EnumDef),
    /// Variable or parameter declaration: `int: n = 5;`
    VarDecl(VarDecl),
    /// Constraint: `constraint x < y;`
    Constraint(Constraint),
    /// Solve item: `solve satisfy;` or `solve minimize x;`
    Solve(Solve),
    /// Output item: `output ["x = ", show(x)];`
    Output(Output),
}

/// Enumerated type definition
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub values: Vec<String>,
    pub span: Span,
}

/// Variable or parameter declaration
#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub type_inst: TypeInst,
    pub name: String,
    pub expr: Option<Expr>,
    pub span: Span,
}

/// Type-inst (type + instantiation)
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInst {
    /// Basic type: bool, int, float
    Basic {
        is_var: bool,
        base_type: BaseType,
    },
    /// Constrained type: var 1..10, var {1,3,5}
    Constrained {
        is_var: bool,
        base_type: BaseType,
        domain: Expr,
    },
    /// Array type: array[1..n] of var int or array[1..n, 1..m] of var int
    /// For multi-dimensional arrays: index_sets contains one entry per dimension
    Array {
        index_sets: Vec<Expr>,
        element_type: Box<TypeInst>,
    },
}

/// Base types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseType {
    Bool,
    Int,
    Float,
    /// Enumerated type (stored as integer domain internally)
    Enum(String),
}

/// Constraint item
#[derive(Debug, Clone, PartialEq)]
pub struct Constraint {
    pub expr: Expr,
    pub span: Span,
}

/// Search options for solve items
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchOption {
    /// Complete search (find all solutions)
    Complete,
    /// Incomplete search (may not find all solutions)
    Incomplete,
}

/// Solve item
#[derive(Debug, Clone, PartialEq)]
pub enum Solve {
    Satisfy { search_option: Option<SearchOption>, span: Span },
    Minimize { expr: Expr, search_option: Option<SearchOption>, span: Span },
    Maximize { expr: Expr, search_option: Option<SearchOption>, span: Span },
}

/// Output item
#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    pub expr: Expr,
    pub span: Span,
}

/// Expressions
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// Identifier: `x`, `queens`
    Ident(String),
    
    /// Boolean literal: `true`, `false`
    BoolLit(bool),
    
    /// Integer literal: `42`, `0`, `-5`
    IntLit(i64),
    
    /// Float literal: `3.14`, `1.0`
    FloatLit(f64),
    
    /// String literal: `"hello"`
    StringLit(String),
    
    /// Array literal: `[1, 2, 3]`
    ArrayLit(Vec<Expr>),
    
    /// Set literal: `{1, 2, 3}`
    SetLit(Vec<Expr>),
    
    /// Range: `1..n`, `0..10`
    Range(Box<Expr>, Box<Expr>),
    
    /// Array access: `x[i]`, `grid[i+1]`, `cube[i,j,k]`
    /// For multi-dimensional arrays, indices contains one entry per dimension
    ArrayAccess {
        array: Box<Expr>,
        indices: Vec<Expr>,
    },
    
    /// Binary operation: `x + y`, `a /\ b`
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    
    /// Unary operation: `-x`, `not b`
    UnOp {
        op: UnOp,
        expr: Box<Expr>,
    },
    
    /// Function/predicate call: `sum(x)`, `alldifferent(queens)`
    Call {
        name: String,
        args: Vec<Expr>,
    },
    
    /// If-then-else: `if x > 0 then 1 else -1 endif`
    IfThenElse {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Option<Box<Expr>>,
    },
    
    /// Array comprehension: `[i*2 | i in 1..n]`
    ArrayComp {
        expr: Box<Expr>,
        generators: Vec<Generator>,
    },
    
    /// Generator call: `forall(i in 1..n)(x[i] > 0)`
    GenCall {
        name: String,
        generators: Vec<Generator>,
        body: Box<Expr>,
    },
    
    /// Array2D initializer: `array2d(row_range, col_range, [values...])`
    /// Wraps a flat array into a 2D structure
    Array2D {
        row_range: Box<Expr>,
        col_range: Box<Expr>,
        values: Box<Expr>,  // Should be an ArrayLit
    },
    
    /// Array3D initializer: `array3d(r1_range, r2_range, r3_range, [values...])`
    /// Wraps a flat array into a 3D structure
    Array3D {
        r1_range: Box<Expr>,
        r2_range: Box<Expr>,
        r3_range: Box<Expr>,
        values: Box<Expr>,  // Should be an ArrayLit
    },
    
    /// Implicit index set for arrays: `int` in `array[int]`
    ImplicitIndexSet(BaseType),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // div
    Mod,      // mod
    FDiv,     // / (float division)
    
    // Comparison
    Lt,       // <
    Le,       // <=
    Gt,       // >
    Ge,       // >=
    Eq,       // == or =
    Ne,       // !=
    
    // Logical
    And,      // /\
    Or,       // \/
    Impl,     // ->
    Iff,      // <->
    Xor,      // xor
    
    // Set
    In,       // in
    Range,    // ..
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,      // -
    Not,      // not
}

/// Generator in comprehension: `i in 1..n where i > 0`
#[derive(Debug, Clone, PartialEq)]
pub struct Generator {
    pub names: Vec<String>,
    pub expr: Expr,
    pub where_clause: Option<Expr>,
}

/// Source location span for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
    
    pub fn dummy() -> Self {
        Self { start: 0, end: 0 }
    }
}

// Display implementations for better error messages

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "div",
            BinOp::Mod => "mod",
            BinOp::FDiv => "/",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::And => "/\\",
            BinOp::Or => "\\/",
            BinOp::Impl => "->",
            BinOp::Iff => "<->",
            BinOp::Xor => "xor",
            BinOp::In => "in",
            BinOp::Range => "..",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnOp::Neg => "-",
            UnOp::Not => "not",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BaseType::Bool => "bool".to_string(),
            BaseType::Int => "int".to_string(),
            BaseType::Float => "float".to_string(),
            BaseType::Enum(name) => name.clone(),
        };
        write!(f, "{}", s)
    }
}
