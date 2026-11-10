//! End-to-end tests that run the compiled `echo` binary.

use std::process::Command;

fn echo(args: &[&str]) -> Vec<u8> {
    let output = Command::new(env!("CARGO_BIN_EXE_echo"))
        .args(args)
        .output()
        .expect("failed to run echo");
    assert!(output.status.success());
    output.stdout
}

#[test]
fn joins_args_with_spaces() {
    assert_eq!(echo(&["hello", "world"]), b"hello world\n");
}

#[test]
fn no_args_prints_newline() {
    assert_eq!(echo(&[]), b"\n");
}

#[test]
fn suppress_newline() {
    assert_eq!(echo(&["-n", "hi"]), b"hi");
}

#[test]
fn escapes_only_with_e() {
    assert_eq!(echo(&["a\\tb"]), b"a\\tb\n");
    assert_eq!(echo(&["-e", "a\\tb"]), b"a\tb\n");
}

#[test]
fn backslash_c_stops_output() {
    assert_eq!(echo(&["-e", "one\\c", "two"]), b"one");
}

#[test]
fn options_only_before_operands() {
    assert_eq!(echo(&["hi", "-n"]), b"hi -n\n");
    assert_eq!(echo(&["--", "hi"]), b"-- hi\n");
    assert_eq!(echo(&["-x", "hi"]), b"-x hi\n");
}
