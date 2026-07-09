// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

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
fn strum_001_converts_simple_four_chord_file() {
    let input = "tempo: 92\ntime: 4/4\n\n| C                       Am\n| D--- D-U- --U- D-U- | --U- D-U- --U- D-U-\n| F                       G\n| D--- D-U- --U- D-U- | --U- D-U- --U- D-U-\n";
    let (output, root) = run_case("simple", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(&midi[..4], b"MThd");
    assert!(midi.windows(4).any(|window| window == b"MTrk"));
}

#[test]
fn strum_002_uses_tempo_from_input_file() {
    let input = "tempo: 120\ntime: 4/4\n\n| C\n| D--- D-U- --U- D-U-\n";
    let (output, root) = run_case("tempo-input", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(
        midi.windows(6)
            .any(|window| window == [0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20])
    );
}

#[test]
fn strum_003_command_line_tempo_overrides_metadata() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| D--- D-U- --U- D-U-\n";
    let (output, root) = run_case("tempo-override", input, &["--tempo", "100"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(
        midi.windows(6)
            .any(|window| window == [0xFF, 0x51, 0x03, 0x09, 0x27, 0xC0])
    );
}

#[test]
fn strum_004_defaults_to_acoustic_guitar_instrument() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| D--- D-U- --U- D-U-\n";
    let (output, root) = run_case("default-instrument", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(midi.windows(2).any(|window| window == [0xC0, 25]));
}

#[test]
fn strum_005_supports_instrument_metadata() {
    let input =
        "tempo: 92\ntime: 4/4\ninstrument: electric_guitar_clean\n\n| C\n| D--- D-U- --U- D-U-\n";
    let (output, root) = run_case("instrument", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(midi.windows(2).any(|window| window == [0xC0, 27]));
}

#[test]
fn strum_006_supports_velocity_metadata() {
    let input = "tempo: 92\ntime: 4/4\nvelocity: 64\n\n| C\n| D--- ---- ---- ----\n";
    let (output, root) = run_case("velocity-metadata", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(midi.windows(3).any(|window| window == [0x90, 48, 64]));
}

#[test]
fn strum_007_command_line_velocity_overrides_metadata() {
    let input = "tempo: 92\ntime: 4/4\nvelocity: 64\n\n| C\n| D--- ---- ---- ----\n";
    let (output, root) = run_case("velocity-override", input, &["--velocity", "80"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(midi.windows(3).any(|window| window == [0x90, 48, 80]));
}

#[test]
fn strum_008_supports_part_markers_between_chart_sections() {
    let input = "tempo: 92\ntime: 4/4\n\npart: verse\n| C\n| D--- D-U- --U- D-U-\npart: chorus\n| G\n| D--- D-U- --U- D-U-\n";
    let (output, root) = run_case("part-markers", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 36);
    assert!(midi.windows(8).any(|window| window == b"\xFF\x06\x05verse"));
}

#[test]
fn strum_009_repeats_previously_defined_parts() {
    let input = "tempo: 92\ntime: 4/4\n\npart: verse\n| C\n| D--- ---- ---- ----\npart: chorus\n| G\n| D--- ---- ---- ----\npart: verse\n";
    let (output, root) = run_case("repeat-part", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 9);
}

#[test]
fn strum_010_warns_and_ignores_undefined_part_repeats() {
    let input = "tempo: 92\ntime: 4/4\n\npart: bridge\n";
    let (output, _root) = run_case("undefined-repeat-part", input, &[]);

    let stderr = stderr(&output);
    assert!(stderr.contains("repeated part 'bridge' is not defined"));
}

#[test]
fn strum_011_downstroke_orders_notes_low_to_high() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| D--- ---- ---- ----\n";
    let (output, root) = run_case("downstroke-order", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    let notes = note_on_notes(&midi);
    assert_eq!(&notes[..3], &[48, 52, 55]);
}

#[test]
fn strum_012_upstroke_orders_notes_high_to_low() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| --U- ---- ---- ----\n";
    let (output, root) = run_case("upstroke-order", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    let notes = note_on_notes(&midi);
    assert_eq!(&notes[..3], &[55, 52, 48]);
}

#[test]
fn strum_019_rejects_unsupported_instrument_metadata() {
    let input = "tempo: 92\ntime: 4/4\ninstrument: banjo\n\n| C\n| D--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("bad-instrument", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("unsupported instrument"));
}

#[test]
fn strum_014_rests_create_no_note_events() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| ---- ---- ---- ----\n";
    let (output, root) = run_case("rests", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(note_events(&midi).is_empty());
}

#[test]
fn strum_015_muted_strums_create_short_low_velocity_chord_events() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| X--- ---- ---- ----\n";
    let (output, root) = run_case("muted", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert!(midi.windows(3).any(|window| window == [0x90, 48, 25]));
    assert!(!midi.iter().any(|byte| *byte == 0x99 || *byte == 0x89));
}

#[test]
fn strum_016_rejects_unknown_chord_with_line_number() {
    let input = "tempo: 92\ntime: 4/4\n\n| Hm\n| D--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("unknown-chord", input, &[]);

    assert!(!output.status.success());
    let stderr = stderr(&output).to_lowercase();
    assert!(stderr.contains("unknown chord"));
    assert!(stderr.contains("line 4"));
}

#[test]
fn strum_013_supports_sharp_and_flat_chords() {
    let input =
        "tempo: 92\ntime: 4/4\n\n| C#      Bbm     Eb7     A#7\n| D---    D---    D---    D---\n";
    let (output, root) = run_case("sharp-flat-chords", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 14);
}

#[test]
fn strum_017_rejects_invalid_symbol() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| D-Z- D-U- --U- D-U-\n";
    let (output, _root) = run_case("invalid-symbol", input, &[]);

    assert!(!output.status.success());
    let stderr = stderr(&output);
    assert!(stderr.contains("invalid strum symbol"));
    assert!(stderr.contains("Z"));
}

#[test]
fn strum_018_rejects_malformed_metadata() {
    let input = "tempo 92\ntime: 4/4\n\n| C\n| D--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("malformed-metadata", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("malformed metadata"));
}

#[test]
fn strum_020_rejects_missing_tempo() {
    let input = "time: 4/4\n\n| C\n| D--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("missing-tempo", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("missing tempo"));
}

#[test]
fn strum_021_rejects_missing_time_signature() {
    let input = "tempo: 92\n\n| C\n| D--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("missing-time", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("missing time signature"));
}

#[test]
fn strum_024_rejects_zero_tempo() {
    let input = "tempo: 0\ntime: 4/4\n\n| C\n| D--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("zero-tempo", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("tempo must be greater than zero"));
}

#[test]
fn strum_022_rejects_wrong_number_of_patterns() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| D--- D-U- --U-\n";
    let (output, _root) = run_case("wrong-patterns", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("expected 4 beat patterns"));
}

#[test]
fn strum_023_accepts_extra_whitespace() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| D---      D-U-      --U-      D-U-\n";
    let (output, root) = run_case("whitespace", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(Path::new(&root.join("song.mid")).exists());
}

#[test]
fn strum_025_accepts_optional_lyrics_under_strum_lines() {
    let input = "tempo: 92\ntime: 4/4\n\n| C                       Am\n| D--- D-U- --U- D-U- | --U- D-U- --U- D-U-\nA lyric line can contain | bar signs\n";
    let (output, root) = run_case("lyrics", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 36);
}

#[test]
fn strum_026_rejects_chart_lines_without_required_bar_prefix() {
    let input = "tempo: 92\ntime: 4/4\n\nC\nD--- D-U- --U- D-U-\n";
    let (output, _root) = run_case("missing-chart-prefix", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("chord and strum lines must start with '| '"));
}

#[test]
fn strum_027_ignores_everything_after_notes_section() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| D--- ---- ---- ----\n## Notes\nFree-form notes can include | bars\nHm\n| ZZZZ\n";
    let (output, root) = run_case("notes-section", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 3);
}

#[test]
fn subdiv_001_supports_four_four_eighth_note_subdivision() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\n| C\n| DU DU DU DU\n";
    let (output, root) = run_case("four-four-eighths", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 24);
}

#[test]
fn subdiv_006_supports_four_four_sixteenth_note_count() {
    let input =
        "tempo: 92\ntime: 4/4\nsubdivision: 16\ncount: 1e&a\n\n| C\n| D-U- D-U- --U- D-U-\n";
    let (output, root) = run_case("four-four-sixteenths", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 21);
}

#[test]
fn subdiv_007_supports_four_four_alternate_sixteenth_note_count() {
    let input =
        "tempo: 92\ntime: 4/4\nsubdivision: 16\ncount: 1a&a\n\n| C\n| D-U- D-U- --U- D-U-\n";
    let (output, root) = run_case("four-four-alternate-sixteenths", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 21);
}

#[test]
fn subdiv_008_default_subdivision_remains_sixteenth_note_compatible() {
    let input = "tempo: 92\ntime: 4/4\n\n| C\n| D--- D-U- --U- D-U-\n";
    let (output, root) = run_case("default-sixteenths", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 18);
}

#[test]
fn subdiv_009_supports_three_four_eighth_note_subdivision() {
    let input = "tempo: 92\ntime: 3/4\nsubdivision: 8\ncount: 1&\n\n| C\n| DU DU DU\n";
    let (output, root) = run_case("three-four-eighths", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 18);
}

#[test]
fn subdiv_010_supports_three_four_sixteenth_note_subdivision() {
    let input = "tempo: 92\ntime: 3/4\nsubdivision: 16\ncount: 1e&a\n\n| C\n| D-U- D-U- --U-\n";
    let (output, root) = run_case("three-four-sixteenths", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 15);
}

#[test]
fn subdiv_011_supports_six_eight_dotted_quarter_beat() {
    let input = "tempo: 72\ntime: 6/8\nbeat: dotted-quarter\nsubdivision: 8\ncount: 1&a\n\n| C\n| D-U D-U\n";
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
fn subdiv_013_rejects_too_many_slots_for_eighth_note_subdivision() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\n| C\n| D-U- D-U- --U- D-U-\n";
    let (output, _root) = run_case("too-many-eighth-slots", input, &[]);

    assert!(!output.status.success());
    let stderr = stderr(&output).to_lowercase();
    assert!(stderr.contains("expected 2 slots"));
    assert!(stderr.contains("line 7"));
}

#[test]
fn subdiv_014_rejects_unsupported_subdivision() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 12\n\n| C\n| DU DU DU DU\n";
    let (output, _root) = run_case("unsupported-subdivision", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("unsupported subdivision"));
}

#[test]
fn subdiv_015_rejects_unsupported_count_style() {
    let input =
        "tempo: 92\ntime: 4/4\nsubdivision: 16\ncount: triplet\n\n| C\n| D-U- D-U- --U- D-U-\n";
    let (output, _root) = run_case("unsupported-count", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("unsupported count"));
}

#[test]
fn subdiv_012_rejects_six_eight_without_beat() {
    let input = "tempo: 72\ntime: 6/8\nsubdivision: 8\ncount: 1&a\n\n| C\n| D-U D-U\n";
    let (output, _root) = run_case("six-eight-missing-beat", input, &[]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("missing beat"));
}

#[test]
fn subdiv_002_supports_multiple_bars_per_line() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\n| C             Am\n| DU DU DU DU | D- DU -U DU\n";
    let (output, root) = run_case("multiple-bars-per-line", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 42);
}

#[test]
fn subdiv_003_supports_chord_repeat_marker() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\n| C             Am    F     G\n| DU DU DU DU | ... | ... | ...\n";
    let (output, root) = run_case("chord-repeat-marker", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 96);
}

#[test]
fn subdiv_004_supports_full_bar_repeat_marker() {
    let input =
        "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\n| C\n| DU DU DU DU | ... | ... | ...\n";
    let (output, root) = run_case("full-repeat-marker", input, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let midi = fs::read(root.join("song.mid")).unwrap();
    assert_eq!(note_on_count(&midi), 96);
}

#[test]
fn subdiv_005_rejects_repeat_marker_without_previous_bar() {
    let input = "tempo: 92\ntime: 4/4\nsubdivision: 8\ncount: 1&\n\n| C\n| ...\n";
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

fn note_on_notes(midi: &[u8]) -> Vec<u8> {
    note_events(midi)
        .iter()
        .filter(|(status, _note, velocity)| *status == 0x90 && *velocity > 0)
        .map(|(_status, note, _velocity)| *note)
        .collect()
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
