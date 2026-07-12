//! `atlas-local` binary entrypoint.

use atlas_local::run;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            for cause in err.chain().skip(1) {
                eprintln!("  → {cause}");
            }
            ExitCode::FAILURE
        }
    }
}
