use crate::{
    error::{AppError, Result},
    model::{Pulse, Song, TimeSignature},
};

pub const PPQ: u16 = 480;

pub fn slot_ticks(song: &Song) -> Result<u32> {
    let pulse = resolved_pulse(song);
    let subdivision = resolved_subdivision(song);
    Ok(
        pulse_ticks(pulse)
            / u32::try_from(slots_per_pulse(pulse, subdivision)?).unwrap_or(u32::MAX),
    )
}

pub fn bar_ticks(song: &Song) -> Result<u32> {
    let time_signature = song
        .metadata
        .time_signature
        .expect("validated song has time");
    Ok(pulse_ticks(resolved_pulse(song))
        * u32::try_from(pulses_per_bar(time_signature, resolved_pulse(song))?).unwrap_or(u32::MAX))
}

pub fn slots_per_pulse(pulse: Pulse, subdivision: u8) -> Result<usize> {
    match (pulse, subdivision) {
        (Pulse::Quarter, 8) => Ok(2),
        (Pulse::Quarter, 16) => Ok(4),
        (Pulse::DottedQuarter, 8) => Ok(3),
        (Pulse::DottedQuarter, 16) => Ok(6),
        (_, other) => Err(AppError::Validation(format!(
            "unsupported subdivision '{other}'"
        ))),
    }
}

pub fn ms_to_ticks(ms: u16, tempo_bpm: u16) -> u32 {
    let ticks = u64::from(ms) * u64::from(tempo_bpm) * u64::from(PPQ) / 60_000;
    u32::try_from(ticks.max(1)).unwrap_or(u32::MAX)
}

fn resolved_pulse(song: &Song) -> Pulse {
    song.metadata.pulse.unwrap_or(Pulse::Quarter)
}

fn resolved_subdivision(song: &Song) -> u8 {
    song.metadata.subdivision.unwrap_or(16)
}

fn pulse_ticks(pulse: Pulse) -> u32 {
    match pulse {
        Pulse::Quarter => u32::from(PPQ),
        Pulse::DottedQuarter => u32::from(PPQ) * 3 / 2,
    }
}

fn pulses_per_bar(time_signature: TimeSignature, pulse: Pulse) -> Result<usize> {
    match (pulse, time_signature.numerator, time_signature.denominator) {
        (Pulse::Quarter, numerator, 4) => Ok(usize::from(numerator)),
        (Pulse::DottedQuarter, numerator, 8) if numerator % 3 == 0 => {
            Ok(usize::from(numerator / 3))
        }
        _ => Err(AppError::Validation(format!(
            "unsupported time signature {}/{} with selected pulse",
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
                pulse: None,
                subdivision: None,
                count: None,
            },
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
                pulse: Some(Pulse::DottedQuarter),
                subdivision: Some(8),
                count: None,
            },
            bars: Vec::new(),
        };

        assert_eq!(slot_ticks(&song).unwrap(), 240);
        assert_eq!(bar_ticks(&song).unwrap(), u32::from(PPQ) * 3);
    }
}
