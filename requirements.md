# Strum2MIDI Requirements

## 1. Purpose

Create a command-line tool that converts a simple text-based guitar strumming format into a standard MIDI file.

The tool is intended for guitar practice, rhythm experimentation, and quick creation of MIDI accompaniment from readable plain text.

## 2. Scope

Version 1 supports:

- Plain text input files
- Tempo and time signature metadata
- Part markers for readable song structure
- Chord names on a chord line above the strum pattern
- Strumming patterns using compact symbols
- MIDI output containing chord strums
- Downstroke and upstroke note ordering
- Rests
- Basic muted strums
- Validation with useful error messages

Version 1 does not need to support:

- Full tablature
- Audio rendering
- Realistic guitar synthesis
- Complex repeats or song forms
- MusicXML export
- GUI editing

## 3. Example Input Format

```text
tempo: 92
time: 4/4

C                       Am
D--- D-U- --U- D-U- | --U- D-U- --U- D-U-
F                       G
D--- D-U- --U- D-U- | --U- D-U- --U- D-U-
```

## 4. Core Concepts

### 4.1 Metadata

The input file may begin with metadata lines.

Required:

```text
tempo: 92
time: 4/4
```

Optional future metadata may include:

```text
swing: false
capo: 0
velocity: 90
```

Optional instrument metadata:

```text
instrument: acoustic_guitar
```

If `instrument` is omitted, the program shall use `acoustic_guitar`.

Supported values:

| Value | MIDI program |
|---|---:|
| `acoustic_guitar` | 25 |
| `electric_guitar_clean` | 27 |
| `nylon_guitar` | 24 |

Optional song structure metadata:

```text
part: verse
```

`part` must be followed by a non-empty part name.

Part markers may appear before or between chord/pattern line pairs:

```text
part: verse
C
D--- D-U- --U- D-U-
part: chorus
G
D--- D-U- --U- D-U-
```

For Version 1, `part` is a structural placeholder for readability and library consumers. It shall not change MIDI timing, notes, instruments, or output events.

Optional rhythm metadata:

```text
beat: quarter
subdivision: 16
count: 1e&a
```

`beat` controls how the time signature is grouped into beat patterns.
Supported values:

| Value | Meaning                                                        |
|---|----------------------------------------------------------------|
| `quarter` | One beat pattern per quarter-note beat                         |
| `dotted-quarter` | One beat pattern per dotted-quarter beat, needed for 6/8 time  |

If `beat` is omitted, the program shall use `quarter` for simple meters such as `3/4` and `4/4`.

For compound meters such as `6/8`, `beat: dotted-quarter` shall group the bar into two beat patterns. This supports common compound counting such as `1&a2&a`.

`subdivision` controls how many equal slots each beat contains.
Supported values:

| Value | Slots per quarter beat | Slots per dotted-quarter beat | Example count |
|---|---:|---:|---|
| `8` | 2 | 3 | `1&` or `1&a` |
| `16` | 4 | 6 | `1e&a` |

If `subdivision` is omitted, the program shall use `16` to preserve the original four-slot beat-pattern behavior.

`count` controls the user-facing count style for the same timing grid. It does not change MIDI timing.
Supported values:

| Value | Meaning |
|---|---|
| `1&` | Eighth-note count labels |
| `1&a` | Compound eighth-note count labels |
| `1e&a` | Standard sixteenth-note count labels |
| `1a&a` | Alternate four-slot count labels |

### 4.2 Chord and Pattern Lines

Musical content is written as pairs of lines:

```text
<chord-line>
<strum-pattern-line>
```

Example:

```text
C                       Am
D--- D-U- --U- D-U- | --U- D-U- --U- D-U-
```

The first letter of each chord marks the beat pattern where that chord begins. The chord applies from that beat pattern until the next chord marker.

The `|` character is a bar separator on the strum-pattern line. A pattern line may contain one bar or several bars.

Beat patterns inside a bar are separated by whitespace.

A chord may begin inside a bar:

```text
C         G
D--- D-U- --U- D-U-
```

In this example, `G` begins at the third beat pattern.

The special marker `...` repeats the previous bar pattern. It may repeat the rhythm with a new chord:

```text
C             Am    F     G
D--- D-U- --U- D-U- | ... | ... | ...
```

It may also repeat the previous chord and rhythm:

```text
C
D--- D-U- --U- D-U- | ... | ... | ...
```

The first bar in a file cannot use `...` because there is no previous bar pattern to repeat.

### 4.3 Beat Pattern

Each beat pattern contains one slot per subdivision within a beat.

With the default `subdivision: 16`, each beat pattern contains four slots.

Example:

```text
D-U-
```

This means:

| Slot | Meaning |
|---|---|
| 1 | Downstroke |
| 2 | Rest |
| 3 | Upstroke |
| 4 | Rest |

For 4/4 time with `beat: quarter`, a bar normally has four beat patterns.

With `beat: quarter` and `subdivision: 8`, each beat pattern contains two slots.

Example:

```text
DU
```

This means:

| Slot | Count | Meaning |
|---|---|---|
| 1 | Beat number | Downstroke |
| 2 | `&` | Upstroke |

For 4/4 time with `subdivision: 8`, a full bar may be written as:

```text
C
DU DU DU DU
```

This represents `1&2&3&4&`.

With `subdivision: 16`, a full bar may use the standard `1e&a2e&a3e&a4e&a` count or the alternate `1a&a2a&a3a&a4a&a` count style. Both count styles use four equal slots per quarter beat and produce the same timing.

For 6/8 time with `beat: dotted-quarter` and `subdivision: 8`, a bar has two beat patterns and each beat pattern contains three slots.

Example:

```text
C
D-U D-U
```

This represents `1&a2&a`.

## 5. Symbols

| Symbol | Meaning |
|---|---|
| `D` | Downstroke |
| `U` | Upstroke |
| `-` | Rest |
| `X` | Muted strum |

Future symbols may include:

| Symbol | Meaning |
|---|---|
| `d` | Soft downstroke |
| `u` | Soft upstroke |
| `!D` | Accented downstroke |
| `!U` | Accented upstroke |

## 6. MIDI Behaviour

### 6.1 Chords

The program shall map common chord names to MIDI notes.

Minimum supported chord roots:

- Natural roots: `C`, `D`, `E`, `F`, `G`, `A`, `B`
- Sharp roots: `C#`, `D#`, `E#`, `F#`, `G#`, `A#`, `B#`
- Flat roots: `Cb`, `Db`, `Eb`, `Fb`, `Gb`, `Ab`, `Bb`

Supported qualities for each root:

- Major: no suffix, for example `C`, `F#`, `Bb`
- Minor: `m`, for example `Cm`, `F#m`, `Bbm`
- Seventh: `7`, for example `C7`, `F#7`, `Bb7`

Enharmonic spellings that refer to the same pitch shall produce the same notes, for example `C#` and `Db`.

The implementation may use simple closed-position voicings for Version 1.

### 6.2 Downstroke

A downstroke shall play the notes of the chord from low to high.

### 6.3 Upstroke

An upstroke shall play the notes of the chord from high to low.

### 6.4 Strum Spread

A strum shall not play every note at exactly the same time.

The program shall use a small configurable delay between notes to simulate a strum.

Default:

```text
strum_spread_ms: 20
```

### 6.5 Muted Strum

A muted strum shall produce a short percussive MIDI event.

The implementation may use a short low-velocity chord, a percussion note, or another simple MIDI representation.

### 6.6 Rests

A rest shall produce no MIDI notes.

### 6.7 Instrument

The MIDI file shall contain a program change event for the selected instrument before note events.

The default instrument shall be:

```text
acoustic_guitar
```

## 7. Timing

The tool shall calculate MIDI timing from:

- Tempo
- Time signature
- Beat grouping
- Number of beat patterns per bar
- Number of slots per beat pattern

For Version 1:

- The default beat is `quarter` for simple meters.
- The default subdivision is `16`.
- With `beat: quarter` and `subdivision: 16`, each beat pattern has four slots.
- With `beat: quarter` and `subdivision: 8`, each beat pattern has two slots.
- With `beat: dotted-quarter` and `subdivision: 8`, each beat pattern has three slots.
- In 4/4 time with `beat: quarter`, each bar has four beat patterns.
- In 3/4 time with `beat: quarter`, each bar has three beat patterns.
- In 6/8 time with `beat: dotted-quarter`, each bar has two beat patterns.
- Therefore a 4/4 bar has eight slots at `subdivision: 8` and sixteen slots at `subdivision: 16`.

## 8. Command Line Interface

The basic command shall be:

```bash
strum2midi input.strum output.mid
```

Optional flags:

```bash
strum2midi input.strum output.mid --tempo 100
strum2midi input.strum output.mid --velocity 85
strum2midi input.strum output.mid --strum-spread-ms 15
```

Command-line flags override metadata in the input file.

## 9. Validation Requirements

The program shall reject invalid input with clear error messages.

Validation should detect:

- Missing tempo
- Missing time signature
- Unknown chord
- Invalid strum symbol
- Wrong number of beat patterns for the time signature
- Wrong number of slots for the selected subdivision
- Repeat marker without a previous bar
- Missing beat for compound meters that require explicit grouping
- Unsupported beat
- Unsupported subdivision
- Unsupported count style
- Unsupported instrument
- Empty part name
- Empty input file
- Malformed metadata line

Example error:

```text
Line 5: unknown chord 'Hm'
```

## 10. Output Requirements

The program shall create a valid Standard MIDI File.

Minimum requirements:

- One MIDI track
- Tempo event
- Time signature event
- Program change event
- Note on/off events
- End-of-track event

## 11. Non-Functional Requirements

The tool should be:

- Cross-platform
- Usable from a terminal
- Deterministic
- Easy to test
- Suitable for use in automated scripts
- Implemented with clear separation between parsing, validation, timing, and MIDI writing

## 12. Suggested Architecture

Suggested modules:

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

## 13. Acceptance Criteria

The implementation is complete when:

- A valid `.strum` file can be converted to `.mid`
- Downstrokes play low-to-high
- Upstrokes play high-to-low
- Rests produce silence
- Muted strums produce short percussive events
- Invalid input produces useful line-based errors
- Automated tests cover parsing, validation, chord mapping, timing, and MIDI event generation

## References
For information only and not part of the requirements:
* https://miditoolbox.com/analyzer
* https://www.midi.org/specifications/item/table-1-summary-of-midi-message
* https://streamdevprojects.com/midi
