// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chord,
    error::{AppError, Result},
    model::{Song, StrumSymbol},
    timing,
    voicing::{self, VoicingLibrary},
};

#[derive(Debug, Clone, Default)]
pub struct MidiOptions {
    pub velocity: Option<u8>,
    pub strum_spread_ms: Option<u16>,
    pub downstroke_spread_ms: Option<u16>,
    pub upstroke_spread_ms: Option<u16>,
    pub voicing: Option<String>,
    pub custom_voicings: Option<VoicingLibrary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MidiEvent {
    tick: u32,
    order: u8,
    bytes: Vec<u8>,
}

pub fn write_midi(song: &Song, options: MidiOptions) -> Result<Vec<u8>> {
    let tempo = song
        .metadata
        .tempo
        .ok_or_else(|| AppError::Validation("missing tempo".to_string()))?;
    if tempo == 0 {
        return Err(AppError::Validation(
            "tempo must be greater than zero".to_string(),
        ));
    }
    let time_signature = song
        .metadata
        .time_signature
        .ok_or_else(|| AppError::Validation("missing time signature".to_string()))?;
    let slot_ticks = timing::slot_ticks(song)?;
    let slots_per_beat = u32::try_from(timing::slots_per_beat(
        song.metadata.beat.unwrap_or(crate::model::Beat::Quarter),
        song.metadata.subdivision.unwrap_or(16),
    )?)
    .map_err(|_| midi_range_error())?;
    let note_duration = slot_ticks.saturating_sub(1).max(1);
    let velocity = crate::validate::resolved_velocity(song, options.velocity)?;
    let strum_spread_ms = crate::validate::resolved_strum_spread_ms(song, options.strum_spread_ms);
    let spread_ticks = timing::ms_to_ticks(strum_spread_ms, tempo);
    let active_voicing = options.voicing.as_ref().or(song.metadata.voicing.as_ref());
    let voiced_settings =
        VoicedStrumSettings::resolve(song, &options, tempo, velocity, strum_spread_ms);
    let mut events = Vec::new();
    let bar_ticks = timing::bar_ticks(song)?;

    for part in &song.parts {
        events.push(MidiEvent {
            tick: checked_mul(to_u32(part.bar_index)?, bar_ticks)?,
            order: 0,
            bytes: marker_event(part.name.as_str()),
        });
    }

    for (bar_index, bar) in song.bars.iter().enumerate() {
        let bar_start = checked_mul(to_u32(bar_index)?, bar_ticks)?;
        for (beat_index, beat) in bar.beats.iter().enumerate() {
            let rendered_notes = if let Some(voicing_set) = active_voicing {
                voicing::select_voicing(&beat.chord, voicing_set, options.custom_voicings.as_ref())
                    .ok()
                    .map(|selected| {
                        RenderedNotes::Voiced(
                            selected
                                .notes
                                .iter()
                                .map(|note| note.midi_note)
                                .collect::<Vec<_>>(),
                        )
                    })
            } else {
                chord::notes_for_chord(&beat.chord).map(RenderedNotes::Legacy)
            }
            .ok_or_else(|| {
                chord_error(
                    beat.chord_line,
                    beat.chord.as_str(),
                    active_voicing.map(String::as_str),
                    options.custom_voicings.as_ref(),
                )
            })?;
            for (slot_index, symbol) in beat.slots.iter().enumerate() {
                let beat_slot_offset = checked_add(
                    checked_mul(to_u32(beat_index)?, slots_per_beat)?,
                    to_u32(slot_index)?,
                )?;
                let slot_offset = checked_mul(beat_slot_offset, slot_ticks)?;
                let tick = checked_add(bar_start, slot_offset)?;
                match symbol {
                    StrumSymbol::Down => match &rendered_notes {
                        RenderedNotes::Legacy(notes) => push_legacy_strum(
                            &mut events,
                            tick,
                            notes,
                            false,
                            velocity,
                            note_duration,
                            spread_ticks,
                        )?,
                        RenderedNotes::Voiced(notes) => push_voiced_strum(
                            &mut events,
                            tick,
                            notes,
                            Stroke::Down,
                            note_duration,
                            voiced_settings,
                        )?,
                    },
                    StrumSymbol::Up => match &rendered_notes {
                        RenderedNotes::Legacy(notes) => push_legacy_strum(
                            &mut events,
                            tick,
                            notes,
                            true,
                            velocity,
                            note_duration,
                            spread_ticks,
                        )?,
                        RenderedNotes::Voiced(notes) => push_voiced_strum(
                            &mut events,
                            tick,
                            notes,
                            Stroke::Up,
                            note_duration,
                            voiced_settings,
                        )?,
                    },
                    StrumSymbol::Muted => match &rendered_notes {
                        RenderedNotes::Legacy(notes) | RenderedNotes::Voiced(notes) => {
                            push_muted(&mut events, tick, notes, note_duration)?
                        }
                    },
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
    track.push(denominator_power(time_signature.denominator)?);
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
    file.extend(to_u32(track.len())?.to_be_bytes());
    file.extend(track);
    Ok(file)
}

#[derive(Debug, Clone, Copy)]
enum Stroke {
    Down,
    Up,
}

#[derive(Debug, Clone)]
enum RenderedNotes {
    Legacy(Vec<u8>),
    Voiced(Vec<u8>),
}

#[derive(Debug, Clone, Copy)]
struct VoicedStrumSettings {
    down_velocity: u8,
    up_velocity: u8,
    down_sweep_ticks: u32,
    up_sweep_ticks: u32,
    up_max_strings: usize,
}

impl VoicedStrumSettings {
    fn resolve(
        song: &Song,
        options: &MidiOptions,
        tempo: u16,
        default_velocity: u8,
        default_spread_ms: u16,
    ) -> Self {
        let down_velocity = song
            .metadata
            .downstroke_velocity
            .unwrap_or(default_velocity);
        let up_velocity = song
            .metadata
            .upstroke_velocity
            .unwrap_or_else(|| down_velocity.saturating_mul(4) / 5);
        let down_spread_ms = options
            .downstroke_spread_ms
            .or(song.metadata.downstroke_spread_ms)
            .unwrap_or(default_spread_ms.max(28));
        let up_spread_ms = options
            .upstroke_spread_ms
            .or(song.metadata.upstroke_spread_ms)
            .unwrap_or(default_spread_ms.max(22));
        Self {
            down_velocity,
            up_velocity,
            down_sweep_ticks: timing::ms_to_ticks(down_spread_ms, tempo),
            up_sweep_ticks: timing::ms_to_ticks(up_spread_ms, tempo),
            up_max_strings: usize::from(song.metadata.upstroke_max_strings.unwrap_or(4).max(1)),
        }
    }
}

fn push_legacy_strum(
    events: &mut Vec<MidiEvent>,
    tick: u32,
    notes: &[u8],
    reverse: bool,
    velocity: u8,
    duration: u32,
    spread: u32,
) -> Result<()> {
    let iter: Box<dyn Iterator<Item = &u8>> = if reverse {
        Box::new(notes.iter().rev())
    } else {
        Box::new(notes.iter())
    };

    for (index, note) in iter.enumerate() {
        let note_tick = checked_add(tick, checked_mul(to_u32(index)?, spread)?)?;
        events.push(MidiEvent {
            tick: note_tick,
            order: 1,
            bytes: vec![0x90, *note, velocity],
        });
        events.push(MidiEvent {
            tick: checked_add(note_tick, duration)?,
            order: 0,
            bytes: vec![0x80, *note, 0],
        });
    }
    Ok(())
}

fn push_voiced_strum(
    events: &mut Vec<MidiEvent>,
    tick: u32,
    notes: &[u8],
    stroke: Stroke,
    duration: u32,
    settings: VoicedStrumSettings,
) -> Result<()> {
    let mut ordered = notes.to_vec();
    let (base_velocity, stroke_sweep) = match stroke {
        Stroke::Down => (settings.down_velocity, settings.down_sweep_ticks),
        Stroke::Up => (settings.up_velocity, settings.up_sweep_ticks),
    };
    if matches!(stroke, Stroke::Up) {
        ordered.reverse();
        if ordered.len() > settings.up_max_strings {
            ordered.truncate(settings.up_max_strings);
        }
    }
    let stroke_gap = strum_gap_ticks(stroke_sweep, ordered.len())?;

    for (index, note) in ordered.iter().enumerate() {
        let note_tick = checked_add(tick, checked_mul(to_u32(index)?, stroke_gap)?)?;
        let taper = u8::try_from(index).unwrap_or(u8::MAX).saturating_mul(3);
        let note_velocity = base_velocity.saturating_sub(taper).max(1);
        events.push(MidiEvent {
            tick: note_tick,
            order: 1,
            bytes: vec![0x90, *note, note_velocity],
        });
        events.push(MidiEvent {
            tick: checked_add(note_tick, duration)?,
            order: 0,
            bytes: vec![0x80, *note, 0],
        });
    }
    Ok(())
}

fn strum_gap_ticks(sweep_ticks: u32, note_count: usize) -> Result<u32> {
    if note_count <= 1 {
        return Ok(0);
    }
    let gaps = to_u32(note_count - 1)?;
    Ok((sweep_ticks / gaps).max(1))
}

fn chord_error(
    line: usize,
    chord: &str,
    voicing_set: Option<&str>,
    custom_voicings: Option<&VoicingLibrary>,
) -> AppError {
    if let Some(voicing_set) = voicing_set {
        match voicing::select_voicing(chord, voicing_set, custom_voicings) {
            Ok(_) => AppError::Validation(format!("Line {line}: unknown chord '{chord}'")),
            Err(err) => AppError::Validation(format!("Line {line}: {err}")),
        }
    } else {
        AppError::Validation(format!("Line {line}: unknown chord '{chord}'"))
    }
}

fn push_muted(events: &mut Vec<MidiEvent>, tick: u32, notes: &[u8], duration: u32) -> Result<()> {
    let muted_velocity = 25;
    let muted_duration = duration.min(30);
    for note in notes {
        events.push(MidiEvent {
            tick,
            order: 1,
            bytes: vec![0x90, *note, muted_velocity],
        });
        events.push(MidiEvent {
            tick: checked_add(tick, muted_duration)?,
            order: 0,
            bytes: vec![0x80, *note, 0],
        });
    }
    Ok(())
}

fn microseconds_per_quarter_note(tempo: u16) -> u32 {
    60_000_000 / u32::from(tempo)
}

fn marker_event(name: &str) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0x06];
    write_delta(&mut bytes, u32::try_from(name.len()).unwrap_or(u32::MAX));
    bytes.extend(name.as_bytes());
    bytes
}

fn denominator_power(denominator: u8) -> Result<u8> {
    if denominator == 0 || !denominator.is_power_of_two() {
        return Err(AppError::Validation(format!(
            "unsupported time signature denominator '{denominator}'"
        )));
    }
    Ok(denominator.ilog2() as u8)
}

fn to_u32(value: usize) -> Result<u32> {
    u32::try_from(value).map_err(|_| midi_range_error())
}

fn checked_add(left: u32, right: u32) -> Result<u32> {
    left.checked_add(right).ok_or_else(midi_range_error)
}

fn checked_mul(left: u32, right: u32) -> Result<u32> {
    left.checked_mul(right).ok_or_else(midi_range_error)
}

fn midi_range_error() -> AppError {
    AppError::Encoding("MIDI timing exceeds supported range".to_string())
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
    use crate::model::{Bar, BeatPattern, Metadata, Song, StrumSymbol, TimeSignature};
    use crate::{parser, validate};

    use super::*;

    #[test]
    fn writes_valid_midi_header_and_tempo() {
        let song = parser::parse("tempo: 120\ntime: 4/4\n\n| C\n| ---- ---- ---- ----\n").unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(
            &song,
            MidiOptions {
                velocity: Some(90),
                strum_spread_ms: Some(20),
                ..Default::default()
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
            "tempo: 120\ntime: 4/4\ninstrument: electric_guitar_clean\n\n| C\n| ---- ---- ---- ----\n",
        )
        .unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(
            &song,
            MidiOptions {
                velocity: Some(90),
                strum_spread_ms: Some(20),
                ..Default::default()
            },
        )
        .unwrap();

        assert!(midi.windows(2).any(|window| window == [0xC0, 27]));
    }

    #[test]
    fn uses_velocity_metadata_when_not_overridden() {
        let song =
            parser::parse("tempo: 92\ntime: 4/4\nvelocity: 64\n\n| C\n| D--- ---- ---- ----\n")
                .unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(&song, MidiOptions::default()).unwrap();

        assert!(midi.windows(3).any(|window| window == [0x90, 48, 64]));
    }

    #[test]
    fn writes_part_names_as_midi_markers() {
        let song =
            parser::parse("tempo: 92\ntime: 4/4\n\npart: verse\n| C\n| ---- ---- ---- ----\n")
                .unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(&song, MidiOptions::default()).unwrap();

        assert!(midi.windows(8).any(|window| window == b"\xFF\x06\x05verse"));
    }

    #[test]
    fn returns_error_for_unvalidated_missing_tempo() {
        let song = Song {
            metadata: Metadata {
                tempo: None,
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

        let err = write_midi(
            &song,
            MidiOptions {
                velocity: Some(90),
                strum_spread_ms: Some(20),
                ..Default::default()
            },
        )
        .unwrap_err();

        assert!(err.to_string().contains("missing tempo"));
    }

    #[test]
    fn returns_error_for_unvalidated_unknown_chord() {
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
            bars: vec![Bar {
                line: 1,
                beats: vec![
                    BeatPattern {
                        chord: "Hm".to_string(),
                        chord_line: 1,
                        slots: vec![
                            StrumSymbol::Down,
                            StrumSymbol::Rest,
                            StrumSymbol::Rest,
                            StrumSymbol::Rest,
                        ],
                    },
                    BeatPattern {
                        chord: "C".to_string(),
                        chord_line: 1,
                        slots: vec![StrumSymbol::Rest; 4],
                    },
                    BeatPattern {
                        chord: "C".to_string(),
                        chord_line: 1,
                        slots: vec![StrumSymbol::Rest; 4],
                    },
                    BeatPattern {
                        chord: "C".to_string(),
                        chord_line: 1,
                        slots: vec![StrumSymbol::Rest; 4],
                    },
                ],
            }],
        };

        let err = write_midi(
            &song,
            MidiOptions {
                velocity: Some(90),
                strum_spread_ms: Some(20),
                ..Default::default()
            },
        )
        .unwrap_err();

        assert!(err.to_string().contains("unknown chord"));
    }

    #[test]
    fn checked_tick_arithmetic_reports_overflow() {
        assert!(checked_add(u32::MAX, 1).is_err());
        assert!(checked_mul(u32::MAX, 2).is_err());
    }

    #[test]
    fn voiced_spread_is_total_sweep_length() {
        assert_eq!(strum_gap_ticks(120, 5).unwrap(), 30);
        assert_eq!(strum_gap_ticks(120, 4).unwrap(), 40);
        assert_eq!(strum_gap_ticks(120, 1).unwrap(), 0);
    }

    #[test]
    fn voiced_downstroke_notes_are_equally_spaced_across_total_sweep() {
        let song = parser::parse(
            "tempo: 120\ntime: 4/4\nvoicing: folk\ndownstroke_spread_ms: 100\n\n| C\n| D--- ---- ---- ----\n",
        )
        .unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(&song, MidiOptions::default()).unwrap();
        let ticks = note_on_ticks(&midi);

        assert_eq!(&ticks[..5], &[0, 24, 48, 72, 96]);
    }

    #[test]
    fn downstroke_orders_notes_low_to_high() {
        let song = parser::parse("tempo: 92\ntime: 4/4\n\n| C\n| D--- ---- ---- ----\n").unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(
            &song,
            MidiOptions {
                velocity: Some(90),
                strum_spread_ms: Some(20),
                ..Default::default()
            },
        )
        .unwrap();

        assert_note_on_order(&midi, &[48, 52, 55]);
    }

    #[test]
    fn upstroke_orders_notes_high_to_low() {
        let song = parser::parse("tempo: 92\ntime: 4/4\n\n| C\n| --U- ---- ---- ----\n").unwrap();
        validate::validate(&song).unwrap();

        let midi = write_midi(
            &song,
            MidiOptions {
                velocity: Some(90),
                strum_spread_ms: Some(20),
                ..Default::default()
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

    fn note_on_ticks(midi: &[u8]) -> Vec<u32> {
        let Some(track_start) = midi.windows(4).position(|window| window == b"MTrk") else {
            return Vec::new();
        };
        let mut index = track_start + 8;
        let mut tick = 0_u32;
        let mut ticks = Vec::new();

        while index < midi.len() {
            let delta = read_delta(midi, &mut index);
            tick += delta;
            if index >= midi.len() {
                break;
            }

            let status = midi[index];
            index += 1;
            match status {
                0x80..=0x9F => {
                    if index + 2 > midi.len() {
                        break;
                    }
                    let velocity = midi[index + 1];
                    if status == 0x90 && velocity > 0 {
                        ticks.push(tick);
                    }
                    index += 2;
                }
                0xC0..=0xDF => {
                    index += 1;
                }
                0xFF => {
                    if index + 1 >= midi.len() {
                        break;
                    }
                    let length = usize::from(midi[index + 1]);
                    index += 2 + length;
                }
                _ => break,
            }
        }

        ticks
    }

    fn read_delta(midi: &[u8], index: &mut usize) -> u32 {
        let mut value = 0_u32;
        while *index < midi.len() {
            let byte = midi[*index];
            *index += 1;
            value = (value << 7) | u32::from(byte & 0x7F);
            if byte & 0x80 == 0 {
                break;
            }
        }
        value
    }
}
