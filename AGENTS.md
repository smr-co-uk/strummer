# AGENTS.md

## Project

`strum2midi` is a small Rust 2024 CLI program that converts a plain-text guitar strumming file into a standard MIDI file.

Use these files as the source of truth:

- `requirements.md` for product requirements
- `strum2midi.feature` for acceptance scenarios

Do not duplicate requirements in this file.

## Environment

The project must build and test in both:

- the local developer environment
- the provided devcontainer

The devcontainer should use the stable Rust toolchain and include any system packages needed for build and test.

## Rust

Use Rust 2024 edition.

Prefer simple, idiomatic Rust over clever abstractions.

## Required Commands

These commands must work locally and in the devcontainer:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

The CLI should run as:

```bash
strum2midi input.strum output.mid
```

## Suggested Structure

Keep the code modular but small:

```text
src/
  main.rs
  cli.rs
  parser.rs
  model.rs
  validate.rs
  timing.rs
  chord.rs
  midi_writer.rs
  error.rs
```

Only add modules when they are useful.

## Guidelines

- Keep parsing, validation, timing, chord mapping, and MIDI writing separate.
- Return clear user-facing errors, with line numbers where possible.
- Avoid panics for invalid input.
- Keep dependencies minimal.
- Prefer deterministic MIDI output so tests are reliable.
- Add tests for new behaviour.

## Devcontainer

Provide a minimal `.devcontainer/devcontainer.json`.

It should allow the project to be opened in a container and run the required Cargo commands.

## Completion Criteria

A change is complete when:

- the code builds
- formatting passes
- clippy passes with warnings denied
- tests pass
- relevant requirements and feature scenarios are satisfied
