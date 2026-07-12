# strum2midi

## Brief Introduction

`strum2midi` is a small Rust CLI that converts a plain-text guitar strumming file into a standard MIDI file.

It reads metadata such as tempo, time signature, rhythm subdivision, instrument, voicing, and song parts, then turns chord and strum pattern lines into deterministic MIDI note events.

## How to Run

Run the CLI with an input `.strum` file and an output `.mid` file:

```bash
cargo run -- input.strum output.mid
```

Example `input.strum`:

```text
tempo: 92
time: 4/4
instrument: acoustic_guitar
voicing: folk

part: verse
| C                       Am
| D--- D-U- --U- X--- | --U- D-U- --U- D-U-
part: chorus
| F                       G
| D--- D-U- --U- D-U- | --U- D-U- --U- D-U-
```

`voicing: folk` selects built-in guitar chord shapes. In strum patterns, `D` is a downstroke, `U` is an upstroke, `-` is a rest, and `X` is a muted strum.

After running the command, `output.mid` can be opened in a MIDI player, DAW, or MIDI inspection tool.

The repository also includes [canonical-example.strum](canonical-example.strum), which demonstrates the supported metadata, voicing, part markers, chord placement, bar separators, repeat markers, rests, muted strums, and sharp/flat chords.

For a fuller explanation of the file format and command-line options, see [USER_GUIDE.md](USER_GUIDE.md).

## How to Build

Build a debug binary:

```bash
cargo build
```

Build an optimized release binary:

```bash
cargo build --release
```

The release binary will be created at:

```text
target/release/strum2midi
```

Useful project checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## How to Install Rust

Install Rust with `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then restart your shell or load Cargo into your current shell:

```bash
source "$HOME/.cargo/env"
```

Verify the installation:

```bash
rustc --version
cargo --version
```

---

Copyright 2026 smr.co.uk ltd.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
