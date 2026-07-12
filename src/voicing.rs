// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, path::Path};

use crate::{
    error::{AppError, Result},
    harmony::{self, ChordHarmony, ChordQuality},
};

const STRING_BASE_NOTES: [u8; 6] = [40, 45, 50, 55, 59, 64];
const STRING_NAMES: [&str; 6] = ["E", "A", "D", "G", "B", "high-E"];
type FolkRootVoicings<'a> = (&'a [&'a str], &'a str, &'a str, &'a str, &'a str, &'a str);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Voicing {
    pub chord: String,
    pub id: String,
    pub frets: [Option<u8>; 6],
    pub priority: i16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VoicingLibrary {
    voicings: HashMap<String, Vec<Voicing>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedVoicing {
    pub voicing: Voicing,
    pub notes: Vec<StringNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringNote {
    pub string_index: usize,
    pub string_name: &'static str,
    pub midi_note: u8,
}

impl VoicingLibrary {
    pub fn insert(&mut self, voicing: Voicing) {
        self.voicings
            .entry(voicing.chord.clone())
            .or_default()
            .push(voicing);
    }

    pub fn merge(&mut self, other: Self) {
        for voicings in other.voicings.into_values() {
            for voicing in voicings {
                self.insert(voicing);
            }
        }
    }
}

pub fn built_in_library(name: &str) -> Result<VoicingLibrary> {
    let mut library = VoicingLibrary::default();
    match name {
        "folk" => {
            add_folk_chromatic_voicings(&mut library)?;
            library.insert(Voicing::new("C/E", "folk-slash", "032010", 100)?);
        }
        "rock" => {
            for (chord, frets) in [
                ("C5", "x355xx"),
                ("D5", "x577xx"),
                ("E5", "022xxx"),
                ("G5", "355xxx"),
                ("A5", "577xxx"),
                ("C", "x3555x"),
                ("Cm", "x3554x"),
            ] {
                library.insert(Voicing::new(chord, "rock", frets, 100)?);
            }
        }
        other => {
            return Err(AppError::Validation(format!(
                "unsupported voicing set '{other}'"
            )));
        }
    }
    Ok(library)
}

fn add_folk_chromatic_voicings(library: &mut VoicingLibrary) -> Result<()> {
    const ROOTS: &[FolkRootVoicings<'_>] = &[
        (
            &["C", "B#"],
            "x32010",
            "x35543",
            "x32313",
            "x32000",
            "x35343",
        ),
        (
            &["C#", "Db"],
            "x46664",
            "x46654",
            "x46464",
            "x46564",
            "x46454",
        ),
        (&["D"], "xx0232", "xx0231", "xx0212", "xx0222", "xx0211"),
        (
            &["D#", "Eb"],
            "x68886",
            "x68876",
            "x68686",
            "x68786",
            "x68676",
        ),
        (
            &["E", "Fb"],
            "022100",
            "022000",
            "020100",
            "021100",
            "022030",
        ),
        (
            &["F", "E#"],
            "133211",
            "133111",
            "131211",
            "133210",
            "131111",
        ),
        (
            &["F#", "Gb"],
            "244322",
            "244222",
            "242322",
            "243322",
            "242222",
        ),
        (&["G"], "320003", "355333", "320001", "320002", "353333"),
        (
            &["G#", "Ab"],
            "466544",
            "466444",
            "464544",
            "465544",
            "464444",
        ),
        (&["A"], "x02220", "x02210", "x02020", "x02120", "x02010"),
        (
            &["A#", "Bb"],
            "x13331",
            "x13321",
            "x13131",
            "x13231",
            "x13121",
        ),
        (
            &["B", "Cb"],
            "x24442",
            "x24432",
            "x21202",
            "x24342",
            "x20202",
        ),
    ];

    for (roots, major, minor, dominant7, major7, minor7) in ROOTS {
        for root in *roots {
            library.insert(Voicing::new(root, "folk-major", major, 100)?);
            library.insert(Voicing::new(&format!("{root}m"), "folk-minor", minor, 100)?);
            library.insert(Voicing::new(
                &format!("{root}7"),
                "folk-dominant7",
                dominant7,
                100,
            )?);
            library.insert(Voicing::new(
                &format!("{root}maj7"),
                "folk-major7",
                major7,
                90,
            )?);
            library.insert(Voicing::new(
                &format!("{root}m7"),
                "folk-minor7",
                minor7,
                90,
            )?);
        }
    }

    Ok(())
}

pub fn load_custom_file(path: &Path) -> Result<VoicingLibrary> {
    let content = std::fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse_custom(content.as_str())
}

pub fn parse_custom(content: &str) -> Result<VoicingLibrary> {
    let mut library = VoicingLibrary::default();
    let mut current_chord = None::<String>;
    let mut current_id = None::<String>;
    let mut current_frets = None::<[Option<u8>; 6]>;
    let mut current_priority = 0_i16;

    for (index, raw_line) in content.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty()
            || line == "voicings:"
            || line.starts_with("name:")
            || line.starts_with("tags:")
        {
            continue;
        }

        if raw_line.starts_with("  ") && !raw_line.starts_with("    ") && line.ends_with(':') {
            flush_custom(
                &mut library,
                &current_chord,
                &mut current_id,
                &mut current_frets,
                current_priority,
            )?;
            current_chord = Some(line.trim_end_matches(':').to_string());
            current_priority = 0;
        } else if let Some(value) = line.strip_prefix("- id:") {
            flush_custom(
                &mut library,
                &current_chord,
                &mut current_id,
                &mut current_frets,
                current_priority,
            )?;
            current_id = Some(value.trim().to_string());
            current_priority = 0;
        } else if let Some(value) = line.strip_prefix("id:") {
            current_id = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("frets:") {
            current_frets = Some(parse_frets(value.trim(), line_number)?);
        } else if let Some(value) = line.strip_prefix("priority:") {
            current_priority = value.trim().parse().map_err(|_| {
                AppError::Parse(format!("Line {line_number}: malformed voicing priority"))
            })?;
        }
    }

    flush_custom(
        &mut library,
        &current_chord,
        &mut current_id,
        &mut current_frets,
        current_priority,
    )?;
    Ok(library)
}

pub fn select_voicing(
    symbol: &str,
    set_name: &str,
    custom: Option<&VoicingLibrary>,
) -> Result<RenderedVoicing> {
    let harmony = harmony::resolve(symbol).map_err(|err| {
        AppError::Validation(format!(
            "{} while selecting voicing",
            err.to_string().trim()
        ))
    })?;
    let mut library = built_in_library(set_name)?;
    if let Some(custom) = custom {
        library.merge(custom.clone());
    }

    let candidates = library
        .voicings
        .get(symbol)
        .into_iter()
        .flatten()
        .filter_map(|voicing| {
            validate_voicing(&harmony, voicing)
                .ok()
                .map(|notes| (voicing, notes))
        })
        .collect::<Vec<_>>();

    let Some((voicing, notes)) = candidates
        .into_iter()
        .max_by_key(|(voicing, _)| (voicing.priority, std::cmp::Reverse(voicing.id.clone())))
    else {
        return Err(AppError::Validation(format!(
            "chord '{}' is valid, but no compatible '{set_name}' voicing was found",
            harmony.symbol
        )));
    };

    Ok(RenderedVoicing {
        voicing: voicing.clone(),
        notes,
    })
}

pub fn validate_voicing(harmony: &ChordHarmony, voicing: &Voicing) -> Result<Vec<StringNote>> {
    let notes = rendered_notes(voicing);
    if notes.is_empty() {
        return Err(AppError::Validation(
            "voicing has no sounding strings".to_string(),
        ));
    }
    let pcs = notes
        .iter()
        .map(|note| note.midi_note % 12)
        .collect::<Vec<_>>();

    let allowed = harmony
        .intervals
        .iter()
        .map(|interval| (harmony.root_pc + interval) % 12)
        .collect::<Vec<_>>();
    for required in &allowed {
        if !pcs.contains(required) {
            return Err(AppError::Validation(format!(
                "voicing for '{}' is missing required chord tone",
                harmony.symbol
            )));
        }
    }
    if harmony.quality == ChordQuality::Power {
        let major_third = (harmony.root_pc + 4) % 12;
        let minor_third = (harmony.root_pc + 3) % 12;
        if pcs.contains(&major_third) || pcs.contains(&minor_third) {
            return Err(AppError::Validation(
                "power chord voicing must not contain a third".to_string(),
            ));
        }
    }
    for pc in &pcs {
        if !allowed.contains(pc) {
            return Err(AppError::Validation(format!(
                "voicing for '{}' contains a non-chord tone",
                harmony.symbol
            )));
        }
    }
    if let Some(bass) = &harmony.bass {
        let lowest = notes
            .first()
            .map(|note| note.midi_note % 12)
            .ok_or_else(|| AppError::Validation("voicing has no sounding strings".to_string()))?;
        if lowest != bass.pc {
            return Err(AppError::Validation(format!(
                "slash chord '{}' expected {} as the lowest sounding note",
                harmony.symbol, bass.name
            )));
        }
    }

    Ok(notes)
}

pub fn rendered_notes(voicing: &Voicing) -> Vec<StringNote> {
    voicing
        .frets
        .iter()
        .enumerate()
        .filter_map(|(string_index, fret)| {
            fret.map(|fret| StringNote {
                string_index,
                string_name: STRING_NAMES[string_index],
                midi_note: STRING_BASE_NOTES[string_index] + fret,
            })
        })
        .collect()
}

fn parse_frets(value: &str, line_number: usize) -> Result<[Option<u8>; 6]> {
    let body = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| AppError::Parse(format!("Line {line_number}: malformed frets")))?;
    let parts = body.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 6 {
        return Err(AppError::Parse(format!(
            "Line {line_number}: expected 6 fret values"
        )));
    }
    let mut frets = [None; 6];
    for (index, part) in parts.iter().enumerate() {
        if *part == "x" {
            continue;
        }
        let fret = part
            .parse::<u8>()
            .map_err(|_| AppError::Parse(format!("Line {line_number}: malformed fret value")))?;
        if fret > 24 {
            return Err(AppError::Validation(format!(
                "Line {line_number}: unsupported fret value {fret}"
            )));
        }
        frets[index] = Some(fret);
    }
    Ok(frets)
}

fn flush_custom(
    library: &mut VoicingLibrary,
    chord: &Option<String>,
    id: &mut Option<String>,
    frets: &mut Option<[Option<u8>; 6]>,
    priority: i16,
) -> Result<()> {
    let (Some(chord), Some(id), Some(frets)) = (chord, id.take(), frets.take()) else {
        return Ok(());
    };
    library.insert(Voicing {
        chord: chord.clone(),
        id,
        frets,
        priority,
    });
    Ok(())
}

impl Voicing {
    pub fn new(chord: &str, id: &str, shape: &str, priority: i16) -> Result<Self> {
        let mut frets = [None; 6];
        if shape.chars().count() != 6 {
            return Err(AppError::Validation(format!(
                "voicing '{shape}' must contain 6 string values"
            )));
        }
        for (index, character) in shape.chars().enumerate() {
            frets[index] = match character {
                'x' => None,
                '0'..='9' => Some(character as u8 - b'0'),
                _ => {
                    return Err(AppError::Validation(format!(
                        "unsupported fret value '{character}'"
                    )));
                }
            };
        }
        Ok(Self {
            chord: chord.to_string(),
            id: id.to_string(),
            frets,
            priority,
        })
    }

    pub fn shape(&self) -> String {
        self.frets
            .iter()
            .map(|fret| fret.map_or('x', |fret| char::from(b'0' + fret)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_built_in_folk_and_rock_shapes() {
        assert_eq!(
            select_voicing("C", "folk", None).unwrap().voicing.shape(),
            "x32010"
        );
        assert_eq!(
            select_voicing("C5", "rock", None).unwrap().voicing.shape(),
            "x355xx"
        );
    }

    #[test]
    fn folk_set_covers_major_minor_and_seventh_chords_for_chromatic_roots() {
        let roots = [
            "C", "B#", "C#", "Db", "D", "D#", "Eb", "E", "Fb", "F", "E#", "F#", "Gb", "G", "G#",
            "Ab", "A", "A#", "Bb", "B", "Cb",
        ];
        for root in roots {
            for suffix in ["", "m", "7", "maj7", "m7"] {
                let symbol = format!("{root}{suffix}");
                select_voicing(&symbol, "folk", None)
                    .unwrap_or_else(|err| panic!("{symbol} should have a folk voicing: {err}"));
            }
        }
    }

    #[test]
    fn validates_power_chord_without_third() {
        let harmony = harmony::resolve("C5").unwrap();
        let valid = Voicing::new("C5", "valid", "x355xx", 100).unwrap();
        assert!(validate_voicing(&harmony, &valid).is_ok());

        let invalid = Voicing::new("C5", "invalid", "x32010", 100).unwrap();
        assert!(
            validate_voicing(&harmony, &invalid)
                .unwrap_err()
                .to_string()
                .contains("third")
        );
    }

    #[test]
    fn validates_slash_bass() {
        let harmony = harmony::resolve("C/E").unwrap();
        let valid = Voicing::new("C/E", "slash", "032010", 100).unwrap();
        assert!(validate_voicing(&harmony, &valid).is_ok());

        let invalid = Voicing::new("C/E", "wrong", "x32010", 100).unwrap();
        assert!(
            validate_voicing(&harmony, &invalid)
                .unwrap_err()
                .to_string()
                .contains("expected E")
        );
    }

    #[test]
    fn parses_custom_file_and_rejects_bad_frets() {
        let custom = parse_custom(
            "name: custom\nvoicings:\n  C:\n    - id: preferred-c\n      frets: [x, 3, 2, 0, 1, 0]\n      priority: 200\n",
        )
        .unwrap();
        assert_eq!(
            select_voicing("C", "folk", Some(&custom))
                .unwrap()
                .voicing
                .id,
            "preferred-c"
        );

        let err = parse_custom(
            "name: custom\nvoicings:\n  C:\n    - id: bad\n      frets: [x, 29, 2, 0, 1, 0]\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("fret value 29"));
    }
}
