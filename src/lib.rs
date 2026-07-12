// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

pub mod chord;
pub mod error;
pub mod harmony;
pub mod midi_writer;
pub mod model;
pub mod parser;
pub mod timing;
pub mod validate;
pub mod voicing;

pub use error::{AppError, Result};
pub use midi_writer::MidiOptions;
pub use model::Song;

pub fn convert_strum_to_midi(input: &str, options: MidiOptions) -> Result<Vec<u8>> {
    let song = parser::parse(input)?;
    convert_song_to_midi(&song, options)
}

pub fn convert_song_to_midi(song: &Song, options: MidiOptions) -> Result<Vec<u8>> {
    validate::validate(song)?;
    midi_writer::write_midi(song, options)
}
