//! Measures scrolling smoothness of whichever browser opens the served page; see `benchmark_command`.

use std::process::ExitCode;

use induckgo_bench::benchmark_command::BenchmarkCommand;

fn main() -> ExitCode {
    match BenchmarkCommand::from_arguments(std::env::args().skip(1)).and_then(|command| command.run()) {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("induckgo-bench: {message}");
            ExitCode::FAILURE
        }
    }
}
