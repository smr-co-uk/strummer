// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

pub fn notes_for_chord(name: &str) -> Option<Vec<u8>> {
    let harmony = crate::harmony::resolve(name).ok()?;
    let root = root_midi_note(harmony.root.as_str())?;
    Some(
        harmony
            .intervals
            .iter()
            .map(|interval| root + interval)
            .collect(),
    )
}

pub fn root_pitch_class(root_name: &str) -> Option<u8> {
    match root_name {
        "C" | "B#" => Some(0),
        "C#" | "Db" => Some(1),
        "D" => Some(2),
        "D#" | "Eb" => Some(3),
        "E" | "Fb" => Some(4),
        "E#" | "F" => Some(5),
        "F#" | "Gb" => Some(6),
        "G" => Some(7),
        "G#" | "Ab" => Some(8),
        "A" => Some(9),
        "A#" | "Bb" => Some(10),
        "B" | "Cb" => Some(11),
        _ => None,
    }
}

fn root_midi_note(root_name: &str) -> Option<u8> {
    Some(48 + root_pitch_class(root_name)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_common_chords() {
        assert_eq!(notes_for_chord("C"), Some(vec![48, 52, 55]));
        assert_eq!(notes_for_chord("Am"), Some(vec![57, 60, 64]));
        assert_eq!(notes_for_chord("G7"), Some(vec![55, 59, 62, 65]));
        assert_eq!(notes_for_chord("Hm"), None);
    }

    #[test]
    fn maps_sharp_and_flat_chords() {
        assert_eq!(notes_for_chord("C#"), Some(vec![49, 53, 56]));
        assert_eq!(notes_for_chord("Db"), Some(vec![49, 53, 56]));
        assert_eq!(notes_for_chord("Bbm"), Some(vec![58, 61, 65]));
        assert_eq!(notes_for_chord("A#7"), Some(vec![58, 62, 65, 68]));
        assert_eq!(notes_for_chord("Eb7"), Some(vec![51, 55, 58, 61]));
    }
}
