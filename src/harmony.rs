// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chord,
    error::{AppError, Result},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChordHarmony {
    pub symbol: String,
    pub root: String,
    pub root_pc: u8,
    pub quality: ChordQuality,
    pub intervals: Vec<u8>,
    pub bass: Option<BassNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BassNote {
    pub name: String,
    pub pc: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordQuality {
    Major,
    Minor,
    Power,
    Sus2,
    Sus4,
    Diminished,
    Augmented,
    Six,
    MinorSix,
    Dominant7,
    Major7,
    Minor7,
    MinorMajor7,
    Nine,
    Minor9,
    Add9,
    MinorAdd9,
    Eleven,
    Thirteen,
}

impl ChordQuality {
    pub fn name(self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Power => "power",
            Self::Sus2 => "sus2",
            Self::Sus4 => "sus4",
            Self::Diminished => "diminished",
            Self::Augmented => "augmented",
            Self::Six => "6",
            Self::MinorSix => "minor6",
            Self::Dominant7 => "dominant7",
            Self::Major7 => "major7",
            Self::Minor7 => "minor7",
            Self::MinorMajor7 => "minor-major7",
            Self::Nine => "9",
            Self::Minor9 => "minor9",
            Self::Add9 => "add9",
            Self::MinorAdd9 => "minor-add9",
            Self::Eleven => "11",
            Self::Thirteen => "13",
        }
    }
}

pub fn resolve(symbol: &str) -> Result<ChordHarmony> {
    let (main, bass) = symbol
        .split_once('/')
        .map_or((symbol, None), |(main, bass)| (main, Some(bass)));
    let (root, suffix) = split_root(main).ok_or_else(|| unsupported(symbol))?;
    let root_pc = pitch_class(root).ok_or_else(|| unsupported(symbol))?;
    let quality = parse_quality(suffix).ok_or_else(|| unsupported(symbol))?;
    let bass = bass
        .map(|bass| {
            let pc = pitch_class(bass).ok_or_else(|| unsupported(symbol))?;
            Ok(BassNote {
                name: bass.to_string(),
                pc,
            })
        })
        .transpose()?;

    Ok(ChordHarmony {
        symbol: symbol.to_string(),
        root: root.to_string(),
        root_pc,
        quality,
        intervals: intervals_for_quality(quality),
        bass,
    })
}

pub fn pitch_class(note: &str) -> Option<u8> {
    chord::root_pitch_class(note)
}

fn split_root(symbol: &str) -> Option<(&str, &str)> {
    let mut chars = symbol.char_indices();
    let (_, first) = chars.next()?;
    if !matches!(first, 'A'..='G') {
        return None;
    }
    let end = match chars.next() {
        Some((index, '#' | 'b')) => index + 1,
        Some((index, _)) => index,
        None => symbol.len(),
    };
    Some(symbol.split_at(end))
}

fn parse_quality(suffix: &str) -> Option<ChordQuality> {
    Some(match suffix {
        "" => ChordQuality::Major,
        "m" => ChordQuality::Minor,
        "5" => ChordQuality::Power,
        "sus2" => ChordQuality::Sus2,
        "sus4" => ChordQuality::Sus4,
        "dim" | "diminished" => ChordQuality::Diminished,
        "aug" | "augmented" => ChordQuality::Augmented,
        "6" => ChordQuality::Six,
        "m6" => ChordQuality::MinorSix,
        "7" => ChordQuality::Dominant7,
        "maj7" => ChordQuality::Major7,
        "m7" => ChordQuality::Minor7,
        "mMaj7" | "mmaj7" => ChordQuality::MinorMajor7,
        "9" | "7b9" | "7#9" => ChordQuality::Nine,
        "m9" => ChordQuality::Minor9,
        "add9" => ChordQuality::Add9,
        "madd9" | "m-add9" => ChordQuality::MinorAdd9,
        "11" | "9#11" => ChordQuality::Eleven,
        "13" | "13b9" | "13#9" | "13b13" => ChordQuality::Thirteen,
        _ => return None,
    })
}

fn intervals_for_quality(quality: ChordQuality) -> Vec<u8> {
    match quality {
        ChordQuality::Major => vec![0, 4, 7],
        ChordQuality::Minor => vec![0, 3, 7],
        ChordQuality::Power => vec![0, 7],
        ChordQuality::Sus2 => vec![0, 2, 7],
        ChordQuality::Sus4 => vec![0, 5, 7],
        ChordQuality::Diminished => vec![0, 3, 6],
        ChordQuality::Augmented => vec![0, 4, 8],
        ChordQuality::Six => vec![0, 4, 7, 9],
        ChordQuality::MinorSix => vec![0, 3, 7, 9],
        ChordQuality::Dominant7 => vec![0, 4, 7, 10],
        ChordQuality::Major7 => vec![0, 4, 7, 11],
        ChordQuality::Minor7 => vec![0, 3, 7, 10],
        ChordQuality::MinorMajor7 => vec![0, 3, 7, 11],
        ChordQuality::Nine => vec![0, 4, 7, 10, 2],
        ChordQuality::Minor9 => vec![0, 3, 7, 10, 2],
        ChordQuality::Add9 => vec![0, 4, 7, 2],
        ChordQuality::MinorAdd9 => vec![0, 3, 7, 2],
        ChordQuality::Eleven => vec![0, 4, 7, 10, 2, 5],
        ChordQuality::Thirteen => vec![0, 4, 7, 10, 2, 5, 9],
    }
}

fn unsupported(symbol: &str) -> AppError {
    AppError::Validation(format!("unsupported chord symbol '{symbol}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_conventional_symbols() {
        assert_eq!(resolve("C").unwrap().intervals, vec![0, 4, 7]);
        assert_eq!(resolve("Cm").unwrap().intervals, vec![0, 3, 7]);
        assert_eq!(resolve("C5").unwrap().intervals, vec![0, 7]);
        assert_eq!(resolve("Csus2").unwrap().intervals, vec![0, 2, 7]);
        assert_eq!(resolve("Csus4").unwrap().intervals, vec![0, 5, 7]);
        assert_eq!(resolve("C7").unwrap().intervals, vec![0, 4, 7, 10]);
        assert_eq!(resolve("Cmaj7").unwrap().intervals, vec![0, 4, 7, 11]);
        assert_eq!(resolve("Cm7").unwrap().intervals, vec![0, 3, 7, 10]);
        assert_eq!(resolve("Cadd9").unwrap().intervals, vec![0, 4, 7, 2]);
    }

    #[test]
    fn resolves_accidentals_and_slash_bass() {
        let sharp = resolve("F#m").unwrap();
        assert_eq!(sharp.root, "F#");
        assert_eq!(sharp.quality.name(), "minor");

        let flat = resolve("Bb7").unwrap();
        assert_eq!(flat.root, "Bb");
        assert_eq!(flat.quality.name(), "dominant7");

        let slash = resolve("C/E").unwrap();
        assert_eq!(slash.bass.unwrap().name, "E");
    }

    #[test]
    fn rejects_unsupported_symbols() {
        assert!(resolve("Cfoo").unwrap_err().to_string().contains("Cfoo"));
    }
}
