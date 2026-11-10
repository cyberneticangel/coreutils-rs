// true - do nothing, successfully.
//
// Goal: exit with status 0 and print nothing.
// Try it: cargo run -p true; echo $?   -> 0

use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::FAILURE
}
