mod chord;
mod cli;
mod error;
mod midi_writer;
mod model;
mod parser;
mod timing;
mod validate;

use std::fs;

use cli::Cli;
use error::{AppError, Result};
use midi_writer::MidiOptions;

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
            velocity: cli.velocity.unwrap_or(90),
            strum_spread_ms: cli.strum_spread_ms.unwrap_or(20),
        },
    )?;

    fs::write(&cli.output, midi).map_err(|source| AppError::Io {
        path: cli.output,
        source,
    })?;

    Ok(())
}
