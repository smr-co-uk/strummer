// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

pub fn notes_for_chord(name: &str) -> Option<Vec<u8>> {
    let (root_name, quality) = split_chord_name(name)?;
    let root = root_midi_note(root_name)?;

    let intervals = match quality {
        Quality::Major => &[0, 4, 7][..],
        Quality::Minor => &[0, 3, 7][..],
        Quality::Seventh => &[0, 4, 7, 10][..],
    };

    Some(intervals.iter().map(|interval| root + interval).collect())
}

fn split_chord_name(name: &str) -> Option<(&str, Quality)> {
    if let Some(root) = name.strip_suffix('m') {
        Some((root, Quality::Minor))
    } else if let Some(root) = name.strip_suffix('7') {
        Some((root, Quality::Seventh))
    } else {
        Some((name, Quality::Major))
    }
}

fn root_midi_note(root_name: &str) -> Option<u8> {
    match root_name {
        "C" => Some(48),
        "C#" | "Db" => Some(49),
        "D" => Some(50),
        "D#" | "Eb" => Some(51),
        "E" | "Fb" => Some(52),
        "E#" | "F" => Some(53),
        "F#" | "Gb" => Some(54),
        "G" => Some(55),
        "G#" | "Ab" => Some(56),
        "A" => Some(57),
        "A#" | "Bb" => Some(58),
        "B" | "Cb" => Some(59),
        "B#" => Some(60),
        _ => None,
    }
}

enum Quality {
    Major,
    Minor,
    Seventh,
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
