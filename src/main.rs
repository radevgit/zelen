//! Zelen MiniZinc Solver - Direct MiniZinc to Selen CSP Solver
//!
//! This CLI tool parses MiniZinc source code directly (without FlatZinc compilation)
//! and solves it using the Selen constraint solver.

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use zelen::parse;
use zelen::translator::{Translator, ObjectiveType};

/// Zelen - Direct MiniZinc Solver backed by Selen CSP Solver
#[derive(Parser, Debug)]
#[command(
    name = "zelen",
    version = "0.4.0",
    about = "Direct MiniZinc to Selen CSP Solver",
    long_about = "Parses MiniZinc models directly and solves them using the Selen constraint solver.\n\
                  This bypasses FlatZinc compilation for supported MiniZinc features.\n\n\
                  Usage:\n  \
                    zelen model.mzn           # Solve model with no data file\n  \
                    zelen model.mzn data.dzn  # Solve model with data file"
)]
struct Args {
    /// MiniZinc model file to solve (.mzn)
    #[arg(value_name = "MODEL")]
    file: PathBuf,

    /// Optional MiniZinc data file (.dzn) containing variable assignments
    #[arg(value_name = "DATA")]
    data_file: Option<PathBuf>,

    /// Find all solutions (for satisfaction problems)
    #[arg(short = 'a', long)]
    all_solutions: bool,

    /// Stop after N solutions
    #[arg(short = 'n', long, value_name = "N")]
    num_solutions: Option<usize>,

    /// Print intermediate solutions (for optimization problems)
    #[arg(short = 'i', long)]
    intermediate: bool,

    /// Print solver statistics
    #[arg(short = 's', long)]
    statistics: bool,

    /// Verbose output (more detail)
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Time limit in milliseconds (0 = use Selen default of 60000ms)
    #[arg(short = 't', long, value_name = "MS", default_value = "0")]
    time: u64,

    /// Memory limit in MB (0 = use Selen default of 2000MB)
    #[arg(long, value_name = "MB", default_value = "0")]
    mem_limit: u64,

    /// Free search (ignore search annotations) - not yet supported
    #[arg(short = 'f', long)]
    free_search: bool,

    /// Use N parallel threads - not yet supported
    #[arg(short = 'p', long, value_name = "N")]
    parallel: Option<usize>,

    /// Random seed - not yet supported
    #[arg(short = 'r', long, value_name = "N")]
    random_seed: Option<u64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Print warnings for unsupported features
    if args.free_search {
        if args.verbose {
            eprintln!("⚠️  Warning: Free search (--free-search) is not yet supported, ignoring");
        }
    }
    if args.parallel.is_some() {
        if args.verbose {
            eprintln!("⚠️  Warning: Parallel search (--parallel) is not yet supported, ignoring");
        }
    }
    if args.random_seed.is_some() {
        if args.verbose {
            eprintln!("⚠️  Warning: Random seed (--random-seed) is not yet supported, ignoring");
        }
    }
    if args.time > 0 {
        if args.verbose {
            eprintln!("ℹ️  Note: Time limit (--time) is not yet implemented");
        }
    }
    if args.mem_limit > 0 {
        if args.verbose {
            eprintln!("ℹ️  Note: Memory limit (--mem-limit) is not yet implemented");
        }
    }

    // Read the MiniZinc source file
    if args.verbose {
        eprintln!("📖 Reading MiniZinc model file: {}", args.file.display());
    }
    let source = fs::read_to_string(&args.file).map_err(|e| {
        format!("Failed to read file '{}': {}", args.file.display(), e)
    })?;

    // Read optional data file
    let data_source = if let Some(ref data_file) = args.data_file {
        if args.verbose {
            eprintln!("📖 Reading MiniZinc data file: {}", data_file.display());
        }
        let data_content = fs::read_to_string(data_file).map_err(|e| {
            format!("Failed to read data file '{}': {}", data_file.display(), e)
        })?;
        Some(data_content)
    } else {
        None
    };

    // Combine model and data sources
    let combined_source = if let Some(data) = data_source {
        if args.verbose {
            eprintln!("📝 Merging model and data sources...");
        }
        format!("{}\n{}", source, data)
    } else {
        source
    };

    // Parse the combined MiniZinc source
    if args.verbose {
        eprintln!("🔍 Parsing MiniZinc source...");
    }
    let ast = parse(&combined_source).map_err(|e| {
        format!("Parse error: {:?}", e)
    })?;

    // Translate to Selen model
    if args.verbose {
        eprintln!("🔄 Translating to Selen model...");
    }
    let model_data = Translator::translate_with_vars(&ast).map_err(|e| {
        format!("Translation error: {:?}", e)
    })?;

    if args.verbose {
        eprintln!(
            "✅ Model created successfully with {} variables",
            model_data.int_vars.len()
                + model_data.bool_vars.len()
                + model_data.float_vars.len()
                + model_data.int_var_arrays.len()
                + model_data.bool_var_arrays.len()
                + model_data.float_var_arrays.len()
        );
    }

    // Solve the model
    if args.verbose {
        eprintln!("⚙️  Starting solver...");
    }
    let start_time = Instant::now();

    // Extract objective info before model is consumed
    let obj_type = model_data.objective_type;
    let obj_var = model_data.objective_var;
    
    let solution_result = match (obj_type, obj_var) {
        (ObjectiveType::Minimize, Some(obj_var)) => {
            if args.verbose {
                eprintln!("📉 Minimizing objective...");
            }
            model_data.model.minimize(obj_var)
        }
        (ObjectiveType::Maximize, Some(obj_var)) => {
            if args.verbose {
                eprintln!("📈 Maximizing objective...");
            }
            model_data.model.maximize(obj_var)
        }
        (ObjectiveType::Satisfy, _) => {
            if args.verbose {
                eprintln!("✓ Solving satisfaction problem...");
            }
            model_data.model.solve()
        }
        _ => model_data.model.solve(),
    };

    let elapsed = start_time.elapsed();

    match solution_result {
        Ok(solution) => {
            if args.verbose {
                eprintln!("✅ Solution found in {:?}", elapsed);
            }

            // Print solution in MiniZinc format
            print_solution(&solution, &model_data.int_vars, &model_data.bool_vars,
                          &model_data.float_vars, &model_data.int_var_arrays,
                          &model_data.bool_var_arrays, &model_data.float_var_arrays,
                          args.statistics, elapsed)?;
        }
        Err(_e) => {
            if args.verbose {
                eprintln!("❌ No solution found");
            }
            println!("=====UNSATISFIABLE=====");
            if args.statistics {
                println!("%%%mzn-stat: solveTime={:.3}", elapsed.as_secs_f64());
            }
            return Ok(());
        }
    }

    Ok(())
}

/// Print solution in MiniZinc/FlatZinc output format
fn print_solution(
    solution: &selen::prelude::Solution,
    int_vars: &std::collections::HashMap<String, selen::prelude::VarId>,
    bool_vars: &std::collections::HashMap<String, selen::prelude::VarId>,
    float_vars: &std::collections::HashMap<String, selen::prelude::VarId>,
    int_var_arrays: &std::collections::HashMap<String, Vec<selen::prelude::VarId>>,
    bool_var_arrays: &std::collections::HashMap<String, Vec<selen::prelude::VarId>>,
    float_var_arrays: &std::collections::HashMap<String, Vec<selen::prelude::VarId>>,
    print_stats: bool,
    elapsed: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    // Print integer variables
    for (name, var_id) in int_vars {
        let value = solution.get_int(*var_id);
        println!("{} = {};", name, value);
    }

    // Print boolean variables (as 0/1 in MiniZinc format)
    for (name, var_id) in bool_vars {
        let value = solution.get_int(*var_id);
        println!("{} = {};", name, value);
    }

    // Print float variables
    for (name, var_id) in float_vars {
        let value = solution.get_float(*var_id);
        println!("{} = {};", name, value);
    }

    // Print integer arrays
    for (name, var_ids) in int_var_arrays {
        print!("{} = [", name);
        for (i, var_id) in var_ids.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            let value = solution.get_int(*var_id);
            print!("{}", value);
        }
        println!("];");
    }

    // Print boolean arrays (as 0/1)
    for (name, var_ids) in bool_var_arrays {
        print!("{} = [", name);
        for (i, var_id) in var_ids.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            let value = solution.get_int(*var_id);
            print!("{}", value);
        }
        println!("];");
    }

    // Print float arrays
    for (name, var_ids) in float_var_arrays {
        print!("{} = [", name);
        for (i, var_id) in var_ids.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            let value = solution.get_float(*var_id);
            print!("{}", value);
        }
        println!("];");
    }

    // Print solution separator
    println!("----------");

    // Print statistics if requested
    if print_stats {
        println!(
            "%%%mzn-stat: solveTime={:.3}",
            elapsed.as_secs_f64()
        );
    }

    Ok(())
}
