// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use strum2midi::error::{AppError, Result};

#[derive(Debug, PartialEq, Eq)]
pub struct Cli {
    pub input: PathBuf,
    pub output: PathBuf,
    pub tempo: Option<u16>,
    pub velocity: Option<u8>,
    pub strum_spread_ms: Option<u16>,
}

impl Cli {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut positionals = Vec::new();
        let mut tempo = None;
        let mut velocity = None;
        let mut strum_spread_ms = None;
        let mut iter = args.into_iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--tempo" => tempo = Some(parse_next(&mut iter, "--tempo")?),
                "--velocity" => velocity = Some(parse_next(&mut iter, "--velocity")?),
                "--strum-spread-ms" => {
                    strum_spread_ms = Some(parse_next(&mut iter, "--strum-spread-ms")?)
                }
                "-h" | "--help" => return Err(AppError::Usage(usage())),
                flag if flag.starts_with('-') => {
                    return Err(AppError::Usage(format!(
                        "unknown option '{flag}'\n{}",
                        usage()
                    )));
                }
                positional => positionals.push(PathBuf::from(positional)),
            }
        }

        if positionals.len() != 2 {
            return Err(AppError::Usage(usage()));
        }

        Ok(Self {
            input: positionals.remove(0),
            output: positionals.remove(0),
            tempo,
            velocity,
            strum_spread_ms,
        })
    }
}

fn parse_next<T>(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<T>
where
    T: std::str::FromStr,
{
    let value = iter
        .next()
        .ok_or_else(|| AppError::Usage(format!("missing value for {flag}\n{}", usage())))?;
    value
        .parse()
        .map_err(|_| AppError::Usage(format!("invalid value for {flag}: '{value}'")))
}

fn usage() -> String {
    "usage: strum2midi input.strum output.mid [--tempo BPM] [--velocity VALUE] [--strum-spread-ms MS]".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_positionals_and_flags() {
        let cli = Cli::parse([
            "in.strum".to_string(),
            "out.mid".to_string(),
            "--tempo".to_string(),
            "100".to_string(),
            "--velocity".to_string(),
            "85".to_string(),
        ])
        .unwrap();

        assert_eq!(cli.input, PathBuf::from("in.strum"));
        assert_eq!(cli.output, PathBuf::from("out.mid"));
        assert_eq!(cli.tempo, Some(100));
        assert_eq!(cli.velocity, Some(85));
    }
}
