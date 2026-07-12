// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{AppError, Result},
    model::{Beat, Song, TimeSignature},
};

pub const PPQ: u16 = 480;

pub fn slot_ticks(song: &Song) -> Result<u32> {
    let beat = resolved_beat(song);
    let subdivision = resolved_subdivision(song);
    Ok(beat_ticks(beat) / u32::try_from(slots_per_beat(beat, subdivision)?).unwrap_or(u32::MAX))
}

pub fn bar_ticks(song: &Song) -> Result<u32> {
    let time_signature = song
        .metadata
        .time_signature
        .ok_or_else(|| AppError::Validation("missing time signature".to_string()))?;
    beat_ticks(resolved_beat(song))
        .checked_mul(
            u32::try_from(beats_per_bar(time_signature, resolved_beat(song))?).map_err(|_| {
                AppError::Encoding("MIDI timing exceeds supported range".to_string())
            })?,
        )
        .ok_or_else(|| AppError::Encoding("MIDI timing exceeds supported range".to_string()))
}

pub fn slots_per_beat(beat: Beat, subdivision: u8) -> Result<usize> {
    match (beat, subdivision) {
        (Beat::Quarter, 8) => Ok(2),
        (Beat::Quarter, 16) => Ok(4),
        (Beat::DottedQuarter, 8) => Ok(3),
        (Beat::DottedQuarter, 16) => Ok(6),
        (_, other) => Err(AppError::Validation(format!(
            "unsupported subdivision '{other}'"
        ))),
    }
}

pub fn ms_to_ticks(ms: u16, tempo_bpm: u16) -> u32 {
    let ticks = u64::from(ms) * u64::from(tempo_bpm) * u64::from(PPQ) / 60_000;
    u32::try_from(ticks.max(1)).unwrap_or(u32::MAX)
}

fn resolved_beat(song: &Song) -> Beat {
    song.metadata.beat.unwrap_or(Beat::Quarter)
}

fn resolved_subdivision(song: &Song) -> u8 {
    song.metadata.subdivision.unwrap_or(16)
}

fn beat_ticks(beat: Beat) -> u32 {
    match beat {
        Beat::Quarter => u32::from(PPQ),
        Beat::DottedQuarter => u32::from(PPQ) * 3 / 2,
    }
}

fn beats_per_bar(time_signature: TimeSignature, beat: Beat) -> Result<usize> {
    match (beat, time_signature.numerator, time_signature.denominator) {
        (Beat::Quarter, numerator, 4) => Ok(usize::from(numerator)),
        (Beat::DottedQuarter, numerator, 8) if numerator % 3 == 0 => Ok(usize::from(numerator / 3)),
        _ => Err(AppError::Validation(format!(
            "unsupported time signature {}/{} with selected beat",
            time_signature.numerator, time_signature.denominator
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Metadata, Song};

    #[test]
    fn calculates_sixteenth_slots_for_four_four() {
        let song = Song {
            metadata: Metadata {
                tempo: Some(92),
                time_signature: Some(TimeSignature {
                    numerator: 4,
                    denominator: 4,
                }),
                velocity: None,
                strum_spread_ms: None,
                downstroke_velocity: None,
                upstroke_velocity: None,
                downstroke_spread_ms: None,
                upstroke_spread_ms: None,
                upstroke_max_strings: None,
                beat: None,
                subdivision: None,
                count: None,
                instrument: None,
                voicing: None,
            },
            parts: Vec::new(),
            warnings: Vec::new(),
            bars: Vec::new(),
        };

        assert_eq!(slot_ticks(&song).unwrap(), 120);
    }

    #[test]
    fn calculates_eighth_slots_for_six_eight_compound() {
        let song = Song {
            metadata: Metadata {
                tempo: Some(72),
                time_signature: Some(TimeSignature {
                    numerator: 6,
                    denominator: 8,
                }),
                velocity: None,
                strum_spread_ms: None,
                downstroke_velocity: None,
                upstroke_velocity: None,
                downstroke_spread_ms: None,
                upstroke_spread_ms: None,
                upstroke_max_strings: None,
                beat: Some(Beat::DottedQuarter),
                subdivision: Some(8),
                count: None,
                instrument: None,
                voicing: None,
            },
            parts: Vec::new(),
            warnings: Vec::new(),
            bars: Vec::new(),
        };

        assert_eq!(slot_ticks(&song).unwrap(), 240);
        assert_eq!(bar_ticks(&song).unwrap(), u32::from(PPQ) * 3);
    }
}
