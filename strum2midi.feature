# Copyright 2026 smr.co.uk ltd
# SPDX-License-Identifier: Apache-2.0

Feature: Convert guitar strumming text files to MIDI

  The program converts a compact plain-text guitar strumming format
  into a standard MIDI file for practice and accompaniment.

  Background:
    Given the program is called "strum2midi"

  @STRUM-001
  Scenario: Convert a simple four-chord strumming file
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C                       Am
      D--- D-U- --U- D-U- | --U- D-U- --U- D-U-
      F                       G
      D--- D-U- --U- D-U- | --U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And a file named "song.mid" should exist

    And "song.mid" should be a valid MIDI file

  @STRUM-002
  Scenario: Use tempo from the input file
    Given a file named "song.strum" containing:
      """
      tempo: 120
      time: 4/4

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the MIDI file should contain a tempo event for 120 BPM

  @STRUM-003
  Scenario: Override tempo from the command line
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid --tempo 100"
    Then the MIDI file should contain a tempo event for 100 BPM

  @STRUM-004
  Scenario: Use acoustic guitar as the default instrument
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain a program change event for acoustic guitar

  @STRUM-005
  Scenario: Use instrument from the input file
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      instrument: electric_guitar_clean

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain a program change event for clean electric guitar

  @STRUM-006
  Scenario: Use performance metadata from the input file
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      velocity: 64
      strum_spread_ms: 15

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain note events with velocity 64

  @STRUM-007
  Scenario: Override performance metadata from the command line
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      velocity: 64
      strum_spread_ms: 15

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid --velocity 80 --strum-spread-ms 5"
    Then the command should succeed
    And the MIDI file should contain note events with velocity 80

  @STRUM-008
  Scenario: Use part markers for song structure
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      part: verse
      C
      D--- D-U- --U- D-U-
      part: chorus
      G
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And a file named "song.mid" should exist
    And the part markers should not change the MIDI notes

  @STRUM-009
  Scenario: Repeat previously defined parts
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      part: verse
      C
      D--- ---- ---- ----
      part: chorus
      G
      D--- ---- ---- ----
      part: verse
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain the verse part twice
    And the MIDI file should contain marker events for part names

  @STRUM-010
  Scenario: Warn and ignore an undefined repeated part
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      part: bridge
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And stderr should contain "repeated part 'bridge' is not defined"

  @STRUM-011
  Scenario: Downstroke plays chord notes from low to high
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      D--- ---- ---- ----
      """
    When I run "strum2midi song.strum song.mid"
    Then the first strum should play the C chord notes from low to high

  @STRUM-012
  Scenario: Upstroke plays chord notes from high to low
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      --U- ---- ---- ----
      """
    When I run "strum2midi song.strum song.mid"
    Then the first strum should play the C chord notes from high to low

  @STRUM-013
  Scenario: Convert sharp and flat chord names
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C#      Bbm     Eb7     A#7
      D---    D---    D---    D---
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain chord note events for sharp and flat chords

  @STRUM-014
  Scenario: Rests create no MIDI note events
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      ---- ---- ---- ----
      """
    When I run "strum2midi song.strum song.mid"
    Then the MIDI file should contain no chord note events

  @STRUM-015
  Scenario: Muted strums create short low-velocity chord events
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      X--- ---- ---- ----
      """
    When I run "strum2midi song.strum song.mid"
    Then the MIDI file should contain short low-velocity C chord note events at the first slot

  @STRUM-016
  Scenario: Reject an unknown chord
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      Hm
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "unknown chord"
    And the error should mention "line 4"

  @STRUM-017
  Scenario: Reject an invalid strum symbol
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      D-Z- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "invalid strum symbol"
    And the error should mention "Z"

  @STRUM-018
  Scenario: Reject a malformed metadata line
    Given a file named "song.strum" containing:
      """
      tempo 92
      time: 4/4

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "malformed metadata"

  @STRUM-019
  Scenario: Reject an unsupported instrument
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      instrument: banjo

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "unsupported instrument"

  @STRUM-020
  Scenario: Reject a missing tempo
    Given a file named "song.strum" containing:
      """
      time: 4/4

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "missing tempo"

  @STRUM-021
  Scenario: Reject a missing time signature
    Given a file named "song.strum" containing:
      """
      tempo: 92

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "missing time signature"

  @STRUM-024
  Scenario: Reject zero tempo
    Given a file named "song.strum" containing:
      """
      tempo: 0
      time: 4/4

      C
      D--- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "tempo must be greater than zero"

  @STRUM-022
  Scenario: Reject the wrong number of beat patterns in 4/4
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      D--- D-U- --U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "expected 4 beat patterns"

  @STRUM-023
  Scenario: Accept extra whitespace
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      D---      D-U-      --U-      D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And a file named "song.mid" should exist
