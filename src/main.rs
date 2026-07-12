// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

mod cli;

use std::fs;

use cli::{ChordsCli, Cli, ConvertCli};
use strum2midi::{
    error::{AppError, Result},
    midi_writer,
    midi_writer::MidiOptions,
    parser, validate, voicing,
};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    match Cli::parse(std::env::args().skip(1))? {
        Cli::Convert(cli) => convert(cli),
        Cli::Chords(cli) => print_chord(cli),
    }
}

fn convert(cli: ConvertCli) -> Result<()> {
    let input = fs::read_to_string(&cli.input).map_err(|source| AppError::Io {
        path: cli.input.clone(),
        source,
    })?;

    let mut song = parser::parse(&input)?;
    for warning in &song.warnings {
        eprintln!("{warning}");
    }
    if let Some(tempo) = cli.tempo {
        song.metadata.tempo = Some(tempo);
    }
    if let Some(voicing) = cli.voicing {
        song.metadata.voicing = Some(voicing);
    }
    let custom_voicings = cli
        .voicing_file
        .as_deref()
        .map(voicing::load_custom_file)
        .transpose()?;
    validate::validate_with_voicings(&song, None, custom_voicings.as_ref())?;

    let midi = midi_writer::write_midi(
        &song,
        MidiOptions {
            velocity: cli.velocity,
            strum_spread_ms: cli.strum_spread_ms,
            downstroke_spread_ms: cli.downstroke_spread_ms,
            upstroke_spread_ms: cli.upstroke_spread_ms,
            voicing: None,
            custom_voicings,
        },
    )?;

    fs::write(&cli.output, midi).map_err(|source| AppError::Io {
        path: cli.output,
        source,
    })?;

    Ok(())
}

fn print_chord(cli: ChordsCli) -> Result<()> {
    let custom_voicings = cli
        .voicing_file
        .as_deref()
        .map(voicing::load_custom_file)
        .transpose()?;
    let set = cli.voicing.as_deref().unwrap_or("folk");
    let selected = voicing::select_voicing(&cli.chord, set, custom_voicings.as_ref())?;

    if cli.diagram {
        let labels = ["E", "A", "D", "G", "B", "e"];
        for (label, fret) in labels.iter().rev().zip(selected.voicing.frets.iter().rev()) {
            let value = fret.map_or("x".to_string(), |fret| fret.to_string());
            println!("{label}|--{value}--");
        }
    } else {
        println!("{}  {}", cli.chord, selected.voicing.shape());
    }
    Ok(())
}
