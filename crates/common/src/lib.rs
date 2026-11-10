//! Shared helpers for every utility in the workspace.

use std::io;
use std::process::ExitCode;

/// Runs a utility's body and turns its result into an exit code.
///
/// Errors are printed as `<name>: <error>` on stderr, like GNU coreutils.
/// A broken pipe (e.g. `yes | head`) is treated as a quiet exit, not an error.
pub fn run(name: &str, body: impl FnOnce() -> io::Result<()>) -> ExitCode {
    match body() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{name}: {e}");
            ExitCode::FAILURE
        }
    }
}
