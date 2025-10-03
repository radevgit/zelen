//! FlatZinc Output Formatter
//!
//! Formats solution results according to FlatZinc output specification.
//! See: https://docs.minizinc.dev/en/stable/fzn-spec.html#output
//!
//! The FlatZinc output format is:
//! 1. For each solution:
//!    - Variable assignments: `varname = value;`
//!    - Solution separator: `----------`
//! 2. After all solutions (or when search completes):
//!    - Search complete: `==========`
//! 3. If no solution:
//!    - `=====UNSATISFIABLE=====`
//! 4. If unknown:
//!    - `=====UNKNOWN=====`
//! 5. Optional statistics (as comments):
//!    - `%%%mzn-stat: statistic=value`

use selen::variables::{Val, VarId};
use std::collections::HashMap;
use std::time::Duration;

/// Represents the type of search being performed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    /// Satisfaction problem (find any solution)
    Satisfy,
    /// Optimization problem (minimize)
    Minimize,
    /// Optimization problem (maximize)
    Maximize,
}

/// Statistics about the solving process
/// 
/// Aligns with FlatZinc specification standard statistics (Section 4.3.3.2):
/// - solutions, nodes, failures, solveTime, peakMem (standard)
/// - propagations, variables, propagators, intVariables, etc. (extended)
#[derive(Debug, Clone, Default)]
pub struct SolveStatistics {
    /// Number of solutions found
    pub solutions: usize,
    /// Number of search nodes (choice points/decisions made)
    pub nodes: usize,
    /// Number of failures (backtracks)
    pub failures: usize,
    /// Number of propagation steps performed
    pub propagations: Option<usize>,
    /// Solving time
    pub solve_time: Option<Duration>,
    /// Peak memory usage in megabytes
    pub peak_memory_mb: Option<usize>,
    /// Number of variables in the problem
    pub variables: Option<usize>,
    /// Number of constraints/propagators in the problem
    pub propagators: Option<usize>,
}

/// FlatZinc output formatter
pub struct OutputFormatter {
    /// Type of search
    #[allow(dead_code)] // Reserved for future use (e.g., validation)
    search_type: SearchType,
    /// Whether to include statistics
    include_stats: bool,
    /// Collected statistics
    stats: SolveStatistics,
}

impl OutputFormatter {
    /// Create a new output formatter
    pub fn new(search_type: SearchType) -> Self {
        Self {
            search_type,
            include_stats: false,
            stats: SolveStatistics::default(),
        }
    }

    /// Enable statistics output
    pub fn with_statistics(mut self, stats: SolveStatistics) -> Self {
        self.include_stats = true;
        self.stats = stats;
        self
    }

    /// Format a single solution
    ///
    /// For satisfaction problems, this should be called once.
    /// For optimization problems, this may be called multiple times as better solutions are found.
    pub fn format_solution(
        &self,
        solution: &HashMap<VarId, Val>,
        var_names: &HashMap<VarId, String>,
    ) -> String {
        let mut output = String::new();
        
        // Sort variables by name for consistent output
        let mut sorted_vars: Vec<_> = var_names.iter().collect();
        sorted_vars.sort_by_key(|(_, name)| name.as_str());
        
        // Output variable assignments
        for (var_id, name) in sorted_vars {
            if let Some(val) = solution.get(var_id) {
                let value_str = match val {
                    Val::ValI(i) => i.to_string(),
                    Val::ValF(f) => f.to_string(),
                };
                output.push_str(&format!("{} = {};\n", name, value_str));
            }
        }
        
        // Solution separator
        output.push_str("----------\n");
        
        output
    }

    /// Format array output (for array variables)
    pub fn format_array(
        &self,
        name: &str,
        index_range: (i32, i32),
        values: &[Val],
    ) -> String {
        let mut output = String::new();
        
        let values_str: Vec<String> = values.iter().map(|v| match v {
            Val::ValI(i) => i.to_string(),
            Val::ValF(f) => f.to_string(),
        }).collect();
        
        output.push_str(&format!(
            "{} = array1d({}..{}, [{}]);\n",
            name,
            index_range.0,
            index_range.1,
            values_str.join(", ")
        ));
        
        output
    }

    /// Format 2D array output
    pub fn format_array_2d(
        &self,
        name: &str,
        index1_range: (i32, i32),
        index2_range: (i32, i32),
        values: &[Val],
    ) -> String {
        let mut output = String::new();
        
        let values_str: Vec<String> = values.iter().map(|v| match v {
            Val::ValI(i) => i.to_string(),
            Val::ValF(f) => f.to_string(),
        }).collect();
        
        output.push_str(&format!(
            "{} = array2d({}..{}, {}..{}, [{}]);\n",
            name,
            index1_range.0,
            index1_range.1,
            index2_range.0,
            index2_range.1,
            values_str.join(", ")
        ));
        
        output
    }

    /// Format the search complete indicator
    ///
    /// This should be output after all solutions have been found,
    /// or when the search space has been exhausted.
    pub fn format_search_complete(&self) -> String {
        let mut output = String::new();
        
        // Search complete separator
        output.push_str("==========\n");
        
        // Add statistics if enabled
        if self.include_stats {
            output.push_str(&self.format_statistics());
        }
        
        output
    }

    /// Format statistics as comments
    /// 
    /// Follows FlatZinc specification format (Section 4.3.3.2):
    /// %%%mzn-stat: name=value
    /// %%%mzn-stat-end
    fn format_statistics(&self) -> String {
        let mut output = String::new();
        
        // Standard FlatZinc statistics
        output.push_str(&format!("%%%mzn-stat: solutions={}\n", self.stats.solutions));
        output.push_str(&format!("%%%mzn-stat: nodes={}\n", self.stats.nodes));
        output.push_str(&format!("%%%mzn-stat: failures={}\n", self.stats.failures));
        
        // Extended statistics (if available)
        if let Some(propagations) = self.stats.propagations {
            output.push_str(&format!("%%%mzn-stat: propagations={}\n", propagations));
        }
        
        if let Some(variables) = self.stats.variables {
            output.push_str(&format!("%%%mzn-stat: variables={}\n", variables));
        }
        
        if let Some(propagators) = self.stats.propagators {
            output.push_str(&format!("%%%mzn-stat: propagators={}\n", propagators));
        }
        
        if let Some(time) = self.stats.solve_time {
            output.push_str(&format!("%%%mzn-stat: solveTime={:.3}\n", time.as_secs_f64()));
        }
        
        if let Some(mb) = self.stats.peak_memory_mb {
            output.push_str(&format!("%%%mzn-stat: peakMem={:.2}\n", mb as f64));
        }
        
        output.push_str("%%%mzn-stat-end\n");
        
        output
    }

    /// Format "no solution found" message
    pub fn format_unsatisfiable(&self) -> String {
        let mut output = String::new();
        output.push_str("=====UNSATISFIABLE=====\n");
        
        if self.include_stats {
            output.push_str(&self.format_statistics());
        }
        
        output
    }

    /// Format "unknown" status (solver couldn't determine satisfiability)
    pub fn format_unknown(&self) -> String {
        let mut output = String::new();
        output.push_str("=====UNKNOWN=====\n");
        
        if self.include_stats {
            output.push_str(&self.format_statistics());
        }
        
        output
    }

    /// Format "unbounded" status (for optimization problems)
    pub fn format_unbounded(&self) -> String {
        let mut output = String::new();
        output.push_str("=====UNBOUNDED=====\n");
        
        if self.include_stats {
            output.push_str(&self.format_statistics());
        }
        
        output
    }
}

// Convenience functions for backward compatibility

/// Format a solution in FlatZinc output format (simple version)
pub fn format_solution(
    solution: &HashMap<VarId, Val>,
    var_names: &HashMap<VarId, String>,
) -> String {
    let formatter = OutputFormatter::new(SearchType::Satisfy);
    formatter.format_solution(solution, var_names)
}

/// Format "no solution" message
pub fn format_no_solution() -> String {
    let formatter = OutputFormatter::new(SearchType::Satisfy);
    formatter.format_unsatisfiable()
}

/// Format "unknown" message (solver could not determine satisfiability)
pub fn format_unknown() -> String {
    let formatter = OutputFormatter::new(SearchType::Satisfy);
    formatter.format_unknown()
}
