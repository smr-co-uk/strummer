// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

use std::{fs, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scenario {
    feature_file: &'static str,
    tag: String,
    title: String,
}

#[test]
fn every_feature_scenario_has_acceptance_trace() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let acceptance = fs::read_to_string(root.join("tests/acceptance.rs"))
        .expect("tests/acceptance.rs should be readable");
    let scenarios = ["strum2midi.feature", "subdivision-count.feature"]
        .into_iter()
        .flat_map(|feature_file| read_scenarios(root, feature_file))
        .collect::<Vec<_>>();

    let missing = scenarios
        .iter()
        .filter(|scenario| !has_acceptance_trace(&acceptance, scenario.tag.as_str()))
        .map(|scenario| {
            format!(
                "{} {} ({})",
                scenario.tag, scenario.title, scenario.feature_file
            )
        })
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "missing acceptance trace for feature scenarios:\n{}",
        missing.join("\n")
    );
}

fn read_scenarios(root: &Path, feature_file: &'static str) -> Vec<Scenario> {
    let content =
        fs::read_to_string(root.join(feature_file)).expect("feature file should be readable");
    let mut pending_tags = Vec::new();
    let mut scenarios = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('@') {
            pending_tags.extend(
                trimmed
                    .split_whitespace()
                    .filter(|token| is_trace_tag(token))
                    .map(ToString::to_string),
            );
            continue;
        }

        let Some(title) = trimmed.strip_prefix("Scenario:") else {
            continue;
        };
        let title = title.trim().to_string();
        for tag in pending_tags.drain(..) {
            scenarios.push(Scenario {
                feature_file,
                tag,
                title: title.clone(),
            });
        }
    }

    scenarios
}

fn has_acceptance_trace(acceptance: &str, tag: &str) -> bool {
    let normalized = normalize_tag(tag);
    acceptance.contains(format!("fn {normalized}_").as_str())
        || acceptance.contains(format!("Covers: {tag}").as_str())
}

fn normalize_tag(tag: &str) -> String {
    tag.trim_start_matches('@').replace('-', "_").to_lowercase()
}

fn is_trace_tag(token: &str) -> bool {
    let Some((prefix, number)) = token.trim_start_matches('@').split_once('-') else {
        return false;
    };

    !prefix.is_empty()
        && prefix
            .chars()
            .all(|character| character.is_ascii_uppercase())
        && number.len() == 3
        && number.chars().all(|character| character.is_ascii_digit())
}
