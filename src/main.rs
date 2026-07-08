// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

mod cli;

use std::fs;

use cli::Cli;
use strum2midi::{
    error::{AppError, Result},
    midi_writer,
    midi_writer::MidiOptions,
    parser, validate,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse(std::env::args().skip(1))?;
    let input = fs::read_to_string(&cli.input).map_err(|source| AppError::Io {
        path: cli.input.clone(),
        source,
    })?;

    let mut song = parser::parse(&input)?;
    if let Some(tempo) = cli.tempo {
        song.metadata.tempo = Some(tempo);
    }
    validate::validate(&song)?;

    let midi = midi_writer::write_midi(
        &song,
        MidiOptions {
            velocity: cli.velocity,
            strum_spread_ms: cli.strum_spread_ms,
        },
    )?;

    fs::write(&cli.output, midi).map_err(|source| AppError::Io {
        path: cli.output,
        source,
    })?;

    Ok(())
}
