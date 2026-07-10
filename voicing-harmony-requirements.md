# Strum2MIDI: Voicing, Harmony, and Strumming Requirements

## 1. Purpose

Extend `strum2midi` so that chord symbols are resolved through a harmony library, mapped to configurable guitar voicings, and rendered as MIDI using realistic guitar-style downstrokes and upstrokes.

This document covers:

1. Configurable guitar voicings with built-in folk and rock defaults
2. A harmony chord library
3. Guitar-like downstroke and upstroke playback

## 2. Core Model

The implementation shall keep these concepts separate:

- **Chord symbol**: textual instruction such as `C`, `Cm7`, `Csus4`, `G5`, or `D/F#`
- **Harmony definition**: intervals implied by the chord symbol
- **Guitar voicing**: fret used on each guitar string
- **Rendered MIDI notes**: pitches and timings derived from the selected voicing

Strings shall be stored from low E to high E.

Example:

```text
C: [x, 3, 2, 0, 1, 0]
```

Standard tuning shall be:

```text
E2 A2 D3 G3 B3 E4
```

## 3. Harmony Chord Library

### 3.1 Required chord qualities

The built-in harmony library shall support at least:

```text
major
minor
power
sus2
sus4
diminished
augmented
6
minor6
7
major7
minor7
minor-major7
9
minor9
add9
minor-add9
11
13
```

Examples:

```text
major:     1 3 5
minor:     1 b3 5
power:     1 5
sus2:      1 2 5
sus4:      1 4 5
dominant7: 1 3 5 b7
major7:    1 3 5 7
minor7:    1 b3 5 b7
add9:      1 3 5 9
```

### 3.2 Accidentals and alterations

The parser shall accept sharp and flat roots, for example:

```text
F#
Gb
Bb
C#m
```

It should support common alterations where practical:

```text
b5
#5
b9
#9
#11
b13
```

Unsupported chord symbols shall produce a clear error.

### 3.3 Slash chords

Slash chords shall be supported:

```text
C/E
G/B
D/F#
```

The slash note is the required lowest sounding note. A voicing that does not satisfy the bass requirement shall be rejected.

### 3.4 Harmony validation

A voicing shall be validated against the resolved chord harmony.

Validation shall check that:

- required chord tones are present
- sounding notes belong to the chord unless explicitly permitted as colour tones
- a power chord does not contain a third
- slash-chord bass requirements are satisfied
- muted strings are ignored

## 4. Guitar Voicing Library

### 4.1 Built-in voicing sets

The program shall include these built-in sets:

```text
folk
rock
```

### 4.2 Folk defaults

The `folk` set should prefer:

- open-position chords
- familiar acoustic shapes
- ringing open strings
- five- or six-string voicings
- low fret positions

Examples:

```text
C   x32010
Am  x02210
G   320003
D   xx0232
Em  022000
```

### 4.3 Rock defaults

The `rock` set should prefer:

- power chords
- movable barre shapes
- fewer open strings
- strong root and fifth
- shapes suited to distortion

Examples:

```text
C5  x355xx
D5  x577xx
E5  022xxx
G5  355xxx
A5  577xxx
```

A requested major or minor chord shall remain major or minor; choosing the rock set must not silently convert it to a power chord.

### 4.4 Configurable voicings

Users shall be able to load a custom voicing file.

Suggested format:

```yaml
name: custom

voicings:
  C:
    - id: open
      frets: [x, 3, 2, 0, 1, 0]
      tags: [folk, open]
      priority: 100

  C5:
    - id: power-a
      frets: [x, 3, 5, 5, x, x]
      tags: [rock, power]
      priority: 100
```

The exact serialization format may be YAML, TOML, or JSON, but it shall be documented and stable.

### 4.5 Selection

The voicing set may be selected in the input file:

```text
voicing: folk
```

or on the command line:

```bash
strum2midi song.strum song.mid --voicing rock
```

Command-line options shall override file metadata.

Where multiple compatible voicings exist, selection shall be deterministic. Version 1 may choose the highest-priority compatible voicing.

### 4.6 Missing voicings

If a chord is recognised by the harmony library but no compatible voicing exists, the program shall fail clearly.

Example:

```text
Line 8: chord 'C#13' is valid, but no compatible 'folk' voicing was found
```

## 5. Guitar-Style Strumming

### 5.1 Downstroke

A downstroke shall:

- move from low E towards high E
- skip muted strings
- trigger each sounding string separately
- apply a small delay between strings
- normally include all sounding strings

For `C x32010`, the order is:

```text
A -> D -> G -> B -> high E
```

### 5.2 Upstroke

An upstroke shall:

- move from high E towards low E
- skip muted strings
- trigger each sounding string separately
- normally strike fewer strings than a downstroke
- normally use lower velocity than a downstroke

For `C x32010`, a typical upstroke is:

```text
high E -> B -> G
```

### 5.3 Defaults

Suggested defaults:

```text
downstroke_velocity: 90
upstroke_velocity: 72
downstroke_spread_ms: 22
upstroke_spread_ms: 14
upstroke_max_strings: 4
```

These values shall be configurable.

### 5.4 Velocity and timing

The renderer should avoid assigning exactly the same velocity to every string.

- Downstrokes may become slightly softer from bass to treble.
- Upstrokes may become slightly softer from treble to bass.
- Output shall remain deterministic unless humanisation is explicitly enabled.

### 5.5 Note lifecycle

Every note-on event shall have a corresponding note-off event.

Repeated strums shall terminate or safely retrigger an already sounding pitch. No note may remain active after the end-of-track event.

### 5.6 Rests and muted strums

```text
-  rest: no note events
X  muted strum: short percussive event
```

Muted strums shall not leave notes sounding.

## 6. ASCII Chord Shapes

The selected voicing shall also be usable for printable ASCII output.

Compact form:

```text
C  x32010
```

Expanded form:

```text
C

e|--0--
B|--1--
G|--0--
D|--2--
A|--3--
E|--x--
```

The MIDI renderer and ASCII renderer shall consume the same selected voicing data.

Suggested commands:

```bash
strum2midi chords C Am F G --voicing folk
strum2midi chords C G5 --voicing rock --diagram
```

## 7. CLI Requirements

Examples:

```bash
strum2midi song.strum song.mid
strum2midi song.strum song.mid --voicing folk
strum2midi song.strum song.mid --voicing rock
strum2midi song.strum song.mid --voicing-file custom.yaml
```

Optional flags may include:

```text
--downstroke-velocity
--upstroke-velocity
--downstroke-spread-ms
--upstroke-spread-ms
--upstroke-max-strings
```

## 8. Architecture Guidance

Keep these responsibilities separate:

```text
chord_parser
harmony
voicing
voicing_library
voicing_selector
fretboard
strum_renderer
midi_writer
ascii_renderer
```

The harmony model shall not depend on MIDI output.

The MIDI and ASCII renderers shall consume the same voicing model.

## 9. Completion Criteria

This feature set is complete when:

- conventional chord symbols resolve to harmony definitions
- built-in folk and rock voicing sets are available
- custom voicing files can be loaded
- voicings are validated against chord harmony
- slash-chord bass notes are enforced
- downstrokes travel from bass to treble
- upstrokes travel from treble to bass
- upstrokes can use fewer strings and lower velocity
- MIDI output is deterministic and valid
- selected voicings can be printed as ASCII
- invalid chords and voicings produce useful errors
- automated tests cover the accompanying feature specification
