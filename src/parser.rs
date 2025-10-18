//! Parser for MiniZinc Core Subset
//!
//! Implements a recursive descent parser that builds an AST from tokens.

use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::{Lexer, Token, TokenKind};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    source: String,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token().unwrap_or_else(|_| Token {
            kind: TokenKind::Eof,
            span: Span::dummy(),
        });
        
        Self {
            lexer,
            current_token,
            source: String::new(),
        }
    }
    
    pub fn with_source(mut self, source: String) -> Self {
        self.source = source;
        self
    }
    
    /// Add source context to an error
    fn add_source_to_error(&self, error: Error) -> Error {
        if !self.source.is_empty() {
            error.with_source(self.source.clone())
        } else {
            error
        }
    }
    
    /// Parse a complete MiniZinc model
    pub fn parse_model(&mut self) -> Result<Model> {
        let mut items = Vec::new();
        
        while self.current_token.kind != TokenKind::Eof {
            items.push(self.parse_item()?);
        }
        
        Ok(Model { items })
    }
    
    /// Parse a single item
    fn parse_item(&mut self) -> Result<Item> {
        match &self.current_token.kind {
            TokenKind::Constraint => self.parse_constraint(),
            TokenKind::Solve => self.parse_solve(),
            TokenKind::Output => self.parse_output(),
            _ => self.parse_var_decl(),
        }
    }
    
    /// Parse variable declaration: `int: n = 5;` or `array[1..n] of var int: x;`
    fn parse_var_decl(&mut self) -> Result<Item> {
        let start = self.current_token.span.start;
        let type_inst = self.parse_type_inst()?;
        
        self.expect(TokenKind::Colon)?;
        
        let name = self.expect_ident()?;
        
        let expr = if self.current_token.kind == TokenKind::Eq {
            self.advance()?;
            Some(self.parse_expr()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Semicolon)?;
        
        let end = self.current_token.span.end;
        
        Ok(Item::VarDecl(VarDecl {
            type_inst,
            name,
            expr,
            span: Span::new(start, end),
        }))
    }
    
    /// Parse type-inst: `var int`, `array[1..n] of var 1..10`, etc.
    fn parse_type_inst(&mut self) -> Result<TypeInst> {
        // Check for array type
        if self.current_token.kind == TokenKind::Array {
            return self.parse_array_type_inst();
        }
        
        // Parse var/par
        let is_var = match &self.current_token.kind {
            TokenKind::Var => {
                self.advance()?;
                true
            }
            TokenKind::Par => {
                self.advance()?;
                false
            }
            _ => false, // Default to par
        };
        
        // Parse base type or domain
        match &self.current_token.kind {
            TokenKind::Bool => {
                self.advance()?;
                Ok(TypeInst::Basic { is_var, base_type: BaseType::Bool })
            }
            TokenKind::Int => {
                self.advance()?;
                // Check if followed by range or set (constrained type)
                // This is a lookahead - we'll handle it in the next iteration if needed
                Ok(TypeInst::Basic { is_var, base_type: BaseType::Int })
            }
            TokenKind::Float => {
                self.advance()?;
                Ok(TypeInst::Basic { is_var, base_type: BaseType::Float })
            }
            TokenKind::IntLit(_) | TokenKind::FloatLit(_) | TokenKind::LBrace => {
                // Constrained type: 1..10 or 0.0..1.0 or {1,3,5}
                let domain = self.parse_range_or_set_expr()?;
                
                // Infer base type from domain
                let base_type = match &domain.kind {
                    ExprKind::BinOp { op: BinOp::Range, left, .. } => {
                        match &left.kind {
                            ExprKind::FloatLit(_) => BaseType::Float,
                            _ => BaseType::Int,
                        }
                    }
                    ExprKind::Range(left, _) => {
                        match &left.kind {
                            ExprKind::FloatLit(_) => BaseType::Float,
                            _ => BaseType::Int,
                        }
                    }
                    ExprKind::SetLit(_) => BaseType::Int,
                    _ => BaseType::Int,
                };
                
                Ok(TypeInst::Constrained {
                    is_var,
                    base_type,
                    domain,
                })
            }
            _ => {
                Err(self.add_source_to_error(Error::unexpected_token(
                    "type (bool, int, float, or constrained type)",
                    &format!("{:?}", self.current_token.kind),
                    self.current_token.span,
                )))
            }
        }
    }
    
    /// Parse a range or set expression for type constraints
    fn parse_range_or_set_expr(&mut self) -> Result<Expr> {
        if self.current_token.kind == TokenKind::LBrace {
            self.parse_set_literal()
        } else {
            // Parse as expression (will handle ranges)
            self.parse_expr()
        }
    }
    
    /// Parse array type: `array[1..n] of var int`, `array[1..n, 1..m] of var int`, etc.
    fn parse_array_type_inst(&mut self) -> Result<TypeInst> {
        self.expect(TokenKind::Array)?;
        self.expect(TokenKind::LBracket)?;
        
        // Parse one or more index sets (comma-separated for multi-dimensional)
        let mut index_sets = vec![];
        
        loop {
            // Handle implicit index sets: array[int], array[bool], array[float]
            let index_set = match &self.current_token.kind {
                TokenKind::Int => {
                    let span = self.current_token.span;
                    self.advance()?;
                    Expr {
                        kind: ExprKind::ImplicitIndexSet(BaseType::Int),
                        span,
                    }
                }
                TokenKind::Bool => {
                    let span = self.current_token.span;
                    self.advance()?;
                    Expr {
                        kind: ExprKind::ImplicitIndexSet(BaseType::Bool),
                        span,
                    }
                }
                TokenKind::Float => {
                    let span = self.current_token.span;
                    self.advance()?;
                    Expr {
                        kind: ExprKind::ImplicitIndexSet(BaseType::Float),
                        span,
                    }
                }
                _ => {
                    // Regular index set expression: array[1..n] or array[1..n, 1..m]
                    self.parse_expr()?
                }
            };
            
            index_sets.push(index_set);
            
            // Check for comma (multi-dimensional) or bracket (end)
            if self.current_token.kind == TokenKind::Comma {
                self.advance()?;
                continue;
            } else {
                break;
            }
        }
        
        self.expect(TokenKind::RBracket)?;
        self.expect(TokenKind::Of)?;
        
        let element_type = Box::new(self.parse_type_inst()?);
        
        Ok(TypeInst::Array {
            index_sets,
            element_type,
        })
    }
    
    /// Parse constraint: `constraint x < y;`
    fn parse_constraint(&mut self) -> Result<Item> {
        let start = self.current_token.span.start;
        self.expect(TokenKind::Constraint)?;
        
        let expr = self.parse_expr()?;
        
        self.expect(TokenKind::Semicolon)?;
        
        let end = self.current_token.span.end;
        
        Ok(Item::Constraint(Constraint {
            expr,
            span: Span::new(start, end),
        }))
    }
    
    /// Parse solve item: `solve satisfy;` or `solve minimize cost;`
    fn parse_solve(&mut self) -> Result<Item> {
        let start = self.current_token.span.start;
        self.expect(TokenKind::Solve)?;
        
        // Check for search annotation: :: int_search(...) or :: complete/incomplete
        let search_option = if self.current_token.kind == TokenKind::ColonColon {
            self.advance()?;
            Some(self.parse_search_annotation()?)
        } else {
            None
        };
        
        let solve = match &self.current_token.kind {
            TokenKind::Satisfy => {
                self.advance()?;
                Solve::Satisfy {
                    search_option,
                    span: Span::new(start, self.current_token.span.end),
                }
            }
            TokenKind::Minimize => {
                self.advance()?;
                let expr = self.parse_expr()?;
                Solve::Minimize {
                    expr,
                    search_option,
                    span: Span::new(start, self.current_token.span.end),
                }
            }
            TokenKind::Maximize => {
                self.advance()?;
                let expr = self.parse_expr()?;
                Solve::Maximize {
                    expr,
                    search_option,
                    span: Span::new(start, self.current_token.span.end),
                }
            }
            _ => {
                return Err(self.add_source_to_error(Error::unexpected_token(
                    "satisfy, minimize, or maximize",
                    &format!("{:?}", self.current_token.kind),
                    self.current_token.span,
                )));
            }
        };
        
        self.expect(TokenKind::Semicolon)?;
        
        Ok(Item::Solve(solve))
    }
    
    /// Parse search annotation: int_search(...) and extract complete/incomplete option
    fn parse_search_annotation(&mut self) -> Result<SearchOption> {
        // We parse the annotation but only extract complete/incomplete
        // We ignore variable selection and value selection strategies
        
        // Expected format: int_search(variables, var_select, val_select, complete/incomplete)
        // or just: complete or incomplete
        
        if let TokenKind::Ident(name) = &self.current_token.kind {
            let name_str = name.clone();
            
            if name_str == "complete" {
                self.advance()?;
                return Ok(SearchOption::Complete);
            } else if name_str == "incomplete" {
                self.advance()?;
                return Ok(SearchOption::Incomplete);
            } else if name_str == "int_search" || name_str == "bool_search" || name_str == "float_search" {
                // Parse function call: int_search(args...)
                self.advance()?;
                self.expect(TokenKind::LParen)?;
                
                // Parse arguments: skip first 3 (variables, var_select, val_select)
                let mut paren_depth = 1;
                let mut arg_count = 0;
                
                while paren_depth > 0 && self.current_token.kind != TokenKind::Eof {
                    match &self.current_token.kind {
                        TokenKind::LParen => paren_depth += 1,
                        TokenKind::RParen => paren_depth -= 1,
                        TokenKind::Comma if paren_depth == 1 => arg_count += 1,
                        _ => {}
                    }
                    
                    // Check the 4th argument (index 3) for complete/incomplete
                    if paren_depth == 1 && arg_count == 3 {
                        if let TokenKind::Ident(opt) = &self.current_token.kind {
                            let opt_str = opt.clone();
                            if opt_str == "complete" {
                                // Consume the rest of the annotation
                                while self.current_token.kind != TokenKind::RParen && 
                                      self.current_token.kind != TokenKind::Eof {
                                    self.advance()?;
                                }
                                if self.current_token.kind == TokenKind::RParen {
                                    self.advance()?;
                                }
                                return Ok(SearchOption::Complete);
                            } else if opt_str == "incomplete" {
                                // Consume the rest of the annotation
                                while self.current_token.kind != TokenKind::RParen && 
                                      self.current_token.kind != TokenKind::Eof {
                                    self.advance()?;
                                }
                                if self.current_token.kind == TokenKind::RParen {
                                    self.advance()?;
                                }
                                return Ok(SearchOption::Incomplete);
                            }
                        }
                    }
                    
                    self.advance()?;
                }
                
                // Default to complete if not specified
                return Ok(SearchOption::Complete);
            }
        }
        
        // If it's not recognized, skip to next valid token and default to complete
        while self.current_token.kind != TokenKind::Satisfy && 
              self.current_token.kind != TokenKind::Minimize &&
              self.current_token.kind != TokenKind::Maximize &&
              self.current_token.kind != TokenKind::Eof {
            self.advance()?;
        }
        
        Ok(SearchOption::Complete)
    }
    
    /// Parse output item: `output ["x = ", show(x)];`
    fn parse_output(&mut self) -> Result<Item> {
        let start = self.current_token.span.start;
        self.expect(TokenKind::Output)?;
        
        let expr = self.parse_expr()?;
        
        self.expect(TokenKind::Semicolon)?;
        
        let end = self.current_token.span.end;
        
        Ok(Item::Output(Output {
            expr,
            span: Span::new(start, end),
        }))
    }
    
    /// Parse expression with precedence climbing
    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_expr_bp(0)
    }
    
    /// Parse expression with binding power (precedence)
    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr> {
        let start = self.current_token.span.start;
        
        // Parse left-hand side (prefix operators or primary)
        let mut lhs = self.parse_prefix_expr()?;
        
        // Parse binary operators
        loop {
            let op = match self.current_token.kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::FDiv,
                TokenKind::Div => BinOp::Div,
                TokenKind::Mod => BinOp::Mod,
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Ge => BinOp::Ge,
                TokenKind::Eq => BinOp::Eq,
                TokenKind::Ne => BinOp::Ne,
                TokenKind::And => BinOp::And,
                TokenKind::Or => BinOp::Or,
                TokenKind::Impl => BinOp::Impl,
                TokenKind::Iff => BinOp::Iff,
                TokenKind::Xor => BinOp::Xor,
                TokenKind::In => BinOp::In,
                TokenKind::DotDot => BinOp::Range,
                _ => break,
            };
            
            let (l_bp, r_bp) = self.binding_power(op);
            
            if l_bp < min_bp {
                break;
            }
            
            self.advance()?;
            let rhs = self.parse_expr_bp(r_bp)?;
            
            let end = rhs.span.end;
            lhs = Expr {
                kind: ExprKind::BinOp {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                },
                span: Span::new(start, end),
            };
        }
        
        Ok(lhs)
    }
    
    /// Get binding power (precedence) for binary operators
    fn binding_power(&self, op: BinOp) -> (u8, u8) {
        match op {
            BinOp::Iff => (2, 1),
            BinOp::Impl => (4, 3),
            BinOp::Or => (6, 5),
            BinOp::Xor => (6, 5),
            BinOp::And => (8, 7),
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Eq | BinOp::Ne => (10, 9),
            BinOp::In => (10, 9),
            BinOp::Range => (12, 11),
            BinOp::Add | BinOp::Sub => (14, 13),
            BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::FDiv => (16, 15),
        }
    }
    
    /// Parse prefix expression (unary operators)
    fn parse_prefix_expr(&mut self) -> Result<Expr> {
        let start = self.current_token.span.start;
        
        match &self.current_token.kind {
            TokenKind::Minus => {
                self.advance()?;
                let expr = self.parse_prefix_expr()?;
                let end = expr.span.end;
                Ok(Expr {
                    kind: ExprKind::UnOp {
                        op: UnOp::Neg,
                        expr: Box::new(expr),
                    },
                    span: Span::new(start, end),
                })
            }
            TokenKind::Not => {
                self.advance()?;
                let expr = self.parse_prefix_expr()?;
                let end = expr.span.end;
                Ok(Expr {
                    kind: ExprKind::UnOp {
                        op: UnOp::Not,
                        expr: Box::new(expr),
                    },
                    span: Span::new(start, end),
                })
            }
            _ => self.parse_postfix_expr(),
        }
    }
    
    /// Parse postfix expression (array access, function calls)
    fn parse_postfix_expr(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary_expr()?;
        
        loop {
            match &self.current_token.kind {
                TokenKind::LBracket => {
                    // Array access (possibly multi-dimensional: grid[i,j] or grid[i,j,k])
                    self.advance()?;
                    let mut indices = vec![];
                    
                    loop {
                        indices.push(self.parse_expr()?);
                        if self.current_token.kind == TokenKind::Comma {
                            self.advance()?;
                            continue;
                        } else {
                            break;
                        }
                    }
                    
                    self.expect(TokenKind::RBracket)?;
                    
                    let end = self.current_token.span.end;
                    expr = Expr {
                        span: Span::new(expr.span.start, end),
                        kind: ExprKind::ArrayAccess {
                            array: Box::new(expr),
                            indices,
                        },
                    };
                }
                TokenKind::LParen => {
                    // Function call (only if expr is an identifier)
                    if let ExprKind::Ident(name) = &expr.kind {
                        let name = name.clone();
                        self.advance()?;
                        
                        // Special handling for array2d and array3d
                        if name == "array2d" {
                            let row_range = self.parse_expr()?;
                            self.expect(TokenKind::Comma)?;
                            let col_range = self.parse_expr()?;
                            self.expect(TokenKind::Comma)?;
                            let values = self.parse_expr()?;
                            self.expect(TokenKind::RParen)?;
                            
                            let end = self.current_token.span.end;
                            expr = Expr {
                                span: Span::new(expr.span.start, end),
                                kind: ExprKind::Array2D {
                                    row_range: Box::new(row_range),
                                    col_range: Box::new(col_range),
                                    values: Box::new(values),
                                },
                            };
                        } else if name == "array3d" {
                            let r1_range = self.parse_expr()?;
                            self.expect(TokenKind::Comma)?;
                            let r2_range = self.parse_expr()?;
                            self.expect(TokenKind::Comma)?;
                            let r3_range = self.parse_expr()?;
                            self.expect(TokenKind::Comma)?;
                            let values = self.parse_expr()?;
                            self.expect(TokenKind::RParen)?;
                            
                            let end = self.current_token.span.end;
                            expr = Expr {
                                span: Span::new(expr.span.start, end),
                                kind: ExprKind::Array3D {
                                    r1_range: Box::new(r1_range),
                                    r2_range: Box::new(r2_range),
                                    r3_range: Box::new(r3_range),
                                    values: Box::new(values),
                                },
                            };
                        } else {
                            // Regular function call
                            let mut args = Vec::new();
                            if self.current_token.kind != TokenKind::RParen {
                                loop {
                                    // Check for generator call: forall(i in 1..n)(expr)
                                    if self.is_generator_start() {
                                        let generators = self.parse_generators()?;
                                        self.expect(TokenKind::RParen)?;
                                        self.expect(TokenKind::LParen)?;
                                        let body = self.parse_expr()?;
                                        self.expect(TokenKind::RParen)?;
                                        
                                        let end = self.current_token.span.end;
                                        return Ok(Expr {
                                            span: Span::new(expr.span.start, end),
                                            kind: ExprKind::GenCall {
                                                name,
                                                generators,
                                                body: Box::new(body),
                                            },
                                        });
                                    }
                                    
                                    args.push(self.parse_expr()?);
                                    if self.current_token.kind != TokenKind::Comma {
                                        break;
                                    }
                                    self.advance()?;
                                }
                            }
                            
                            self.expect(TokenKind::RParen)?;
                            
                            let end = self.current_token.span.end;
                            expr = Expr {
                                span: Span::new(expr.span.start, end),
                                kind: ExprKind::Call { name, args },
                            };
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    /// Parse primary expression (literals, identifiers, parentheses, arrays, etc.)
    fn parse_primary_expr(&mut self) -> Result<Expr> {
        let start = self.current_token.span.start;
        
        let kind = match self.current_token.kind.clone() {
            TokenKind::BoolLit(b) => {
                self.advance()?;
                ExprKind::BoolLit(b)
            }
            TokenKind::IntLit(i) => {
                self.advance()?;
                ExprKind::IntLit(i)
            }
            TokenKind::FloatLit(f) => {
                self.advance()?;
                ExprKind::FloatLit(f)
            }
            TokenKind::StringLit(s) => {
                self.advance()?;
                ExprKind::StringLit(s)
            }
            TokenKind::Ident(name) => {
                self.advance()?;
                ExprKind::Ident(name)
            }
            TokenKind::LParen => {
                self.advance()?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                return Ok(expr);
            }
            TokenKind::LBracket => {
                return self.parse_array_literal_or_comp();
            }
            TokenKind::LBrace => {
                return self.parse_set_literal();
            }
            _ => {
                return Err(self.add_source_to_error(Error::unexpected_token(
                    "expression",
                    &format!("{:?}", self.current_token.kind),
                    self.current_token.span,
                )));
            }
        };
        
        let end = self.current_token.span.end;
        Ok(Expr {
            kind,
            span: Span::new(start, end),
        })
    }
    
    /// Parse array literal or comprehension: `[1,2,3]` or `[i*2 | i in 1..n]`
    fn parse_array_literal_or_comp(&mut self) -> Result<Expr> {
        let start = self.current_token.span.start;
        self.expect(TokenKind::LBracket)?;
        
        if self.current_token.kind == TokenKind::RBracket {
            // Empty array
            self.advance()?;
            return Ok(Expr {
                kind: ExprKind::ArrayLit(Vec::new()),
                span: Span::new(start, self.current_token.span.end),
            });
        }
        
        let first_expr = self.parse_expr()?;
        
        // Check for comprehension
        if self.current_token.kind == TokenKind::Pipe {
            self.advance()?;
            let generators = self.parse_generators()?;
            self.expect(TokenKind::RBracket)?;
            
            let end = self.current_token.span.end;
            return Ok(Expr {
                kind: ExprKind::ArrayComp {
                    expr: Box::new(first_expr),
                    generators,
                },
                span: Span::new(start, end),
            });
        }
        
        // Regular array literal
        let mut elements = vec![first_expr];
        while self.current_token.kind == TokenKind::Comma {
            self.advance()?;
            if self.current_token.kind == TokenKind::RBracket {
                break;
            }
            elements.push(self.parse_expr()?);
        }
        
        self.expect(TokenKind::RBracket)?;
        
        let end = self.current_token.span.end;
        Ok(Expr {
            kind: ExprKind::ArrayLit(elements),
            span: Span::new(start, end),
        })
    }
    
    /// Parse set literal: `{1, 2, 3}`
    fn parse_set_literal(&mut self) -> Result<Expr> {
        let start = self.current_token.span.start;
        self.expect(TokenKind::LBrace)?;
        
        let mut elements = Vec::new();
        if self.current_token.kind != TokenKind::RBrace {
            loop {
                elements.push(self.parse_expr()?);
                if self.current_token.kind != TokenKind::Comma {
                    break;
                }
                self.advance()?;
            }
        }
        
        self.expect(TokenKind::RBrace)?;
        
        let end = self.current_token.span.end;
        Ok(Expr {
            kind: ExprKind::SetLit(elements),
            span: Span::new(start, end),
        })
    }
    
    /// Check if current position starts a generator
    /// We need to peek ahead to see if there's an 'in' keyword
    fn is_generator_start(&mut self) -> bool {
        if !matches!(self.current_token.kind, TokenKind::Ident(_)) {
            return false;
        }
        
        // Try to peek ahead to find 'in' keyword
        // Simple heuristic: if we see ident followed by 'in', it's a generator
        let mut peek_lexer = self.lexer.clone();
        let mut depth = 0;
        
        loop {
            match peek_lexer.next_token() {
                Ok(token) => match token.kind {
                    TokenKind::In if depth == 0 => return true,
                    TokenKind::LParen => depth += 1,
                    TokenKind::RParen if depth > 0 => depth -= 1,
                    TokenKind::RParen | TokenKind::Comma if depth == 0 => return false,
                    TokenKind::Eof => return false,
                    _ => {}
                }
                Err(_) => return false,
            }
        }
    }
    
    /// Parse generators: `i in 1..n where i > 0, j in 1..m`
    fn parse_generators(&mut self) -> Result<Vec<Generator>> {
        let mut generators = Vec::new();
        
        loop {
            let mut names = vec![self.expect_ident()?];
            
            while self.current_token.kind == TokenKind::Comma {
                self.advance()?;
                if self.current_token.kind == TokenKind::In {
                    break;
                }
                names.push(self.expect_ident()?);
            }
            
            self.expect(TokenKind::In)?;
            let expr = self.parse_expr()?;
            
            let where_clause = if self.current_token.kind == TokenKind::Where {
                self.advance()?;
                Some(self.parse_expr()?)
            } else {
                None
            };
            
            generators.push(Generator {
                names,
                expr,
                where_clause,
            });
            
            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.advance()?;
        }
        
        Ok(generators)
    }
    
    // Helper methods
    
    fn advance(&mut self) -> Result<()> {
        self.current_token = self.lexer.next_token().map_err(|e| {
            self.add_source_to_error(e)
        })?;
        Ok(())
    }
    
    fn expect(&mut self, expected: TokenKind) -> Result<()> {
        if std::mem::discriminant(&self.current_token.kind) == std::mem::discriminant(&expected) {
            self.advance()?;
            Ok(())
        } else {
            // For better error reporting, point to the location where we expected the token
            // If it's at the beginning of a new line, point to the end of previous line
            let error_span = if self.current_token.span.start > 0 {
                // Look back one character - this often points to end of previous token
                Span::new(self.current_token.span.start.saturating_sub(1), self.current_token.span.start)
            } else {
                self.current_token.span
            };
            
            Err(self.add_source_to_error(Error::unexpected_token(
                &format!("{:?}", expected),
                &format!("{:?}", self.current_token.kind),
                error_span,
            )))
        }
    }
    
    fn expect_ident(&mut self) -> Result<String> {
        if let TokenKind::Ident(name) = &self.current_token.kind {
            let name = name.clone();
            self.advance()?;
            Ok(name)
        } else {
            Err(self.add_source_to_error(Error::unexpected_token(
                "identifier",
                &format!("{:?}", self.current_token.kind),
                self.current_token.span,
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Result<Model> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).with_source(source.to_string());
        parser.parse_model()
    }

    #[test]
    fn test_simple_var_decl() {
        let model = parse("int: n = 5;").unwrap();
        assert_eq!(model.items.len(), 1);
    }

    #[test]
    fn test_array_decl() {
        let model = parse("array[1..n] of var int: x;").unwrap();
        assert_eq!(model.items.len(), 1);
    }

    #[test]
    fn test_constraint() {
        let model = parse("constraint x < y;").unwrap();
        assert_eq!(model.items.len(), 1);
    }

    #[test]
    fn test_solve_satisfy() {
        let model = parse("solve satisfy;").unwrap();
        assert_eq!(model.items.len(), 1);
    }

    #[test]
    fn test_nqueens_simple() {
        let source = r#"
            int: n = 4;
            array[1..n] of var 1..n: queens;
            constraint alldifferent(queens);
            solve satisfy;
        "#;
        let model = parse(source).unwrap();
        assert_eq!(model.items.len(), 4);
    }

    #[test]
    fn test_expressions() {
        let source = r#"
            constraint x + y > 10;
            constraint a /\ b \/ c;
            constraint sum(arr) <= 100;
        "#;
        let model = parse(source).unwrap();
        assert_eq!(model.items.len(), 3);
    }

    #[test]
    fn test_implicit_index_array() {
        // Test array[int] syntax (implicitly-indexed arrays)
        let source = r#"
            array[int] of int: evens = [2, 4, 6, 8];
        "#;
        let model = parse(source).unwrap();
        assert_eq!(model.items.len(), 1);
        
        // Verify it's an array with implicit index set
        if let Item::VarDecl(var_decl) = &model.items[0] {
            if let TypeInst::Array { index_sets, .. } = &var_decl.type_inst {
                assert_eq!(index_sets.len(), 1);
                assert!(matches!(index_sets[0].kind, ExprKind::ImplicitIndexSet(BaseType::Int)));
            } else {
                panic!("Expected array type");
            }
        } else {
            panic!("Expected var decl");
        }
    }
}
