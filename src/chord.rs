pub fn notes_for_chord(name: &str) -> Option<Vec<u8>> {
    let (root_name, quality) = if let Some(root) = name.strip_suffix('m') {
        (root, Quality::Minor)
    } else if let Some(root) = name.strip_suffix('7') {
        (root, Quality::Seventh)
    } else {
        (name, Quality::Major)
    };

    let root = match root_name {
        "C" => 48,
        "D" => 50,
        "E" => 52,
        "F" => 53,
        "G" => 55,
        "A" => 57,
        "B" => 59,
        _ => return None,
    };

    let intervals = match quality {
        Quality::Major => &[0, 4, 7][..],
        Quality::Minor => &[0, 3, 7][..],
        Quality::Seventh => &[0, 4, 7, 10][..],
    };

    Some(intervals.iter().map(|interval| root + interval).collect())
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
}
