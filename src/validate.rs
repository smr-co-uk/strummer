use crate::{
    chord,
    error::{AppError, Result},
    model::{CountStyle, Pulse, Song, TimeSignature},
    timing,
};

pub fn validate(song: &Song) -> Result<()> {
    if song.metadata.tempo.is_none() {
        return Err(AppError::Validation("missing tempo".to_string()));
    }
    let Some(time_signature) = song.metadata.time_signature else {
        return Err(AppError::Validation("missing time signature".to_string()));
    };
    if song.bars.is_empty() {
        return Err(AppError::Validation("empty input file".to_string()));
    }

    let pulse = resolved_pulse(song, time_signature)?;
    let subdivision = resolved_subdivision(song)?;
    validate_count(song.metadata.count, pulse, subdivision)?;
    let expected_beats = expected_beat_patterns(time_signature, pulse)?;
    let expected_slots = timing::slots_per_pulse(pulse, subdivision)?;
    for bar in &song.bars {
        if bar.beats.len() != expected_beats {
            return Err(AppError::Validation(format!(
                "Line {}: expected {expected_beats} beat patterns, found {}",
                bar.line,
                bar.beats.len()
            )));
        }
        for beat in &bar.beats {
            if chord::notes_for_chord(&beat.chord).is_none() {
                return Err(AppError::Validation(format!(
                    "Line {}: unknown chord '{}'",
                    beat.chord_line, beat.chord
                )));
            }
            if beat.slots.len() != expected_slots {
                return Err(AppError::Validation(format!(
                    "Line {}: expected {expected_slots} slots per beat pattern, found {}",
                    bar.line,
                    beat.slots.len()
                )));
            }
        }
    }

    Ok(())
}

fn resolved_pulse(song: &Song, time_signature: TimeSignature) -> Result<Pulse> {
    if let Some(pulse) = song.metadata.pulse {
        return Ok(pulse);
    }

    if time_signature.denominator == 8 {
        return Err(AppError::Validation("missing pulse".to_string()));
    }

    Ok(Pulse::Quarter)
}

fn resolved_subdivision(song: &Song) -> Result<u8> {
    let subdivision = song.metadata.subdivision.unwrap_or(16);
    match subdivision {
        8 | 16 => Ok(subdivision),
        _ => Err(AppError::Validation(format!(
            "unsupported subdivision '{subdivision}'"
        ))),
    }
}

fn validate_count(count: Option<CountStyle>, pulse: Pulse, subdivision: u8) -> Result<()> {
    let Some(count) = count else {
        return Ok(());
    };

    let valid = matches!(
        (count, pulse, subdivision),
        (CountStyle::OneAnd, Pulse::Quarter, 8)
            | (CountStyle::OneAndA, Pulse::DottedQuarter, 8)
            | (CountStyle::OneEAndA, Pulse::Quarter, 16)
            | (CountStyle::OneAAndA, Pulse::Quarter, 16)
    );

    if valid {
        Ok(())
    } else {
        Err(AppError::Validation(
            "unsupported count for pulse and subdivision".to_string(),
        ))
    }
}

fn expected_beat_patterns(time_signature: TimeSignature, pulse: Pulse) -> Result<usize> {
    match (pulse, time_signature.numerator, time_signature.denominator) {
        (Pulse::Quarter, numerator, 4) if numerator > 0 => Ok(usize::from(numerator)),
        (Pulse::DottedQuarter, numerator, 8) if numerator > 0 && numerator % 3 == 0 => {
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
    use crate::parser;

    use super::*;

    #[test]
    fn accepts_three_four_with_three_quarter_pulses() {
        let song =
            parser::parse("tempo: 92\ntime: 3/4\nsubdivision: 8\ncount: 1&\n\nC\nDU DU DU\n")
                .unwrap();

        validate(&song).unwrap();
    }

    #[test]
    fn accepts_six_eight_with_two_dotted_quarter_pulses() {
        let song = parser::parse(
            "tempo: 72\ntime: 6/8\npulse: dotted-quarter\nsubdivision: 8\ncount: 1&a\n\nC\nD-U D-U\n",
        )
        .unwrap();

        validate(&song).unwrap();
    }

    #[test]
    fn rejects_six_eight_without_pulse() {
        let song =
            parser::parse("tempo: 72\ntime: 6/8\nsubdivision: 8\ncount: 1&a\n\nC\nD-U D-U\n")
                .unwrap();

        assert!(
            validate(&song)
                .unwrap_err()
                .to_string()
                .contains("missing pulse")
        );
    }
}
