// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chord,
    error::Result,
    model::{Song, StrumSymbol},
    timing,
};

#[derive(Debug, Clone, Copy)]
pub struct MidiOptions {
    pub velocity: u8,
    pub strum_spread_ms: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MidiEvent {
    tick: u32,
    order: u8,
    bytes: [u8; 3],
}

pub fn write_midi(song: &Song, options: MidiOptions) -> Result<Vec<u8>> {
    let tempo = song.metadata.tempo.expect("validated song has tempo");
    let time_signature = song
        .metadata
        .time_signature
        .expect("validated song has time");
    let slot_ticks = timing::slot_ticks(song)?;
    let slots_per_beat = u32::try_from(timing::slots_per_beat(
        song.metadata.beat.unwrap_or(crate::model::Beat::Quarter),
        song.metadata.subdivision.unwrap_or(16),
    )?)
    .unwrap_or(u32::MAX);
    let note_duration = slot_ticks.saturating_sub(1).max(1);
    let spread_ticks = timing::ms_to_ticks(options.strum_spread_ms, tempo);
    let mut events = Vec::new();

    for (bar_index, bar) in song.bars.iter().enumerate() {
        let bar_start = u32::try_from(bar_index).unwrap_or(u32::MAX) * timing::bar_ticks(song)?;
        for (beat_index, beat) in bar.beats.iter().enumerate() {
            let chord_notes = chord::notes_for_chord(&beat.chord).expect("validated chord exists");
            for (slot_index, symbol) in beat.slots.iter().enumerate() {
                let slot_offset = (u32::try_from(beat_index).unwrap_or(u32::MAX) * slots_per_beat
                    + u32::try_from(slot_index).unwrap_or(u32::MAX))
                    * slot_ticks;
                let tick = bar_start + slot_offset;
                match symbol {
                    StrumSymbol::Down => push_strum(
                        &mut events,
                        tick,
                        &chord_notes,
                        false,
                        options.velocity,
                        note_duration,
                        spread_ticks,
                    ),
                    StrumSymbol::Up => push_strum(
                        &mut events,
                        tick,
                        &chord_notes,
                        true,
                        options.velocity,
                        note_duration,
                        spread_ticks,
                    ),
                    StrumSymbol::Muted => push_muted(&mut events, tick, note_duration),
                    StrumSymbol::Rest => {}
                }
            }
        }
    }

    events.sort_by_key(|event| (event.tick, event.order));

    let mut track = Vec::new();
    write_delta(&mut track, 0);
    track.extend([0xFF, 0x51, 0x03]);
    track.extend(microseconds_per_quarter_note(tempo).to_be_bytes()[1..].iter());

    write_delta(&mut track, 0);
    track.extend([0xFF, 0x58, 0x04]);
    track.push(time_signature.numerator);
    track.push(denominator_power(time_signature.denominator));
    track.extend([24, 8]);

    write_delta(&mut track, 0);
    track.extend([
        0xC0,
        crate::validate::resolved_instrument(song).midi_program(),
    ]);

    let mut current_tick = 0;
    for event in events {
        write_delta(&mut track, event.tick - current_tick);
        track.extend(event.bytes);
        current_tick = event.tick;
    }

    write_delta(&mut track, 0);
    track.extend([0xFF, 0x2F, 0x00]);

    let mut file = Vec::new();
    file.extend(b"MThd");
    file.extend(6_u32.to_be_bytes());
    file.extend(0_u16.to_be_bytes());
    file.extend(1_u16.to_be_bytes());
    file.extend(timing::PPQ.to_be_bytes());
    file.extend(b"MTrk");
    file.extend(u32::try_from(track.len()).unwrap_or(u32::MAX).to_be_bytes());
    file.extend(track);
    Ok(file)
}

fn push_strum(
    events: &mut Vec<MidiEvent>,
    tick: u32,
    notes: &[u8],
    reverse: bool,
    velocity: u8,
    duration: u32,
    spread: u32,
) {
    let iter: Box<dyn Iterator<Item = &u8>> = if reverse {
        Box::new(notes.iter().rev())
    } else {
        Box::new(notes.iter())
    };

    for (index, note) in iter.enumerate() {
        let note_tick = tick + u32::try_from(index).unwrap_or(u32::MAX) * spread;
        events.push(MidiEvent {
            tick: note_tick,
            order: 1,
            bytes: [0x90, *note, velocity],
        });
        events.push(MidiEvent {
            tick: note_tick + duration,
            order: 0,
            bytes: [0x80, *note, 0],
        });
    }
}

fn push_muted(events: &mut Vec<MidiEvent>, tick: u32, duration: u32) {
    let note = 37;
    events.push(MidiEvent {
        tick,
        order: 1,
        bytes: [0x99, note, 35],
    });
    events.push(MidiEvent {
        tick: tick + duration.min(30),
        order: 0,
        bytes: [0x89, note, 0],
    });
}

fn microseconds_per_quarter_note(tempo: u16) -> u32 {
    60_000_000 / u32::from(tempo)
}

fn denominator_power(denominator: u8) -> u8 {
    denominator.ilog2() as u8
}

fn write_delta(output: &mut Vec<u8>, value: u32) {
    let mut buffer = [0_u8; 5];
    let mut index = 4;
    buffer[index] = (value & 0x7F) as u8;
    let mut remaining = value >> 7;
    while remaining > 0 {
        index -= 1;
        buffer[index] = ((remaining & 0x7F) as u8) | 0x80;
        remaining >>= 7;
    }
    output.extend(&buffer[index..]);
}

#[cfg(test)]
mod tests {
    use crate::{parser, validate};

    use super::*;

    #[test]
    fn writes_valid_midi_header_and_tempo() {
        let song = parser::parse("tempo: 120\ntime: 4/4\n\nC\n---- ---- ---- ----\n").unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(
            &song,
            MidiOptions {
                velocity: 90,
                strum_spread_ms: 20,
            },
        )
        .unwrap();

        assert_eq!(&midi[..4], b"MThd");
        assert!(
            midi.windows(6)
                .any(|window| window == [0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20])
        );
        assert!(midi.windows(2).any(|window| window == [0xC0, 25]));
    }

    #[test]
    fn writes_explicit_instrument_program_change() {
        let song = parser::parse(
            "tempo: 120\ntime: 4/4\ninstrument: electric_guitar_clean\n\nC\n---- ---- ---- ----\n",
        )
        .unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(
            &song,
            MidiOptions {
                velocity: 90,
                strum_spread_ms: 20,
            },
        )
        .unwrap();

        assert!(midi.windows(2).any(|window| window == [0xC0, 27]));
    }

    #[test]
    fn downstroke_orders_notes_low_to_high() {
        let song = parser::parse("tempo: 92\ntime: 4/4\n\nC\nD--- ---- ---- ----\n").unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(
            &song,
            MidiOptions {
                velocity: 90,
                strum_spread_ms: 20,
            },
        )
        .unwrap();

        assert_note_on_order(&midi, &[48, 52, 55]);
    }

    #[test]
    fn upstroke_orders_notes_high_to_low() {
        let song = parser::parse("tempo: 92\ntime: 4/4\n\nC\n--U- ---- ---- ----\n").unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(
            &song,
            MidiOptions {
                velocity: 90,
                strum_spread_ms: 20,
            },
        )
        .unwrap();

        assert_note_on_order(&midi, &[55, 52, 48]);
    }

    fn assert_note_on_order(midi: &[u8], expected: &[u8]) {
        let notes = midi
            .windows(3)
            .filter(|window| window[0] == 0x90 && window[2] > 0)
            .map(|window| window[1])
            .take(expected.len())
            .collect::<Vec<_>>();
        assert_eq!(notes, expected);
    }
}
