// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Song {
    pub metadata: Metadata,
    pub parts: Vec<Part>,
    pub warnings: Vec<String>,
    pub bars: Vec<Bar>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub line: usize,
    pub name: String,
    pub bar_index: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Metadata {
    pub tempo: Option<u16>,
    pub time_signature: Option<TimeSignature>,
    pub velocity: Option<u8>,
    pub strum_spread_ms: Option<u16>,
    pub downstroke_velocity: Option<u8>,
    pub upstroke_velocity: Option<u8>,
    pub downstroke_spread_ms: Option<u16>,
    pub upstroke_spread_ms: Option<u16>,
    pub upstroke_max_strings: Option<u8>,
    pub beat: Option<Beat>,
    pub subdivision: Option<u8>,
    pub count: Option<CountStyle>,
    pub instrument: Option<Instrument>,
    pub voicing: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Beat {
    Quarter,
    DottedQuarter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountStyle {
    OneAnd,
    OneAndA,
    OneEAndA,
    OneAAndA,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instrument {
    AcousticGuitar,
    ElectricGuitarClean,
    NylonGuitar,
}

impl Instrument {
    pub fn midi_program(self) -> u8 {
        match self {
            Self::AcousticGuitar => 25,
            Self::ElectricGuitarClean => 27,
            Self::NylonGuitar => 24,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bar {
    pub line: usize,
    pub beats: Vec<BeatPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeatPattern {
    pub chord: String,
    pub chord_line: usize,
    pub slots: Vec<StrumSymbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrumSymbol {
    Down,
    Up,
    Rest,
    Muted,
}
