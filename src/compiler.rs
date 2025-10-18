//! Compiler for MiniZinc Core Subset to Rust/Selen
//!
//! Translates a parsed MiniZinc AST into executable Rust code using the Selen constraint solver API.

use crate::ast::*;
use crate::error::{Error, Result};
use std::collections::HashMap;

/// Context for tracking variables and their types during compilation
#[derive(Debug)]
struct CompilerContext {
    /// Map from MiniZinc variable names to Rust variable info
    variables: HashMap<String, VarInfo>,
    /// Counter for generating unique temporary variable names
    temp_counter: usize,
}

/// Information about a compiled variable
#[derive(Debug, Clone)]
struct VarInfo {
    /// The Rust variable name (may differ from MiniZinc name if sanitized)
    rust_name: String,
    /// Whether this is a decision variable (var) or parameter (par)
    is_var: bool,
    /// Type information
    var_type: VarType,
}

#[derive(Debug, Clone)]
enum VarType {
    Bool,
    Int,
    Float,
    IntArray,
    BoolArray,
}

impl CompilerContext {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            temp_counter: 0,
        }
    }

    fn add_variable(&mut self, mzn_name: String, info: VarInfo) {
        self.variables.insert(mzn_name, info);
    }

    fn get_variable(&self, name: &str) -> Option<&VarInfo> {
        self.variables.get(name)
    }

    fn gen_temp(&mut self) -> String {
        let name = format!("_temp{}", self.temp_counter);
        self.temp_counter += 1;
        name
    }
}

/// Main compiler struct
pub struct Compiler {
    context: CompilerContext,
    /// Generated Rust code
    output: String,
    /// Indentation level
    indent: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            context: CompilerContext::new(),
            output: String::new(),
            indent: 0,
        }
    }

    /// Compile a MiniZinc model to Rust code
    pub fn compile(&mut self, model: &Model) -> Result<String> {
        // Generate preamble
        self.emit_preamble();

        // Process all items
        for item in &model.items {
            self.compile_item(item)?;
        }

        // Generate epilogue
        self.emit_epilogue();

        Ok(self.output.clone())
    }

    fn emit_preamble(&mut self) {
        self.emit_line("use zelen::*;");
        self.emit_line("");
        self.emit_line("fn main() {");
        self.indent += 1;
        self.emit_line("let mut model = Model::new();");
        self.emit_line("");
    }

    fn emit_epilogue(&mut self) {
        self.emit_line("");
        self.emit_line("// Solve the model");
        self.emit_line("let solver = model.solve();");
        self.emit_line("match solver.next_solution() {");
        self.indent += 1;
        self.emit_line("Some(solution) => {");
        self.indent += 1;
        self.emit_line("println!(\"Solution found:\");");
        self.emit_line("// TODO: Print solution values");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("None => {");
        self.indent += 1;
        self.emit_line("println!(\"No solution found\");");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
    }

    fn compile_item(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::VarDecl(var_decl) => self.compile_var_decl(var_decl),
            Item::Constraint(constraint) => self.compile_constraint(constraint),
            Item::Solve(solve) => self.compile_solve(solve),
            Item::Output(_output) => {
                // Skip output items for now
                Ok(())
            }
        }
    }

    fn compile_var_decl(&mut self, var_decl: &VarDecl) -> Result<()> {
        let rust_name = sanitize_identifier(&var_decl.name);

        match &var_decl.type_inst {
            TypeInst::Basic { is_var, base_type } => {
                if *is_var {
                    return Err(Error::unsupported_feature(
                        "Decision variables without domains",
                        "Phase 1",
                        var_decl.span,
                    ));
                }

                // Parameter declaration
                if let Some(expr) = &var_decl.expr {
                    let value = self.compile_expr(expr)?;
                    self.emit_line(&format!("let {} = {};", rust_name, value));
                    
                    let var_type = match base_type {
                        BaseType::Bool => VarType::Bool,
                        BaseType::Int => VarType::Int,
                        BaseType::Float => VarType::Float,
                    };
                    
                    self.context.add_variable(
                        var_decl.name.clone(),
                        VarInfo {
                            rust_name,
                            is_var: false,
                            var_type,
                        },
                    );
                } else {
                    return Err(Error::type_error(
                        "parameter with initializer",
                        "parameter without initializer",
                        var_decl.span,
                    ));
                }
            }

            TypeInst::Constrained { is_var, base_type, domain } => {
                if !is_var {
                    return Err(Error::unsupported_feature(
                        "Constrained parameters",
                        "Phase 1",
                        var_decl.span,
                    ));
                }

                // Decision variable with domain
                let domain_code = self.compile_domain(domain, base_type)?;
                self.emit_line(&format!(
                    "let {} = model.new_int_var({});",
                    rust_name, domain_code
                ));

                self.context.add_variable(
                    var_decl.name.clone(),
                    VarInfo {
                        rust_name,
                        is_var: true,
                        var_type: VarType::Int,
                    },
                );
            }

            TypeInst::Array { index_sets, element_type } => {
                self.compile_array_decl(&var_decl.name, index_sets, element_type, &var_decl.expr)?;
            }
        }

        Ok(())
    }

    fn compile_array_decl(
        &mut self,
        name: &str,
        index_sets: &[Expr],
        element_type: &TypeInst,
        init_expr: &Option<Expr>,
    ) -> Result<()> {
        let rust_name = sanitize_identifier(name);

        // Determine if it's a var array or par array
        let is_var = match element_type {
            TypeInst::Basic { is_var, .. } => *is_var,
            TypeInst::Constrained { is_var, .. } => *is_var,
            TypeInst::Array { .. } => {
                return Err(Error::unsupported_feature(
                    "Multi-dimensional arrays",
                    "Phase 2",
                    Span::dummy(),
                ));
            }
        };

        if is_var {
            // Decision variable array
            let domain_code = match element_type {
                TypeInst::Constrained { base_type, domain, .. } => {
                    self.compile_domain(domain, base_type)?
                }
                TypeInst::Basic { base_type, .. } => {
                    match base_type {
                        BaseType::Int => "i32::MIN..=i32::MAX".to_string(),
                        BaseType::Bool => return Err(Error::unsupported_feature(
                            "Bool arrays without domain",
                            "Phase 1",
                            Span::dummy(),
                        )),
                        BaseType::Float => return Err(Error::unsupported_feature(
                            "Float decision variables",
                            "Phase 1",
                            Span::dummy(),
                        )),
                    }
                }
                _ => unreachable!(),
            };

            // For multi-dimensional arrays, compute total size as product of all dimensions
            let mut size_code = String::new();
            for (i, index_set) in index_sets.iter().enumerate() {
                let dim_size = self.compile_index_set_size(index_set)?;
                if i == 0 {
                    size_code = dim_size;
                } else {
                    size_code = format!("({}) * ({})", size_code, dim_size);
                }
            }
            
            self.emit_line(&format!(
                "let {} = model.new_int_var_array({}, {});",
                rust_name, size_code, domain_code
            ));

            self.context.add_variable(
                name.to_string(),
                VarInfo {
                    rust_name,
                    is_var: true,
                    var_type: VarType::IntArray,
                },
            );
        } else {
            // Parameter array
            if let Some(expr) = init_expr {
                let value = self.compile_expr(expr)?;
                self.emit_line(&format!("let {} = {};", rust_name, value));

                self.context.add_variable(
                    name.to_string(),
                    VarInfo {
                        rust_name,
                        is_var: false,
                        var_type: VarType::IntArray,
                    },
                );
            } else {
                return Err(Error::type_error(
                    "parameter array with initializer",
                    "parameter array without initializer",
                    Span::dummy(),
                ));
            }
        }

        Ok(())
    }

    fn compile_domain(&mut self, domain: &Expr, _base_type: &BaseType) -> Result<String> {
        match &domain.kind {
            ExprKind::BinOp { op: BinOp::Range, left, right } => {
                // Handle range as binary operation: start..end
                let start_code = self.compile_expr(left)?;
                let end_code = self.compile_expr(right)?;
                Ok(format!("{}..={}", start_code, end_code))
            }
            ExprKind::SetLit(elements) => {
                let values: Result<Vec<String>> = elements
                    .iter()
                    .map(|e| self.compile_expr(e))
                    .collect();
                Ok(format!("&[{}]", values?.join(", ")))
            }
            _ => Err(Error::type_error(
                "range or set literal",
                "other expression",
                domain.span,
            )),
        }
    }

    fn compile_index_set_size(&mut self, index_set: &Expr) -> Result<String> {
        match &index_set.kind {
            ExprKind::Range(start, end) => {
                let start_code = self.compile_expr(start)?;
                let end_code = self.compile_expr(end)?;
                Ok(format!("({} - {} + 1) as usize", end_code, start_code))
            }
            ExprKind::ImplicitIndexSet(_) => {
                Err(Error::unsupported_feature(
                    "Implicit index sets",
                    "Phase 1",
                    index_set.span,
                ))
            }
            _ => Err(Error::type_error(
                "range expression",
                "other expression",
                index_set.span,
            )),
        }
    }

    fn compile_constraint(&mut self, constraint: &Constraint) -> Result<()> {
        let constraint_code = self.compile_constraint_expr(&constraint.expr)?;
        self.emit_line(&constraint_code);
        Ok(())
    }

    fn compile_constraint_expr(&mut self, expr: &Expr) -> Result<String> {
        match &expr.kind {
            ExprKind::Call { name, args } => {
                self.compile_constraint_call(name, args)
            }
            ExprKind::BinOp { op, left, right } => {
                self.compile_constraint_binop(*op, left, right)
            }
            _ => Err(Error::type_error(
                "constraint expression",
                "other expression",
                expr.span,
            )),
        }
    }

    fn compile_constraint_call(&mut self, name: &str, args: &[Expr]) -> Result<String> {
        match name {
            "alldifferent" => {
                if args.len() != 1 {
                    return Err(Error::type_error(
                        "1 argument",
                        &format!("{} arguments", args.len()),
                        Span::dummy(),
                    ));
                }
                let arr = self.compile_expr(&args[0])?;
                Ok(format!("model.all_different(&{});", arr))
            }
            _ => Err(Error::unsupported_feature(
                &format!("Constraint '{}'", name),
                "Phase 1",
                Span::dummy(),
            )),
        }
    }

    fn compile_constraint_binop(&mut self, op: BinOp, left: &Expr, right: &Expr) -> Result<String> {
        let left_code = self.compile_expr(left)?;
        let right_code = self.compile_expr(right)?;

        let method = match op {
            BinOp::Lt => "less_than",
            BinOp::Le => "less_or_equal",
            BinOp::Gt => "greater_than",
            BinOp::Ge => "greater_or_equal",
            BinOp::Eq => "equals",
            BinOp::Ne => "not_equals",
            _ => {
                return Err(Error::unsupported_feature(
                    &format!("Binary operator {:?} in constraints", op),
                    "Phase 1",
                    Span::dummy(),
                ));
            }
        };

        Ok(format!("model.{}({}, {});", method, left_code, right_code))
    }

    fn compile_solve(&mut self, solve: &Solve) -> Result<()> {
        self.emit_line("");
        self.emit_line("// Solve configuration");
        match solve {
            Solve::Satisfy { .. } => {
                self.emit_line("// solve satisfy (default)");
            }
            Solve::Minimize { expr, .. } => {
                let obj = self.compile_expr(expr)?;
                self.emit_line(&format!("model.minimize({});", obj));
            }
            Solve::Maximize { expr, .. } => {
                let obj = self.compile_expr(expr)?;
                self.emit_line(&format!("model.maximize({});", obj));
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<String> {
        match &expr.kind {
            ExprKind::Ident(name) => {
                if let Some(var_info) = self.context.get_variable(name) {
                    Ok(var_info.rust_name.clone())
                } else {
                    Err(Error::message(
                        &format!("Undefined variable: {}", name),
                        expr.span,
                    ))
                }
            }
            ExprKind::IntLit(i) => Ok(i.to_string()),
            ExprKind::BoolLit(b) => Ok(b.to_string()),
            ExprKind::FloatLit(f) => Ok(f.to_string()),
            ExprKind::StringLit(s) => Ok(format!("\"{}\"", s)),
            ExprKind::ArrayLit(elements) => {
                let values: Result<Vec<String>> = elements
                    .iter()
                    .map(|e| self.compile_expr(e))
                    .collect();
                Ok(format!("vec![{}]", values?.join(", ")))
            }
            ExprKind::BinOp { op, left, right } => {
                let left_code = self.compile_expr(left)?;
                let right_code = self.compile_expr(right)?;
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div | BinOp::FDiv => "/",
                    BinOp::Mod => "%",
                    BinOp::Lt => "<",
                    BinOp::Le => "<=",
                    BinOp::Gt => ">",
                    BinOp::Ge => ">=",
                    BinOp::Eq => "==",
                    BinOp::Ne => "!=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                    _ => {
                        return Err(Error::unsupported_feature(
                            &format!("Binary operator {:?}", op),
                            "Phase 1",
                            expr.span,
                        ));
                    }
                };
                Ok(format!("({} {} {})", left_code, op_str, right_code))
            }
            ExprKind::UnOp { op, expr: inner } => {
                let inner_code = self.compile_expr(inner)?;
                let op_str = match op {
                    UnOp::Neg => "-",
                    UnOp::Not => "!",
                };
                Ok(format!("({}{})", op_str, inner_code))
            }
            ExprKind::ArrayAccess { array, indices } => {
                let array_code = self.compile_expr(array)?;
                
                // For now, handle 1D arrays only  in code generation mode
                // Multi-dimensional will be handled separately in translator
                if indices.len() != 1 {
                    return Err(Error::unsupported_feature(
                        "Multi-dimensional array access in code generation",
                        "Phase 2",
                        Span::dummy(),
                    ));
                }
                
                let index_code = self.compile_expr(&indices[0])?;
                Ok(format!("{}[{} as usize - 1]", array_code, index_code))
            }
            ExprKind::Call { name, args } => {
                let args_code: Result<Vec<String>> = args
                    .iter()
                    .map(|e| self.compile_expr(e))
                    .collect();
                Ok(format!("{}({})", name, args_code?.join(", ")))
            }
            ExprKind::Range(start, end) => {
                let start_code = self.compile_expr(start)?;
                let end_code = self.compile_expr(end)?;
                Ok(format!("{}..={}", start_code, end_code))
            }
            _ => Err(Error::unsupported_feature(
                &format!("Expression type: {:?}", expr.kind),
                "Phase 1",
                expr.span,
            )),
        }
    }

    fn emit_line(&mut self, line: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(line);
        self.output.push('\n');
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Sanitize a MiniZinc identifier to be a valid Rust identifier
fn sanitize_identifier(name: &str) -> String {
    // For now, just return as-is
    // TODO: Handle reserved keywords, special characters, etc.
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_compile_simple_param() {
        let source = "int: n = 5;";
        let model = parse(source).unwrap();
        
        let mut compiler = Compiler::new();
        let rust_code = compiler.compile(&model).unwrap();
        
        assert!(rust_code.contains("let n = 5;"));
    }

    #[test]
    fn test_compile_var_with_domain() {
        let source = "var 1..10: x;";
        let model = parse(source).unwrap();
        
        let mut compiler = Compiler::new();
        let rust_code = compiler.compile(&model).unwrap();
        
        assert!(rust_code.contains("new_int_var"));
        assert!(rust_code.contains("1..=10"));
    }
}
