use clap::Parser;
use zelen::FlatZincSolver;
use std::process;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "zelen")]
#[command(about = "FlatZinc solver using Selen CSP solver")]
#[command(version)]
struct Args {
    #[arg(value_name = "FILE")]
    input: PathBuf,

    #[arg(short = 'a', long = "all-solutions")]
    all_solutions: bool,

    #[arg(short = 'n', long = "num-solutions", value_name = "N")]
    num_solutions: Option<usize>,

    #[arg(short = 'i', long = "intermediate")]
    intermediate: bool,

    #[arg(short = 'f', long = "free-search")]
    free_search: bool,

    #[arg(short = 's', long = "statistics")]
    statistics: bool,

    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    #[arg(short = 'p', long = "parallel", value_name = "N", default_value = "1")]
    parallel: usize,

    #[arg(short = 'r', long = "random-seed", value_name = "N")]
    random_seed: Option<u64>,

    /// Time limit in milliseconds (0 = use Selen default of 60000ms)
    #[arg(short = 't', long = "time", value_name = "MS", default_value = "0")]
    time_limit: u64,

    /// Memory limit in MB (0 = use Selen default of 2000MB)
    #[arg(long = "mem-limit", value_name = "MB", default_value = "0")]
    mem_limit: usize,

    /// Export the problem as a standalone Selen Rust program for debugging
    #[arg(long = "export-selen", value_name = "FILE")]
    export_selen: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        eprintln!("Reading FlatZinc file: {:?}", args.input);
    }

    // Create solver
    let mut solver = FlatZincSolver::new();

    // Configure statistics
    if !args.statistics {
        solver.with_statistics(false);
    }

    // Configure timeout
    if args.time_limit > 0 {
        solver.with_timeout(args.time_limit);
        if args.verbose {
            eprintln!("Timeout set to {} ms", args.time_limit);
        }
    }

    // Configure memory limit
    if args.mem_limit > 0 {
        solver.with_memory_limit(args.mem_limit);
        if args.verbose {
            eprintln!("Memory limit set to {} MB", args.mem_limit);
        }
    }

    // Load the FlatZinc file
    if let Err(e) = solver.load_file(args.input.to_str().unwrap()) {
        eprintln!("Error loading FlatZinc file: {:?}", e);
        process::exit(1);
    }

    if args.verbose {
        eprintln!("FlatZinc model loaded successfully");
    }

    // Export to Selen Rust program if requested
    if let Some(export_path) = args.export_selen {
        if args.verbose {
            eprintln!("Exporting Selen model to: {:?}", export_path);
        }
        if let Err(e) = solver.export_selen_program(export_path.to_str().unwrap()) {
            eprintln!("Error exporting Selen program: {:?}", e);
            process::exit(1);
        }
        eprintln!("Selen program exported successfully");
        process::exit(0);
    }

    // Configure solution search
    if args.all_solutions {
        solver.find_all_solutions();
        if args.verbose {
            eprintln!("Finding all solutions");
        }
    } else if let Some(n) = args.num_solutions {
        solver.max_solutions(n);
        if args.verbose {
            eprintln!("Finding up to {} solutions", n);
        }
    }

    // Note: intermediate flag affects optimization problems automatically
    // when max_solutions > 1 or find_all_solutions is set
    if args.intermediate && args.verbose {
        eprintln!("Intermediate solutions will be shown for optimization problems");
    }

    // Warn about unsupported options
    if args.free_search && args.verbose {
        eprintln!("Warning: --free-search option is not yet supported");
    }

    if args.parallel > 1 && args.verbose {
        eprintln!("Warning: --parallel option is not yet supported");
    }

    if args.random_seed.is_some() && args.verbose {
        eprintln!("Warning: --random-seed option is not yet supported");
    }

    if args.verbose {
        eprintln!("Solving...");
    }

    // Solve and print results
    match solver.solve() {
        Ok(()) => {
            if args.verbose {
                eprintln!("Found {} solution(s)", solver.solution_count());
            }
            solver.print_flatzinc();
            process::exit(0);
        }
        Err(()) => {
            if args.verbose {
                eprintln!("No solutions found");
            }
            solver.print_flatzinc();
            process::exit(0);
        }
    }
}
