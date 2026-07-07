use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn run_case(name: &str, input: &str, args: &[&str]) -> (std::process::Output, PathBuf) {
    let root = std::env::temp_dir().join(format!("strum2midi-test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("song.strum"), input).unwrap();

    let mut command = Command::new(env!("CARGO_BIN_EXE_strum2midi"));
    command.current_dir(&root);
    command.arg("song.strum").arg("song.mid").args(args);
    (command.output().unwrap(), root)
}

#[test]
fn converts_simple_four_chord_file() {
    let input = "tempo: 92\ntime: 4/4\n\nC                       Am\nD--- D-U- --U- D-U- | --U- D-U- --U- D-U-\nF                       G\nD--- D-U- --U- D-U- | --U- D-U- --U- D-U-\n";
    let (output, root) = run_case("simple", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(&midi[..4], b"MThd");
    assert!(midi.windows(4).any(|window| window == b"MTrk"));
}

#[test]
fn command_line_tempo_overrides_metadata() {
    let input = "tempo: 92\ntime: 4/4\n\nC\nD--- D-U- --U- D-U-\n";
    let (output, root) = run_case("tempo-override", input, &["--tempo", "100"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(
        midi.windows(6)
            .any(|window| window == [0xFF, 0x51, 0x03, 0x09, 0x27, 0xC0])
    );
}

#[test]
fn defaults_to_acoustic_guitar_instrument() {
    let input = "tempo: 92\ntime: 4/4\n\nC\nD--- D-U- --U- D-U-\n";
    let (output, root) = run_case("default-instrument", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(midi.windows(2).any(|window| window == [0xC0, 25]));
}

#[test]
fn supports_instrument_metadata() {
    let input =
        "tempo: 92\ntime: 4/4\ninstrument: electric_guitar_clean\n\nC\nD--- D-U- --U- D-U-\n";
    let (output, root) = run_case("instrument", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(midi.windows(2).any(|window| window == [0xC0, 27]));
}

#[test]
fn supports_part_markers_between_chart_sections() {
    let input = "tempo: 92\ntime: 4/4\n\npart: verse\nC\nD--- D-U- --U- D-U-\npart: chorus\nG\nD--- D-U- --U- D-U-\n";
    let (output, root) = run_case("part-markers", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 36);
}

#[test]
fn rejects_unsupported_instrument_metadata() {
    let input = "tempo: 92\ntime: 4/4\ninstrument: banjo\n\nC\nD--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("bad-instrument", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("unsupported instrument"));
}

#[test]
fn rests_create_no_note_events() {
    let input = "tempo: 92\ntime: 4/4\n\nC\n---- ---- ---- ----\n";
    let (output, root) = run_case("rests", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(note_events(&midi).is_empty());
}

#[test]
fn muted_strums_create_percussive_events() {
    let input = "tempo: 92\ntime: 4/4\n\nC\nX--- ---- ---- ----\n";
    let (output, root) = run_case("muted", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(midi.windows(3).any(|window| window == [0x99, 37, 35]));
}

#[test]
fn rejects_unknown_chord_with_line_number() {
    let input = "tempo: 92\ntime: 4/4\n\nHm\nD--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("unknown-chord", input, &[]);

    assert!(!output.status.success());
    let stderr = stderr(&output).to_lowercase();
    assert!(stderr.contains("unknown chord"));
    assert!(stderr.contains("line 4"));
}

#[test]
fn supports_sharp_and_flat_chords() {
    let input =
        "tempo: 92\ntime: 4/4\n\nC#      Bbm     Eb7     A#7\nD---    D---    D---    D---\n";
    let (output, root) = run_case("sharp-flat-chords", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 14);
}

#[test]
fn rejects_invalid_symbol() {
    let input = "tempo: 92\ntime: 4/4\n\nC\nD-Z- D-U- --U- D-U-\n";
    let (output, _root) = run_case("invalid-symbol", input, &[]);

    assert!(!output.status.success());
    let stderr = stderr(&output);
    assert!(stderr.contains("invalid strum symbol"));
    assert!(stderr.contains("Z"));
}

#[test]
fn rejects_malformed_metadata() {
    let input = "tempo 92\ntime: 4/4\n\nC\nD--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("malformed-metadata", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("malformed metadata"));
}

#[test]
fn rejects_missing_tempo() {
    let input = "time: 4/4\n\nC\nD--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("missing-tempo", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("missing tempo"));
}

#[test]
fn rejects_missing_time_signature() {
    let input = "tempo: 92\n\nC\nD--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("missing-time", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("missing time signature"));
}

#[test]
fn rejects_wrong_number_of_patterns() {
    let input = "tempo: 92\ntime: 4/4\n\nC\nD--- D-U- --U-\n";
    let (output, _root) = run_case("wrong-patterns", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("expected 4 beat patterns"));
}

#[test]
fn accepts_extra_whitespace() {
    let input = "tempo: 92\ntime: 4/4\n\nC\nD---      D-U-      --U-      D-U-\n";
    let (output, root) = run_case("whitespace", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(Path::new(&root.join("song.mid")).exists());
}

#[test]
fn supports_four_four_eighth_note_subdivision() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\nC\nDU DU DU DU\n";
    let (output, root) = run_case("four-four-eighths", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 24);
}

#[test]
fn supports_three_four_eighth_note_subdivision() {
    let input = "tempo: 92\ntime: 3/4\nsubdivision: 8\ncount: 1&\n\nC\nDU DU DU\n";
    let (output, root) = run_case("three-four-eighths", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 18);
}

#[test]
fn supports_six_eight_dotted_quarter_beat() {
    let input =
        "tempo: 72\ntime: 6/8\nbeat: dotted-quarter\nsubdivision: 8\ncount: 1&a\n\nC\nD-U D-U\n";
    let (output, root) = run_case("six-eight-compound", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 12);
    assert!(
        midi.windows(7)
            .any(|window| window == [0xFF, 0x58, 0x04, 0x06, 0x03, 0x18, 0x08])
    );
}

#[test]
fn rejects_too_many_slots_for_eighth_note_subdivision() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\nC\nD-U- D-U- --U- D-U-\n";
    let (output, _root) = run_case("too-many-eighth-slots", input, &[]);

    assert!(!output.status.success());
    let stderr = stderr(&output).to_lowercase();
    assert!(stderr.contains("expected 2 slots"));
    assert!(stderr.contains("line 7"));
}

#[test]
fn rejects_unsupported_subdivision() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 12\n\nC\nDU DU DU DU\n";
    let (output, _root) = run_case("unsupported-subdivision", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("unsupported subdivision"));
}

#[test]
fn rejects_unsupported_count_style() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 16\ncount: triplet\n\nC\nD-U- D-U- --U- D-U-\n";
    let (output, _root) = run_case("unsupported-count", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("unsupported count"));
}

#[test]
fn rejects_six_eight_without_beat() {
    let input = "tempo: 72\ntime: 6/8\nsubdivision: 8\ncount: 1&a\n\nC\nD-U D-U\n";
    let (output, _root) = run_case("six-eight-missing-beat", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("missing beat"));
}

#[test]
fn supports_multiple_bars_per_line() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\nC             Am\nDU DU DU DU | D- DU -U DU\n";
    let (output, root) = run_case("multiple-bars-per-line", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 42);
}

#[test]
fn supports_chord_repeat_marker() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\nC             Am    F     G\nDU DU DU DU | ... | ... | ...\n";
    let (output, root) = run_case("chord-repeat-marker", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 96);
}

#[test]
fn supports_full_bar_repeat_marker() {
    let input =
        "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\nC\nDU DU DU DU | ... | ... | ...\n";
    let (output, root) = run_case("full-repeat-marker", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 96);
}

#[test]
fn rejects_repeat_marker_without_previous_bar() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\nC\n...\n";
    let (output, _root) = run_case("bad-repeat-marker", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("repeat marker requires a previous bar"));
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

fn note_on_count(midi: &[u8]) -> usize {
    note_events(midi)
        .iter()
        .filter(|(status, _note, velocity)| *status == 0x90 && *velocity > 0)
        .count()
}

fn note_events(midi: &[u8]) -> Vec<(u8, u8, u8)> {
    let Some(track_start) = midi.windows(4).position(|window| window == b"MTrk") else {
        return Vec::new();
    };
    let mut index = track_start + 8;
    let mut events = Vec::new();

    while index < midi.len() {
        while index < midi.len() {
            let byte = midi[index];
            index += 1;
            if byte & 0x80 == 0 {
                break;
            }
        }
        if index >= midi.len() {
            break;
        }

        let status = midi[index];
        index += 1;
        match status {
            0x80..=0x9F => {
                if index + 2 > midi.len() {
                    break;
                }
                events.push((status, midi[index], midi[index + 1]));
                index += 2;
            }
            0xC0..=0xDF => {
                if index + 1 > midi.len() {
                    break;
                }
                index += 1;
            }
            0xFF => {
                if index + 1 >= midi.len() {
                    break;
                }
                let length = usize::from(midi[index + 1]);
                index += 2 + length;
            }
            _ => break,
        }
    }

    events
}
