use ciborium_ll::{Decoder, Error, Header};
use std::path::Path;
use std::process::{Command, ExitCode};

const ADMITTED_COMMIT: &str = "fdf3ec5c53256000f8e12f06c361ca18026a8d99";
const DISABLED_PUSH_URL: &str = "https://invalid.invalid/enarx/ciborium-direct-push-disabled";

#[derive(Default)]
struct Summary {
    direct_complete: u64,
    facade_complete: u64,
    reserved: u64,
    truncated: u64,
    empty: u64,
}

enum Outcome {
    Header(Header, usize),
    Syntax(usize, usize),
    Io(usize),
}

fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| format!("failed to run git: {error}"))?;
    if !output.status.success() {
        return Err(format!("git {args:?} failed with {}", output.status));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("git output was not UTF-8: {error}"))
}

fn check_checkout() -> Result<(), String> {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../checkout");
    let head = git(&repo, &["rev-parse", "HEAD"])?;
    if head != ADMITTED_COMMIT {
        return Err(format!(
            "checkout HEAD {head} != admitted {ADMITTED_COMMIT}"
        ));
    }
    if git(&repo, &["rev-parse", "--abbrev-ref", "HEAD"])? != "HEAD" {
        return Err("production checkout is not detached".into());
    }
    if !git(&repo, &["status", "--porcelain=v1"])?.is_empty() {
        return Err("production checkout is not clean".into());
    }
    let push_url = git(&repo, &["remote", "get-url", "--push", "upstream"])?;
    if push_url != DISABLED_PUSH_URL {
        return Err(format!("unexpected push URL: {push_url}"));
    }
    Ok(())
}

fn observe(input: &[u8]) -> Outcome {
    let mut decoder = Decoder::from(input);
    let result = decoder.pull();
    let offset = decoder.offset();
    match result {
        Ok(header) => Outcome::Header(header, offset),
        Err(Error::Syntax(position)) => Outcome::Syntax(position, offset),
        Err(Error::Io(_)) => Outcome::Io(offset),
    }
}

fn width(additional: u8) -> Option<usize> {
    match additional {
        0..=23 => Some(0),
        24 => Some(1),
        25 => Some(2),
        26 => Some(4),
        27 => Some(8),
        _ => None,
    }
}

fn encode(major: u8, additional: u8, argument: u64, trailing: bool) -> Vec<u8> {
    let n_argument_bytes = width(additional).expect("supported additional value");
    let mut input = Vec::with_capacity(1 + n_argument_bytes + usize::from(trailing));
    input.push((major << 5) | additional);
    let argument_bytes = argument.to_be_bytes();
    input.extend_from_slice(&argument_bytes[argument_bytes.len() - n_argument_bytes..]);
    if trailing {
        input.push(0xf6);
    }
    input
}

fn expected_header(major: u8, argument: u64) -> Header {
    let length = usize::try_from(argument).expect("search requires a 64-bit host");
    match major {
        0 => Header::Positive(argument),
        1 => Header::Negative(argument),
        2 => Header::Bytes(Some(length)),
        3 => Header::Text(Some(length)),
        4 => Header::Array(Some(length)),
        5 => Header::Map(Some(length)),
        _ => unreachable!(),
    }
}

fn check_complete(
    summary: &mut Summary,
    major: u8,
    additional: u8,
    argument: u64,
) -> Result<(), String> {
    let expected_offset = 1 + width(additional).expect("supported additional value");
    let expected = expected_header(major, argument);
    let input = encode(major, additional, argument, true);
    match observe(&input) {
        Outcome::Header(observed, offset) if observed == expected && offset == expected_offset => {}
        Outcome::Header(observed, offset) => {
            return Err(format!(
                "header mismatch: major={major} additional={additional} argument={argument} expected={expected:?}/{expected_offset} observed={observed:?}/{offset}"
            ));
        }
        Outcome::Syntax(position, offset) => {
            return Err(format!(
                "unexpected syntax error: major={major} additional={additional} argument={argument} position={position} offset={offset}"
            ));
        }
        Outcome::Io(offset) => {
            return Err(format!(
                "unexpected I/O error: major={major} additional={additional} argument={argument} offset={offset}"
            ));
        }
    }
    summary.direct_complete += 1;

    if major == 0 {
        let facade_input = encode(major, additional, argument, false);
        match ciborium::de::from_reader::<u64, _>(&facade_input[..]) {
            Ok(observed) if observed == argument => {}
            Ok(observed) => {
                return Err(format!(
                    "facade mismatch: additional={additional} argument={argument} observed={observed}"
                ));
            }
            Err(error) => {
                return Err(format!(
                    "facade rejected complete value: additional={additional} argument={argument} error={error}"
                ));
            }
        }
        summary.facade_complete += 1;
    }
    Ok(())
}

fn check_argument_set(
    summary: &mut Summary,
    additional: u8,
    arguments: impl IntoIterator<Item = u64> + Clone,
) -> Result<(), String> {
    for major in 0..=5 {
        for argument in arguments.clone() {
            check_complete(summary, major, additional, argument)?;
        }
    }
    Ok(())
}

fn check_reserved(summary: &mut Summary) -> Result<(), String> {
    for major in 0..=7 {
        for additional in 28..=30 {
            let input = [(major << 5) | additional, 0xa5];
            match observe(&input) {
                Outcome::Syntax(0, 1) => {}
                Outcome::Header(header, offset) => {
                    return Err(format!(
                        "reserved value accepted: major={major} additional={additional} header={header:?} offset={offset}"
                    ));
                }
                Outcome::Syntax(position, offset) => {
                    return Err(format!(
                        "reserved syntax location mismatch: major={major} additional={additional} position={position} offset={offset}"
                    ));
                }
                Outcome::Io(offset) => {
                    return Err(format!(
                        "reserved value caused I/O error: major={major} additional={additional} offset={offset}"
                    ));
                }
            }
            summary.reserved += 1;
        }
    }
    Ok(())
}

fn check_truncation(summary: &mut Summary) -> Result<(), String> {
    for major in 0..=5 {
        for additional in 24..=27 {
            let n_argument_bytes = width(additional).expect("supported additional value");
            for available in 0..n_argument_bytes {
                let full = encode(major, additional, u64::MAX, false);
                let input = &full[..1 + available];
                match observe(input) {
                    Outcome::Io(1) => {}
                    Outcome::Io(offset) => {
                        return Err(format!(
                            "truncation offset mismatch: major={major} additional={additional} available={available} offset={offset}"
                        ));
                    }
                    Outcome::Header(header, offset) => {
                        return Err(format!(
                            "truncation accepted: major={major} additional={additional} available={available} header={header:?} offset={offset}"
                        ));
                    }
                    Outcome::Syntax(position, offset) => {
                        return Err(format!(
                            "truncation caused syntax error: major={major} additional={additional} available={available} position={position} offset={offset}"
                        ));
                    }
                }
                summary.truncated += 1;
            }
        }
    }
    Ok(())
}

fn run_search() -> Result<Summary, String> {
    check_checkout()?;
    if usize::BITS != 64 {
        return Err(format!(
            "search requires a 64-bit host, found {}",
            usize::BITS
        ));
    }

    let mut summary = Summary::default();
    for additional in 0..=23 {
        check_argument_set(&mut summary, additional, [u64::from(additional)])?;
    }
    check_argument_set(&mut summary, 24, (u8::MIN..=u8::MAX).map(u64::from))?;
    check_argument_set(&mut summary, 25, (u16::MIN..=u16::MAX).map(u64::from))?;
    check_argument_set(
        &mut summary,
        26,
        [
            0,
            1,
            23,
            24,
            255,
            256,
            65_535,
            65_536,
            0x7fff_ffff,
            0x8000_0000,
            u64::from(u32::MAX),
        ],
    )?;
    check_argument_set(
        &mut summary,
        27,
        [
            0,
            1,
            23,
            24,
            255,
            256,
            65_535,
            65_536,
            u64::from(u32::MAX),
            u64::from(u32::MAX) + 1,
            i64::MAX as u64,
            (i64::MAX as u64) + 1,
            u64::MAX,
        ],
    )?;
    check_reserved(&mut summary)?;
    check_truncation(&mut summary)?;

    match observe(&[]) {
        Outcome::Io(0) => summary.empty += 1,
        Outcome::Io(offset) => return Err(format!("empty-input offset mismatch: {offset}")),
        Outcome::Header(header, offset) => {
            return Err(format!(
                "empty input accepted: header={header:?} offset={offset}"
            ));
        }
        Outcome::Syntax(position, offset) => {
            return Err(format!(
                "empty input caused syntax error: position={position} offset={offset}"
            ));
        }
    }
    Ok(summary)
}

fn main() -> ExitCode {
    match run_search() {
        Ok(summary) => {
            println!("status=no-counterexample");
            println!("direct_complete_cases={}", summary.direct_complete);
            println!("facade_complete_cases={}", summary.facade_complete);
            println!("reserved_cases={}", summary.reserved);
            println!("truncated_cases={}", summary.truncated);
            println!("empty_cases={}", summary.empty);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("COUNTEREXAMPLE: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkout_gate() {
        check_checkout().unwrap();
    }

    #[test]
    fn width_and_endian_boundaries() {
        let cases = [
            (0, 0),
            (23, 23),
            (24, 0),
            (24, 255),
            (25, 256),
            (25, 65_535),
            (26, 65_536),
            (26, u64::from(u32::MAX)),
            (27, u64::from(u32::MAX) + 1),
            (27, u64::MAX),
        ];
        let mut summary = Summary::default();
        for major in 0..=5 {
            for (additional, argument) in cases {
                check_complete(&mut summary, major, additional, argument).unwrap();
            }
        }
        assert_eq!(summary.direct_complete, 60);
        assert_eq!(summary.facade_complete, 10);
    }

    #[test]
    fn reserved_matrix() {
        let mut summary = Summary::default();
        check_reserved(&mut summary).unwrap();
        assert_eq!(summary.reserved, 24);
    }

    #[test]
    fn every_proper_argument_truncation() {
        let mut summary = Summary::default();
        check_truncation(&mut summary).unwrap();
        assert_eq!(summary.truncated, 90);
    }
}
