Feature: Convert guitar strumming text files to MIDI

  The program converts a compact plain-text guitar strumming format
  into a standard MIDI file for practice and accompaniment.

  Background:
    Given the program is called "strum2midi"

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

  Scenario: Muted strums create short percussive events
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      X--- ---- ---- ----
      """
    When I run "strum2midi song.strum song.mid"
    Then the MIDI file should contain a short percussive event at the first slot

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
