Feature: Resolve harmony, select guitar voicings, and render realistic strums

  The program resolves chord symbols through a harmony library,
  selects a compatible guitar voicing, and renders guitar-like
  downstrokes and upstrokes to MIDI.

  Background:
    Given the program is called "strum2midi"

  Rule: Built-in voicing sets are available

    Scenario: Use the built-in folk voicing set
      Given a strum file containing:
        """
        tempo: 92
        time: 4/4
        voicing: folk

        C D--- | ---- | ---- | ----
        """
      When I generate MIDI
      Then the command should succeed
      And the selected C voicing should be "x32010"

    Scenario: Folk voicings cover chromatic major minor and seventh chords
      Given a strum file containing:
        """
        tempo: 92
        time: 4/4
        voicing: folk

        | F#      Gbm     Bb7     Cbmaj7
        | D---    D---    D---    D---
        | E#m7    Db      Ebm     B#7
        | D---    D---    D---    D---
        """
      When I generate MIDI
      Then the command should succeed

    Scenario: Use the built-in rock voicing set
      Given a strum file containing:
        """
        tempo: 92
        time: 4/4
        voicing: rock

        C5 D--- | ---- | ---- | ----
        """
      When I generate MIDI
      Then the command should succeed
      And the selected C5 voicing should be "x355xx"

    Scenario: Command-line voicing overrides file metadata
      Given a strum file specifies "voicing: folk"
      When I run "strum2midi song.strum song.mid --voicing rock"
      Then the active voicing set should be "rock"

  Rule: Custom voicing files can be loaded

    Scenario: Use a custom voicing
      Given a custom voicing file containing:
        """
        name: custom
        voicings:
          C:
            - id: preferred-c
              frets: [x, 3, 2, 0, 1, 0]
              priority: 100
        """
      And a strum file containing a C chord
      When I run "strum2midi song.strum song.mid --voicing-file custom.yaml"
      Then the command should succeed
      And the selected voicing id should be "preferred-c"

    Scenario: Reject an invalid fret value
      Given a custom voicing contains fret 29
      When I load the custom voicing file
      Then the command should fail
      And the error should mention "fret value 29"

    Scenario: Select a voicing deterministically
      Given two compatible C voicings with different priorities
      When I generate MIDI twice
      Then the higher-priority voicing should be selected both times

  Rule: Conventional chord symbols are resolved by the harmony library

    Scenario Outline: Parse a conventional chord symbol
      Given the chord symbol "<symbol>"
      When the harmony library resolves it
      Then the intervals should be "<intervals>"

      Examples:
        | symbol | intervals |
        | C      | 1 3 5     |
        | Cm     | 1 b3 5    |
        | C5     | 1 5       |
        | Csus2  | 1 2 5     |
        | Csus4  | 1 4 5     |
        | C7     | 1 3 5 b7  |
        | Cmaj7  | 1 3 5 7   |
        | Cm7    | 1 b3 5 b7 |
        | Cadd9  | 1 3 5 9   |

    Scenario: Parse a sharp root
      Given the chord symbol "F#m"
      When the harmony library resolves it
      Then the root should be "F#"
      And the quality should be "minor"

    Scenario: Parse a flat root
      Given the chord symbol "Bb7"
      When the harmony library resolves it
      Then the root should be "Bb"
      And the quality should be "dominant7"

    Scenario: Reject an unsupported chord symbol
      Given a strum file contains the chord "Cfoo"
      When I generate MIDI
      Then the command should fail
      And the error should mention "unsupported chord symbol"
      And the error should mention "Cfoo"

  Rule: Voicings are validated against harmony

    Scenario: Validate an open C major voicing
      Given the chord symbol "C"
      And the voicing "x32010"
      When the voicing is validated
      Then it should contain the root
      And it should contain the major third
      And it should contain the perfect fifth
      And it should be valid

    Scenario: Validate a power chord without a third
      Given the chord symbol "C5"
      And the voicing "x355xx"
      When the voicing is validated
      Then it should contain C
      And it should contain G
      And it should not contain E
      And it should be valid

    Scenario: Reject a power chord containing a third
      Given the chord symbol "C5"
      And a voicing containing C, E, and G
      When the voicing is validated
      Then it should be invalid
      And the error should mention "third"

  Rule: Slash chords require the requested bass note

    Scenario: Accept a valid slash-chord voicing
      Given the chord symbol "C/E"
      And a voicing whose lowest sounding note is E
      When the voicing is validated
      Then it should be valid

    Scenario: Reject a slash chord with the wrong bass note
      Given the chord symbol "C/E"
      And a voicing whose lowest sounding note is C
      When the voicing is validated
      Then it should be invalid
      And the error should mention "expected E"

  Rule: Downstrokes behave like guitar downstrokes

    Scenario: Play a folk C chord from bass to treble
      Given the selected C voicing is "x32010"
      And the strum symbol is "D"
      When the strum is rendered
      Then the string order should be "A D G B high-E"
      And muted strings should produce no note events
      And successive notes should use the downstroke spread

    Scenario: A downstroke normally uses all sounding strings
      Given the selected C voicing is "x32010"
      When a downstroke is rendered with default settings
      Then 5 pitched note events should be produced

    Scenario: A downstroke uses the configured velocity
      Given the downstroke velocity is 94
      When a downstroke is rendered
      Then the first note velocity should be 94
      And later string velocities may be lower

  Rule: Upstrokes behave like guitar upstrokes

    Scenario: Play a folk C chord from treble to bass
      Given the selected C voicing is "x32010"
      And the strum symbol is "U"
      When the strum is rendered
      Then the first string should be "high-E"
      And later strings should proceed towards the bass

    Scenario: An upstroke may strike fewer strings
      Given the selected C voicing is "x32010"
      And the upstroke maximum string count is 3
      When an upstroke is rendered
      Then the string order should be "high-E B G"
      And exactly 3 pitched note events should be produced

    Scenario: An upstroke is lighter than a downstroke
      Given the downstroke velocity is 90
      And the upstroke velocity is 72
      When equivalent strokes are rendered
      Then the initial upstroke velocity should be lower

    Scenario: Upstroke and downstroke have different spreads
      Given default strum settings
      When a downstroke and an upstroke are rendered
      Then the downstroke spread should be 22 milliseconds
      And the upstroke spread should be 14 milliseconds

  Rule: Rests and muted strums are distinct

    Scenario: A rest generates no note events
      Given the strum symbol is "-"
      When the slot is rendered
      Then no MIDI note event should be produced

    Scenario: A muted strum generates a short percussive event
      Given the strum symbol is "X"
      When the slot is rendered
      Then a short percussive event should be produced
      And no note should remain active

  Rule: MIDI note lifecycle is valid

    Scenario: Every note-on has a corresponding note-off
      Given a valid strum file
      When I generate MIDI
      Then every note-on event should have a corresponding note-off event

    Scenario: Repeated strums do not leave hanging notes
      Given two consecutive strums contain the same pitch
      When I generate MIDI
      Then the first note should be terminated or safely retriggered
      And no note should remain active after the end-of-track event

  Rule: ASCII output uses the same selected voicing

    Scenario: Print a compact folk chord shape
      Given the active voicing set is "folk"
      When I run "strum2midi chords C"
      Then the output should contain:
        """
        C  x32010
        """

    Scenario: Print an expanded chord shape
      Given the active voicing set is "folk"
      When I run "strum2midi chords C --diagram"
      Then the output should contain:
        """
        e|--0--
        B|--1--
        G|--0--
        D|--2--
        A|--3--
        E|--x--
        """

    Scenario: MIDI and ASCII use the same voicing
      Given the selected C voicing is "x32010"
      When I generate MIDI and an ASCII diagram
      Then both outputs should be derived from "x32010"

  Rule: Missing voicings produce useful errors

    Scenario: Recognised harmony has no matching folk voicing
      Given the harmony library recognises "C#13"
      And the folk set contains no compatible C#13 voicing
      When I generate MIDI
      Then the command should fail
      And the error should mention "C#13"
      And the error should mention "folk"
      And the error should mention "no compatible voicing"
