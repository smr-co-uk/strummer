# strum2midi User Guide

This guide explains the `.strum` file format and the command-line options in practical terms.

## Quick Start

A minimal strum file looks like this:

```text
tempo: 92
time: 4/4

| C
| D--- D-U- --U- D-U-
```

Convert it to MIDI with:

```bash
strum2midi input.strum output.mid
```

During development you can run the same command through Cargo:

```bash
cargo run -- input.strum output.mid
```

## File Layout

A `.strum` file is normally written in this order:

```text
metadata

part: verse
| chord line
| strum pattern line
optional lyric line

part: chorus
| chord line
| strum pattern line

## Notes
free-form notes ignored by MIDI generation
```

Blank lines are allowed for readability.

## Metadata

Required metadata:

```text
tempo: 92
time: 4/4
```

Common optional metadata:

```text
instrument: acoustic_guitar
voicing: folk
velocity: 90
strum_spread_ms: 20
beat: quarter
subdivision: 16
count: 1e&a
```

Voiced strum controls:

```text
downstroke_velocity: 90
upstroke_velocity: 72
downstroke_spread_ms: 120
upstroke_spread_ms: 90
upstroke_max_strings: 4
```

`downstroke_spread_ms` and `upstroke_spread_ms` are the total sweep length from the first struck string to the last struck string. Notes are spaced evenly within that time.

## Instruments

Supported instruments:

| Metadata value | MIDI program |
|---|---:|
| `acoustic_guitar` | 25 |
| `electric_guitar_clean` | 27 |
| `nylon_guitar` | 24 |

If omitted, `instrument: acoustic_guitar` is used.

## Chord and Strum Lines

Chord and strum lines must start with `| `:

```text
| C                       Am
| D--- D-U- --U- D-U- | --U- D-U- --U- D-U-
```

The `| ` prefix marks the line as musical chart content. The prefix is not part of the chord or strum pattern.

The first character of each chord marks where that chord starts. In this example, `C` starts at the first beat pattern and `Am` starts at the first beat pattern after the bar separator.

## Strum Symbols

| Symbol | Meaning |
|---|---|
| `D` | Downstroke |
| `U` | Upstroke |
| `-` | Rest |
| `X` | Muted strum |

Example with a muted strum:

```text
| C
| D--- D-U- --U- X---
```

`X` produces a short low-velocity percussive chord event using the current chord.

## Rhythm and Subdivision

`subdivision` controls how many slots each beat pattern contains.

For sixteenth-note style patterns:

```text
tempo: 92
time: 4/4
subdivision: 16
count: 1e&a

| C
| D--- D-U- --U- D-U-
```

Each beat pattern has four slots.

For eighth-note style patterns:

```text
tempo: 92
time: 4/4
subdivision: 8
count: 1&

| C
| DU DU DU DU
```

Each beat pattern has two slots.

For 3/4 with eighth notes:

```text
tempo: 92
time: 3/4
subdivision: 8
count: 1&

| C
| DU DU DU
```

For 6/8 compound time:

```text
tempo: 72
time: 6/8
beat: dotted-quarter
subdivision: 8
count: 1&a

| C
| D-U D-U
```

## Multiple Bars on One Line

Use `|` inside the strum pattern line to separate bars:

```text
| C             Am             F              G
| DU DU DU DU | D- DU -U DU | DU D- DU -U | D- DU DU -U
```

## Repeating a Bar

Use `...` to repeat the previous bar pattern:

```text
| C             Am    F     G
| DU DU DU DU | ... | ... | ...
```

If a chord marker appears above a repeated bar, the rhythm is repeated with the new chord.

If no new chord marker appears, both the previous chord and rhythm are repeated:

```text
| C
| DU DU DU DU | ... | ... | ...
```

The first bar cannot be `...` because there is no previous bar to repeat.

## Lyrics

One optional lyric line may appear below each strum pattern line:

```text
| C                       Am
| D--- D-U- --U- D-U- | --U- D-U- --U- D-U-
This lyric line can contain | bar signs
```

Lyric lines are ignored when MIDI is generated. A lyric line must not start with `|`.

## Notes Section

Use `## Notes` for free-form notes at the end of the file:

```text
## Notes
This section is ignored by MIDI generation.
You can write arrangement notes, reminders, or alternate ideas here.
```

Everything after `## Notes` is ignored.

## Parts and Song Structure

Parts mark song sections:

```text
part: verse
| C
| D--- D-U- --U- D-U-

part: chorus
| G
| D--- D-U- --U- D-U-
```

Part names are written into the MIDI as marker events where supported by MIDI players.

Repeating a part name after it has been defined repeats the MIDI for that part:

```text
part: verse
| C
| D--- ---- ---- ----

part: chorus
| G
| D--- ---- ---- ----

part: verse
part: chorus
```

If a repeated part has not been defined, a warning is printed and the repeat is ignored.

## Chords

Supported roots include natural, sharp, flat, and enharmonic spellings:

```text
C C# Db D D# Eb E F F# Gb G G# Ab A A# Bb B
Cb B# E# Fb
```

Common supported qualities include:

```text
C     major
Cm    minor
C7    dominant seventh
Cmaj7 major seventh
Cm7   minor seventh
C5    power chord
Csus2
Csus4
Cadd9
Cm9
C13
```

Slash chords are supported when a compatible voicing exists:

```text
C/E
D/F#
G/B
```

## Voicing

Voicing chooses guitar chord shapes.

Use the built-in folk set:

```text
voicing: folk
```

Use the built-in rock set:

```text
voicing: rock
```

You can override the file on the command line:

```bash
strum2midi song.strum song.mid --voicing rock
```

The folk set includes major, minor, dominant seventh, major seventh, and minor seventh voicings for supported natural, sharp, flat, and enharmonic roots.

## Printing Chord Shapes

Print a compact shape:

```bash
strum2midi chords C
```

Output:

```text
C  x32010
```

Print an ASCII diagram:

```bash
strum2midi chords C --diagram
```

Output:

```text
e|--0--
B|--1--
G|--0--
D|--2--
A|--3--
E|--x--
```

Choose a voicing set:

```bash
strum2midi chords C5 --voicing rock
```

## Custom Voicing Files

You can load a custom voicing file:

```bash
strum2midi song.strum song.mid --voicing-file custom.yaml
```

Example format:

```yaml
name: custom

voicings:
  C:
    - id: preferred-c
      frets: [x, 3, 2, 0, 1, 0]
      tags: [folk, open]
      priority: 100
```

Fret values are written from low E to high E. `x` means the string is muted.

## Command-Line Options

Convert a file:

```bash
strum2midi input.strum output.mid
```

Override tempo:

```bash
strum2midi input.strum output.mid --tempo 100
```

Override velocity:

```bash
strum2midi input.strum output.mid --velocity 85
```

Override legacy per-note strum spread:

```bash
strum2midi input.strum output.mid --strum-spread-ms 15
```

Override voiced total sweep lengths:

```bash
strum2midi input.strum output.mid --downstroke-spread-ms 120 --upstroke-spread-ms 90
```

Select a voicing set:

```bash
strum2midi input.strum output.mid --voicing folk
```

Load custom voicings:

```bash
strum2midi input.strum output.mid --voicing-file custom.yaml
```

Command-line values override matching `.strum` metadata.

## Troubleshooting

`missing tempo`

Add `tempo: 92` near the top of the file.

`missing time signature`

Add `time: 4/4` or another supported time signature.

`chord and strum lines must start with '| '`

Chord and strum pattern lines need the leading chart marker:

```text
| C
| D--- D-U- --U- D-U-
```

`unknown chord`

Check the chord spelling. For voiced output, make sure the selected voicing set has a compatible shape.

`no compatible voicing`

The chord is understood, but the selected voicing library has no matching guitar shape. Try another voicing set, use a simpler chord, or provide a custom voicing file.

`expected 4 beat patterns`

The number of beat patterns in the bar does not match the time signature and beat grouping. In 4/4 with `beat: quarter`, each bar needs four beat patterns.

`expected 2 slots` or `expected 4 slots`

The number of characters in a beat pattern does not match `subdivision`.

---

Copyright 2026 smr.co.uk ltd.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
