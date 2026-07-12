// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use strum2midi::error::{AppError, Result};

#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    Convert(ConvertCli),
    Chords(ChordsCli),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConvertCli {
    pub input: PathBuf,
    pub output: PathBuf,
    pub tempo: Option<u16>,
    pub velocity: Option<u8>,
    pub strum_spread_ms: Option<u16>,
    pub downstroke_spread_ms: Option<u16>,
    pub upstroke_spread_ms: Option<u16>,
    pub voicing: Option<String>,
    pub voicing_file: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChordsCli {
    pub chord: String,
    pub voicing: Option<String>,
    pub voicing_file: Option<PathBuf>,
    pub diagram: bool,
}

impl Cli {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let args = args.into_iter().collect::<Vec<_>>();
        let mut positionals = Vec::new();
        let mut tempo = None;
        let mut velocity = None;
        let mut strum_spread_ms = None;
        let mut downstroke_spread_ms = None;
        let mut upstroke_spread_ms = None;
        let mut voicing = None;
        let mut voicing_file = None;
        let mut diagram = false;

        if args.first().is_some_and(|command| command == "chords") {
            let mut iter = args.into_iter().skip(1);
            while let Some(arg) = iter.next() {
                match arg.as_str() {
                    "--voicing" => voicing = Some(parse_next(&mut iter, "--voicing")?),
                    "--voicing-file" => {
                        voicing_file = Some(PathBuf::from(parse_next::<String>(
                            &mut iter,
                            "--voicing-file",
                        )?))
                    }
                    "--diagram" => diagram = true,
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
            if positionals.len() != 1 {
                return Err(AppError::Usage(usage()));
            }
            return Ok(Self::Chords(ChordsCli {
                chord: positionals.remove(0).to_string_lossy().to_string(),
                voicing,
                voicing_file,
                diagram,
            }));
        }

        let mut iter = args.into_iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--tempo" => tempo = Some(parse_next(&mut iter, "--tempo")?),
                "--velocity" => velocity = Some(parse_next(&mut iter, "--velocity")?),
                "--strum-spread-ms" => {
                    strum_spread_ms = Some(parse_next(&mut iter, "--strum-spread-ms")?)
                }
                "--downstroke-spread-ms" => {
                    downstroke_spread_ms = Some(parse_next(&mut iter, "--downstroke-spread-ms")?)
                }
                "--upstroke-spread-ms" => {
                    upstroke_spread_ms = Some(parse_next(&mut iter, "--upstroke-spread-ms")?)
                }
                "--voicing" => voicing = Some(parse_next(&mut iter, "--voicing")?),
                "--voicing-file" => {
                    voicing_file = Some(PathBuf::from(parse_next::<String>(
                        &mut iter,
                        "--voicing-file",
                    )?))
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

        Ok(Self::Convert(ConvertCli {
            input: positionals.remove(0),
            output: positionals.remove(0),
            tempo,
            velocity,
            strum_spread_ms,
            downstroke_spread_ms,
            upstroke_spread_ms,
            voicing,
            voicing_file,
        }))
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
    "usage: strum2midi input.strum output.mid [--tempo BPM] [--velocity VALUE] [--strum-spread-ms MS] [--downstroke-spread-ms MS] [--upstroke-spread-ms MS] [--voicing SET] [--voicing-file PATH]\n       strum2midi chords CHORD [--voicing SET] [--voicing-file PATH] [--diagram]".to_string()
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
            "--downstroke-spread-ms".to_string(),
            "120".to_string(),
            "--upstroke-spread-ms".to_string(),
            "90".to_string(),
        ])
        .unwrap();

        let Cli::Convert(cli) = cli else {
            panic!("expected convert command");
        };
        assert_eq!(cli.input, PathBuf::from("in.strum"));
        assert_eq!(cli.output, PathBuf::from("out.mid"));
        assert_eq!(cli.tempo, Some(100));
        assert_eq!(cli.velocity, Some(85));
        assert_eq!(cli.downstroke_spread_ms, Some(120));
        assert_eq!(cli.upstroke_spread_ms, Some(90));
    }

    #[test]
    fn parses_chords_command() {
        let cli = Cli::parse([
            "chords".to_string(),
            "C".to_string(),
            "--voicing".to_string(),
            "folk".to_string(),
            "--diagram".to_string(),
        ])
        .unwrap();

        let Cli::Chords(cli) = cli else {
            panic!("expected chords command");
        };
        assert_eq!(cli.chord, "C");
        assert_eq!(cli.voicing.as_deref(), Some("folk"));
        assert!(cli.diagram);
    }
}
