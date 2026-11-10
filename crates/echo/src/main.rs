//! echo - display a line of text
//!
//! Follows GNU semantics: options are only recognized before the first
//! operand, `--` is printed literally, and an unknown flag such as `-x`
//! is treated as text.

use std::ffi::OsString;
use std::io::{self, BufWriter, Write};
use std::os::unix::ffi::OsStrExt;
use std::process::ExitCode;

struct Options {
    newline: bool,
    escapes: bool,
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    common::run("echo", || echo(&args))
}

fn echo(args: &[OsString]) -> io::Result<()> {
    let (opts, operands) = parse_options(args);
    let mut out = BufWriter::new(io::stdout().lock());

    for (i, arg) in operands.iter().enumerate() {
        if i > 0 {
            out.write_all(b" ")?;
        }
        let bytes = arg.as_bytes();
        if opts.escapes {
            if write_escaped(&mut out, bytes)? == Flow::Stop {
                return out.flush();
            }
        } else {
            out.write_all(bytes)?;
        }
    }

    if opts.newline {
        out.write_all(b"\n")?;
    }
    out.flush()
}

/// Consumes leading flag arguments like `-n`, `-e`, `-neE`. Stops at the first
/// argument that isn't made up entirely of known flag letters.
fn parse_options(args: &[OsString]) -> (Options, &[OsString]) {
    let mut opts = Options { newline: true, escapes: false };
    let mut consumed = 0;

    for arg in args {
        let flags = match arg.as_bytes() {
            [b'-', rest @ ..] if !rest.is_empty() => rest,
            _ => break,
        };
        if !flags.iter().all(|c| matches!(c, b'n' | b'e' | b'E')) {
            break;
        }
        for c in flags {
            match c {
                b'n' => opts.newline = false,
                b'e' => opts.escapes = true,
                b'E' => opts.escapes = false,
                _ => unreachable!(),
            }
        }
        consumed += 1;
    }

    (opts, &args[consumed..])
}

#[derive(PartialEq)]
enum Flow {
    Continue,
    /// `\c` was seen: produce no further output, including the newline.
    Stop,
}

fn write_escaped(out: &mut impl Write, s: &[u8]) -> io::Result<Flow> {
    let mut i = 0;
    while i < s.len() {
        if s[i] != b'\\' || i + 1 == s.len() {
            out.write_all(&s[i..=i])?;
            i += 1;
            continue;
        }
        i += 1;
        let byte = match s[i] {
            b'\\' => b'\\',
            b'a' => 0x07,
            b'b' => 0x08,
            b'c' => return Ok(Flow::Stop),
            b'e' => 0x1b,
            b'f' => 0x0c,
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'v' => 0x0b,
            // \0nnn: up to three octal digits after the 0
            b'0' => {
                let (value, len) = parse_digits(&s[i + 1..], 8, 3);
                i += len;
                value
            }
            // \xHH: one or two hex digits; a bare \x is printed as-is
            b'x' => match parse_digits(&s[i + 1..], 16, 2) {
                (_, 0) => {
                    out.write_all(b"\\x")?;
                    i += 1;
                    continue;
                }
                (value, len) => {
                    i += len;
                    value
                }
            },
            // Unknown escape: keep the backslash
            other => {
                out.write_all(&[b'\\', other])?;
                i += 1;
                continue;
            }
        };
        out.write_all(&[byte])?;
        i += 1;
    }
    Ok(Flow::Continue)
}

/// Parses up to `max` digits in `radix` from the start of `s`.
/// Returns the value (wrapped to a byte) and how many digits were used.
fn parse_digits(s: &[u8], radix: u32, max: usize) -> (u8, usize) {
    let mut value: u32 = 0;
    let mut len = 0;
    for &c in s.iter().take(max) {
        match (c as char).to_digit(radix) {
            Some(d) => {
                value = value * radix + d;
                len += 1;
            }
            None => break,
        }
    }
    (value as u8, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escaped(s: &str) -> (Vec<u8>, bool) {
        let mut out = Vec::new();
        let flow = write_escaped(&mut out, s.as_bytes()).unwrap();
        (out, flow == Flow::Stop)
    }

    #[test]
    fn simple_escapes() {
        assert_eq!(escaped(r"a\tb\nc"), (b"a\tb\nc".to_vec(), false));
        assert_eq!(escaped(r"\\"), (b"\\".to_vec(), false));
    }

    #[test]
    fn octal_and_hex() {
        assert_eq!(escaped(r"\0101\x42"), (b"AB".to_vec(), false));
        assert_eq!(escaped(r"\0"), (vec![0], false));
        assert_eq!(escaped(r"\xZ"), (b"\\xZ".to_vec(), false));
    }

    #[test]
    fn stop_and_unknown() {
        assert_eq!(escaped(r"ab\cde"), (b"ab".to_vec(), true));
        assert_eq!(escaped(r"\q"), (b"\\q".to_vec(), false));
        assert_eq!(escaped("trailing\\"), (b"trailing\\".to_vec(), false));
    }

    #[test]
    fn option_parsing() {
        let args: Vec<OsString> = ["-ne", "-x", "hi"].iter().map(Into::into).collect();
        let (opts, rest) = parse_options(&args);
        assert!(!opts.newline && opts.escapes);
        assert_eq!(rest, &args[1..]);
    }
}
