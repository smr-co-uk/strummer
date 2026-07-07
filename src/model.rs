#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Song {
    pub metadata: Metadata,
    pub parts: Vec<Part>,
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
    pub beat: Option<Beat>,
    pub subdivision: Option<u8>,
    pub count: Option<CountStyle>,
    pub instrument: Option<Instrument>,
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
