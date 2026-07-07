use crate::{
    error::{AppError, Result},
    model::{Bar, Beat, BeatPattern, CountStyle, Metadata, Song, StrumSymbol, TimeSignature},
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChordMarker {
    column: usize,
    chord: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PatternTokenKind {
    Pattern(String),
    Bar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PatternToken {
    column: usize,
    kind: PatternTokenKind,
}

pub fn parse(input: &str) -> Result<Song> {
    if input.trim().is_empty() {
        return Err(AppError::Validation("empty input file".to_string()));
    }

    let mut metadata = Metadata::default();
    let mut bars = Vec::new();
    let mut chart_lines = Vec::new();

    for (index, raw_line) in input.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if bars.is_empty() && chart_lines.is_empty() && looks_like_metadata(line) {
            parse_metadata_line(line, line_number, &mut metadata)?;
        } else if bars.is_empty() && chart_lines.is_empty() && looks_like_malformed_metadata(line) {
            return Err(AppError::Parse(format!(
                "Line {line_number}: malformed metadata line"
            )));
        } else {
            chart_lines.push((line_number, raw_line.to_string()));
        }
    }

    if chart_lines.len() % 2 != 0 {
        let (line_number, _) = chart_lines.last().expect("odd chart line count has last");
        return Err(AppError::Parse(format!(
            "Line {line_number}: expected strum pattern line after chord line"
        )));
    }

    for pair in chart_lines.chunks(2) {
        parse_chart_pair(
            pair[0].0,
            pair[0].1.as_str(),
            pair[1].0,
            pair[1].1.as_str(),
            &mut bars,
        )?;
    }

    Ok(Song { metadata, bars })
}

fn looks_like_metadata(line: &str) -> bool {
    let key = line.split(':').next().unwrap_or_default().trim();
    matches!(key, "tempo" | "time" | "beat" | "subdivision" | "count")
}

fn looks_like_malformed_metadata(line: &str) -> bool {
    let key = line.split_whitespace().next().unwrap_or_default();
    matches!(key, "tempo" | "time" | "beat" | "subdivision" | "count") && !line.contains(':')
}

fn parse_metadata_line(line: &str, line_number: usize, metadata: &mut Metadata) -> Result<()> {
    let Some((key, value)) = line.split_once(':') else {
        return Err(AppError::Parse(format!(
            "Line {line_number}: malformed metadata line"
        )));
    };
    let key = key.trim();
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::Parse(format!(
            "Line {line_number}: malformed metadata line"
        )));
    }

    match key {
        "tempo" => {
            metadata.tempo = Some(value.parse().map_err(|_| {
                AppError::Parse(format!("Line {line_number}: malformed metadata line"))
            })?);
        }
        "time" => {
            let Some((numerator, denominator)) = value.split_once('/') else {
                return Err(AppError::Parse(format!(
                    "Line {line_number}: malformed metadata line"
                )));
            };
            metadata.time_signature = Some(TimeSignature {
                numerator: numerator.trim().parse().map_err(|_| {
                    AppError::Parse(format!("Line {line_number}: malformed metadata line"))
                })?,
                denominator: denominator.trim().parse().map_err(|_| {
                    AppError::Parse(format!("Line {line_number}: malformed metadata line"))
                })?,
            });
        }
        "beat" => {
            metadata.beat = Some(match value {
                "quarter" => Beat::Quarter,
                "dotted-quarter" => Beat::DottedQuarter,
                _ => {
                    return Err(AppError::Validation(format!(
                        "Line {line_number}: unsupported beat '{value}'"
                    )));
                }
            });
        }
        "subdivision" => {
            metadata.subdivision = Some(value.parse().map_err(|_| {
                AppError::Parse(format!("Line {line_number}: malformed metadata line"))
            })?);
        }
        "count" => {
            metadata.count = Some(match value {
                "1&" => CountStyle::OneAnd,
                "1&a" => CountStyle::OneAndA,
                "1e&a" => CountStyle::OneEAndA,
                "1a&a" => CountStyle::OneAAndA,
                _ => {
                    return Err(AppError::Validation(format!(
                        "Line {line_number}: unsupported count '{value}'"
                    )));
                }
            });
        }
        _ => {
            return Err(AppError::Parse(format!(
                "Line {line_number}: malformed metadata line"
            )));
        }
    }

    Ok(())
}

fn parse_chart_pair(
    chord_line_number: usize,
    chord_line: &str,
    pattern_line_number: usize,
    pattern_line: &str,
    bars: &mut Vec<Bar>,
) -> Result<()> {
    let chord_markers = parse_chord_markers(chord_line);
    if chord_markers.is_empty() {
        return Err(AppError::Parse(format!(
            "Line {chord_line_number}: expected chord markers"
        )));
    }

    let tokens = parse_pattern_tokens(pattern_line, pattern_line_number)?;
    if tokens.is_empty() {
        return Err(AppError::Parse(format!(
            "Line {pattern_line_number}: expected strum pattern"
        )));
    }

    let mut current_chord = bars
        .last()
        .and_then(|bar| bar.beats.last())
        .map(|beat| (beat.chord.clone(), beat.chord_line));
    let mut marker_index = 0;
    let mut current_beats = Vec::new();

    for token in tokens {
        while marker_index < chord_markers.len()
            && chord_markers[marker_index].column <= token.column
        {
            current_chord = Some((chord_markers[marker_index].chord.clone(), chord_line_number));
            marker_index += 1;
        }

        match token.kind {
            PatternTokenKind::Bar => finish_bar(&mut current_beats, pattern_line_number, bars),
            PatternTokenKind::Pattern(pattern) if pattern == "..." => {
                if !current_beats.is_empty() {
                    return Err(AppError::Parse(format!(
                        "Line {pattern_line_number}: repeat marker must replace the whole bar pattern"
                    )));
                }
                let Some(previous) = bars.last() else {
                    return Err(AppError::Parse(format!(
                        "Line {pattern_line_number}: repeat marker requires a previous bar"
                    )));
                };
                let mut repeated = previous.clone();
                repeated.line = pattern_line_number;
                if let Some((chord, chord_line)) = current_chord.clone() {
                    for beat in &mut repeated.beats {
                        beat.chord.clone_from(&chord);
                        beat.chord_line = chord_line;
                    }
                }
                bars.push(repeated);
            }
            PatternTokenKind::Pattern(pattern) => {
                let Some((chord, chord_line)) = current_chord.clone() else {
                    return Err(AppError::Parse(format!(
                        "Line {pattern_line_number}: missing chord marker for strum pattern"
                    )));
                };
                current_beats.push(BeatPattern {
                    chord,
                    chord_line,
                    slots: parse_pattern(pattern.as_str(), pattern_line_number)?,
                });
            }
        }
    }

    finish_bar(&mut current_beats, pattern_line_number, bars);
    Ok(())
}

fn finish_bar(beats: &mut Vec<BeatPattern>, line_number: usize, bars: &mut Vec<Bar>) {
    if !beats.is_empty() {
        bars.push(Bar {
            line: line_number,
            beats: std::mem::take(beats),
        });
    }
}

fn parse_chord_markers(line: &str) -> Vec<ChordMarker> {
    line.char_indices()
        .filter(|(_, character)| !character.is_whitespace())
        .fold(Vec::new(), |mut markers, (column, _)| {
            let starts_token = column == 0
                || line[..column]
                    .chars()
                    .next_back()
                    .is_none_or(char::is_whitespace);
            if starts_token {
                let chord = line[column..]
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string();
                markers.push(ChordMarker { column, chord });
            }
            markers
        })
}

fn parse_pattern_tokens(line: &str, line_number: usize) -> Result<Vec<PatternToken>> {
    let mut tokens = Vec::new();
    let mut token_start = None;

    for (column, character) in line.char_indices() {
        match character {
            '|' => {
                if let Some(start) = token_start.take() {
                    tokens.push(pattern_token(start, &line[start..column]));
                }
                tokens.push(PatternToken {
                    column,
                    kind: PatternTokenKind::Bar,
                });
            }
            character if character.is_whitespace() => {
                if let Some(start) = token_start.take() {
                    tokens.push(pattern_token(start, &line[start..column]));
                }
            }
            _ => {
                token_start.get_or_insert(column);
            }
        }
    }

    if let Some(start) = token_start {
        tokens.push(pattern_token(start, &line[start..]));
    }

    if tokens.iter().any(
        |token| matches!(&token.kind, PatternTokenKind::Pattern(pattern) if pattern.is_empty()),
    ) {
        return Err(AppError::Parse(format!(
            "Line {line_number}: expected strum pattern"
        )));
    }

    Ok(tokens)
}

fn pattern_token(column: usize, token: &str) -> PatternToken {
    PatternToken {
        column,
        kind: PatternTokenKind::Pattern(token.to_string()),
    }
}

fn parse_pattern(pattern: &str, line_number: usize) -> Result<Vec<StrumSymbol>> {
    let mut slots = Vec::new();
    for symbol in pattern.chars() {
        slots.push(match symbol {
            'D' => StrumSymbol::Down,
            'U' => StrumSymbol::Up,
            '-' => StrumSymbol::Rest,
            'X' => StrumSymbol::Muted,
            other => {
                return Err(AppError::Validation(format!(
                    "Line {line_number}: invalid strum symbol '{other}'"
                )));
            }
        });
    }

    Ok(slots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_metadata_and_chart_pair() {
        let song =
            parse("tempo: 92\ntime: 4/4\nbeat: quarter\nsubdivision: 8\ncount: 1&\n\nC            Am\nDU DU DU DU | ...\n")
                .unwrap();

        assert_eq!(song.metadata.tempo, Some(92));
        assert_eq!(
            song.metadata.time_signature,
            Some(TimeSignature {
                numerator: 4,
                denominator: 4
            })
        );
        assert_eq!(song.metadata.beat, Some(Beat::Quarter));
        assert_eq!(song.metadata.subdivision, Some(8));
        assert_eq!(song.metadata.count, Some(CountStyle::OneAnd));
        assert_eq!(song.bars.len(), 2);
        assert_eq!(song.bars[0].beats[0].chord, "C");
        assert_eq!(song.bars[1].beats[0].chord, "Am");
        assert_eq!(song.bars[0].beats[0].slots, song.bars[1].beats[0].slots);
    }

    #[test]
    fn supports_chord_change_inside_bar() {
        let song = parse("tempo: 92\ntime: 4/4\n\nC         G\nD--- D-U- --U- D-U-\n").unwrap();

        assert_eq!(song.bars.len(), 1);
        assert_eq!(song.bars[0].beats[0].chord, "C");
        assert_eq!(song.bars[0].beats[2].chord, "G");
    }

    #[test]
    fn reports_invalid_symbol() {
        let err = parse("tempo: 92\ntime: 4/4\n\nC\nD-Z- D-U- --U- D-U-\n").unwrap_err();
        assert!(err.to_string().contains("invalid strum symbol 'Z'"));
    }
}
