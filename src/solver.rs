//! FlatZinc Solver Wrapper
//!
//! Provides a high-level Model wrapper with automatic FlatZinc output formatting.

use crate::ast::{SolveGoal, FlatZincModel};
use crate::output::{OutputFormatter, SearchType, SolveStatistics};
use crate::{tokenizer, parser, FlatZincResult, FlatZincError};
use crate::mapper::map_to_model_with_context;
use selen::prelude::*;
use selen::variables::Val;
use selen::utils::config::SolverConfig;
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};

/// Context information from FlatZinc model mapping
#[derive(Debug, Clone)]
pub struct FlatZincContext {
    pub var_names: HashMap<VarId, String>,
    pub name_to_var: HashMap<String, VarId>,
    pub arrays: HashMap<String, Vec<VarId>>,
    pub solve_goal: SolveGoal,
}

/// Solver options for configuring behavior
#[derive(Debug, Clone)]
pub struct SolverOptions {
    /// Whether to find all solutions (satisfaction problems only)
    pub find_all_solutions: bool,
    /// Maximum number of solutions to find (None = unlimited)
    pub max_solutions: Option<usize>,
    /// Whether to include statistics in output
    pub include_statistics: bool,
    /// Timeout in milliseconds (0 = no limit)
    pub timeout_ms: u64,
    /// Memory limit in megabytes (0 = no limit)
    pub memory_limit_mb: usize,
}

impl Default for SolverOptions {
    fn default() -> Self {
        Self {
            find_all_solutions: false,
            max_solutions: Some(1),
            include_statistics: true,
            timeout_ms: 0,
            memory_limit_mb: 0,
        }
    }
}

/// High-level FlatZinc solver with automatic output formatting
pub struct FlatZincSolver {
    model: Option<Model>,
    context: Option<FlatZincContext>,
    ast: Option<FlatZincModel>,
    solutions: Vec<Solution>,
    solve_time: Option<Duration>,
    options: SolverOptions,
}

impl FlatZincSolver {
    /// Create a new empty FlatZinc solver with default options
    pub fn new() -> Self {
        Self::with_options(SolverOptions::default())
    }

    /// Create a new solver with custom options
    pub fn with_options(options: SolverOptions) -> Self {
        // Create SolverConfig from options
        let mut config = SolverConfig::default();
        if options.timeout_ms == 0 {
            // 0 means no timeout limit
            config.timeout_ms = None;
        } else {
            config.timeout_ms = Some(options.timeout_ms);
        }
        if options.memory_limit_mb == 0 {
            // 0 means no memory limit
            config.max_memory_mb = None;
        } else {
            config.max_memory_mb = Some(options.memory_limit_mb as u64);
        }
        
        Self {
            model: Some(Model::with_config(config)),
            context: None,
            ast: None,
            solutions: Vec::new(),
            solve_time: None,
            options,
        }
    }

    /// Configure to find all solutions (for satisfaction problems)
    pub fn find_all_solutions(&mut self) -> &mut Self {
        self.options.find_all_solutions = true;
        self.options.max_solutions = None;
        self
    }

    /// Set maximum number of solutions to find
    pub fn max_solutions(&mut self, n: usize) -> &mut Self {
        self.options.max_solutions = Some(n);
        self.options.find_all_solutions = false;
        self
    }

    /// Configure whether to include statistics in output
    pub fn with_statistics(&mut self, enable: bool) -> &mut Self {
        self.options.include_statistics = enable;
        self
    }

    /// Set timeout in milliseconds (0 means no limit)
    pub fn with_timeout(&mut self, timeout_ms: u64) -> &mut Self {
        self.options.timeout_ms = timeout_ms;
        self
    }

    /// Set memory limit in megabytes (0 means no limit)
    pub fn with_memory_limit(&mut self, memory_limit_mb: usize) -> &mut Self {
        self.options.memory_limit_mb = memory_limit_mb;
        self
    }

    /// Load a FlatZinc problem from a string
    pub fn load_str(&mut self, fzn: &str) -> FlatZincResult<()> {
        let tokens = tokenizer::tokenize(fzn)?;
        let ast = parser::parse(tokens)?;
        
        // Recreate model with current configuration options
        let mut config = SolverConfig::default();
        if self.options.timeout_ms == 0 {
            // 0 means no timeout limit
            config.timeout_ms = None;
        } else {
            config.timeout_ms = Some(self.options.timeout_ms);
        }
        if self.options.memory_limit_mb == 0 {
            // 0 means no memory limit
            config.max_memory_mb = None;
        } else {
            config.max_memory_mb = Some(self.options.memory_limit_mb as u64);
        }
        let mut model = Model::with_config(config);
        
        self.context = Some(map_to_model_with_context(ast.clone(), &mut model)?);
        self.ast = Some(ast);
        self.model = Some(model);
        Ok(())
    }

    /// Load a FlatZinc problem from a file
    pub fn load_file(&mut self, path: &str) -> FlatZincResult<()> {
        let content = fs::read_to_string(path)
            .map_err(|e| FlatZincError::IoError(format!("Failed to read file: {}", e)))?;
        self.load_str(&content)
    }

    /// Export the loaded problem as a standalone Selen Rust program
    /// 
    /// This is useful for debugging - the generated program can be compiled
    /// and run independently to test Selen behavior directly.
    pub fn export_selen_program(&self, output_path: &str) -> FlatZincResult<()> {
        let ast = self.ast.as_ref().ok_or_else(|| FlatZincError::MapError {
            message: "No model loaded. Call load_file() or load_str() first.".to_string(),
            line: None,
            column: None,
        })?;
        
        crate::exporter::export_selen_program(ast, output_path)
    }

    /// Solve the problem (satisfaction or optimization)
    /// 
    /// For satisfaction problems:
    /// - By default, finds one solution
    /// - Use `find_all_solutions()` to find all solutions
    /// - Use `max_solutions(n)` to find up to n solutions
    /// 
    /// For optimization problems:
    /// - Finds the optimal solution
    /// - Intermediate solutions are collected if multiple solutions requested
    pub fn solve(&mut self) -> Result<(), ()> {
        let start = Instant::now();
        let model = self.model.take().expect("Model already consumed by solve()");
        let context = self.context.as_ref().expect("No context available");
        
        // Note: Timeout and memory limit are already configured when the model was created
        
        self.solutions.clear();
        
        match &context.solve_goal {
            SolveGoal::Satisfy { .. } => {
                // Satisfaction problem - use enumerate
                if self.options.find_all_solutions || self.options.max_solutions.map_or(false, |n| n > 1) {
                    // Collect multiple solutions
                    let max = self.options.max_solutions.unwrap_or(usize::MAX);
                    self.solutions = model.enumerate().take(max).collect();
                } else {
                    // Single solution
                    if let Ok(solution) = model.solve() {
                        self.solutions.push(solution);
                    }
                }
            }
            SolveGoal::Minimize { objective, .. } => {
                // Minimization problem
                let obj_var = Self::get_objective_var(objective, context)?;
                
                if self.options.find_all_solutions || self.options.max_solutions.map_or(false, |n| n > 1) {
                    // Collect intermediate solutions
                    let max = self.options.max_solutions.unwrap_or(usize::MAX);
                    self.solutions = model.minimize_and_iterate(obj_var).take(max).collect();
                } else {
                    // Just the optimal solution
                    if let Ok(solution) = model.minimize(obj_var) {
                        self.solutions.push(solution);
                    }
                }
            }
            SolveGoal::Maximize { objective, .. } => {
                // Maximization problem
                let obj_var = Self::get_objective_var(objective, context)?;
                
                if self.options.find_all_solutions || self.options.max_solutions.map_or(false, |n| n > 1) {
                    // Collect intermediate solutions
                    let max = self.options.max_solutions.unwrap_or(usize::MAX);
                    self.solutions = model.maximize_and_iterate(obj_var).take(max).collect();
                } else {
                    // Just the optimal solution
                    if let Ok(solution) = model.maximize(obj_var) {
                        self.solutions.push(solution);
                    }
                }
            }
        }
        
        self.solve_time = Some(start.elapsed());
        
        if self.solutions.is_empty() {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Get the number of solutions found
    pub fn solution_count(&self) -> usize {
        self.solutions.len()
    }

    /// Get a reference to a specific solution (0-indexed)
    pub fn get_solution(&self, index: usize) -> Option<&Solution> {
        self.solutions.get(index)
    }

    /// Extract objective variable from expression
    fn get_objective_var(expr: &crate::ast::Expr, context: &FlatZincContext) -> Result<VarId, ()> {
        use crate::ast::Expr;
        
        match expr {
            Expr::Ident(name) => {
                context.name_to_var.get(name)
                    .copied()
                    .ok_or(())
            }
            _ => Err(()) // Only support simple variable references for now
        }
    }

    /// Format the result as FlatZinc output
    /// 
    /// Outputs all solutions found, each terminated with `----------`
    /// If search completed, outputs `==========` at the end
    pub fn to_flatzinc(&self) -> String {
        if self.solutions.is_empty() {
            return self.format_unsatisfiable();
        }

        let mut output = String::new();
        
        // Output each solution
        for (i, solution) in self.solutions.iter().enumerate() {
            output.push_str(&self.format_solution(solution, i == self.solutions.len() - 1));
        }
        
        output
    }

    /// Print the FlatZinc output
    pub fn print_flatzinc(&self) {
        print!("{}", self.to_flatzinc());
    }

    fn format_solution(&self, solution: &Solution, is_last: bool) -> String {
        let context = self.context.as_ref().expect("No context loaded");
        
        // Build solution HashMap for OutputFormatter
        let solution_map: HashMap<VarId, Val> = context.var_names
            .keys()
            .map(|var_id| (*var_id, solution[*var_id]))
            .collect();
        
        // Determine search type
        let search_type = match &context.solve_goal {
            SolveGoal::Satisfy { .. } => SearchType::Satisfy,
            SolveGoal::Minimize { .. } => SearchType::Minimize,
            SolveGoal::Maximize { .. } => SearchType::Maximize,
        };
        
        // Create formatter
        let mut formatter = OutputFormatter::new(search_type);
        
        // Add statistics if enabled and available
        if self.options.include_statistics && is_last {
            // Extract statistics from Selen's Solution
            let selen_stats = &solution.stats;
            
            let stats = SolveStatistics {
                solutions: self.solutions.len(),
                nodes: selen_stats.node_count,
                failures: 0, // TODO: Selen doesn't expose failure count yet
                propagations: Some(selen_stats.propagation_count),
                solve_time: Some(selen_stats.solve_time),
                peak_memory_mb: Some(selen_stats.peak_memory_mb),
                variables: Some(selen_stats.variable_count),
                propagators: Some(selen_stats.constraint_count),
            };
            formatter = formatter.with_statistics(stats);
        }
        
        // Format solution
        let mut output = formatter.format_solution(&solution_map, &context.var_names);
        
        // Add search complete marker after last solution
        if is_last {
            output.push_str(&formatter.format_search_complete());
        }
        
        output
    }

    fn format_unsatisfiable(&self) -> String {
        let context = self.context.as_ref().expect("No context loaded");
        let search_type = match &context.solve_goal {
            SolveGoal::Satisfy { .. } => SearchType::Satisfy,
            SolveGoal::Minimize { .. } => SearchType::Minimize,
            SolveGoal::Maximize { .. } => SearchType::Maximize,
        };
        
        let mut formatter = OutputFormatter::new(search_type);
        
        // Add statistics if enabled (at least solve time for unsatisfiable problems)
        if self.options.include_statistics {
            let mut stats = SolveStatistics::default();
            stats.solutions = 0;
            stats.solve_time = self.solve_time;
            stats.peak_memory_mb = Some(1); // Placeholder since we can't get actual stats from failed solve
            // Note: We can't get nodes/failures/etc from Selen when solve fails
            // The solver consumed the model and didn't return statistics
            formatter = formatter.with_statistics(stats);
        }
        
        formatter.format_unsatisfiable()
    }
}

impl Default for FlatZincSolver {
    fn default() -> Self {
        Self::new()
    }
}
