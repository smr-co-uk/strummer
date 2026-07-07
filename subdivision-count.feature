Feature: Use subdivision and count metadata for strumming patterns

  The program supports both eighth-note and sixteenth-note strumming
  patterns in 4/4 while preserving the existing compact beat-pattern format.

  Background:
    Given the program is called "strum2midi"

  Scenario: Convert an eighth-note 1&2&3&4& pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      C
      DU DU DU DU
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And a file named "song.mid" should exist
    And the MIDI file should contain eight evenly spaced strums in the bar

  Scenario: Convert multiple bars on one line using bar separators
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      C             Am             F              G
      DU DU DU DU | D- DU -U DU | DU D- DU -U | D- DU DU -U
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain four bars

  Scenario: Repeat the previous bar pattern with a new chord
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      C             Am    F     G
      DU DU DU DU | ... | ... | ...
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And each repeated bar should use the previous bar pattern
    And each repeated bar should use its own chord

  Scenario: Repeat the previous full bar
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      C
      DU DU DU DU | ... | ... | ...
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And each repeated bar should use the previous chord and bar pattern

  Scenario: Reject a repeat marker before any bar pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      C
      ...
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "repeat marker requires a previous bar"

  Scenario: Convert a sixteenth-note 1e&a count pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 16
      count: 1e&a

      C
      D-U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should place each slot on the sixteenth-note grid

  Scenario: Convert a sixteenth-note 1a&a count pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 16
      count: 1a&a

      C
      D-U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI timing should match the same pattern with count "1e&a"

  Scenario: Default subdivision remains sixteenth-note compatible
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      C
      D-U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should place each slot on the sixteenth-note grid

  Scenario: Convert a 3/4 eighth-note 1&2&3& pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 3/4
      subdivision: 8
      count: 1&

      C
      DU DU DU
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain six evenly spaced strums in the bar

  Scenario: Convert a 3/4 sixteenth-note 1e&a pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 3/4
      subdivision: 16
      count: 1e&a

      C
      D-U- D-U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should place each slot on the sixteenth-note grid for three beats

  Scenario: Convert a 6/8 compound 1&a2&a pattern
    Given a file named "song.strum" containing:
      """
      tempo: 72
      time: 6/8
      beat: dotted-quarter
      subdivision: 8
      count: 1&a

      C
      D-U D-U
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain two dotted-quarter beats
    And each beat should contain three eighth-note slots

  Scenario: Reject 6/8 compound meter without a beat
    Given a file named "song.strum" containing:
      """
      tempo: 72
      time: 6/8
      subdivision: 8
      count: 1&a

      C
      D-U D-U
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "missing beat"

  Scenario: Reject a pattern with too many slots for eighth-note subdivision
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      C
      D-U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "expected 2 slots"
    And the error should mention "line 7"

  Scenario: Reject an unsupported subdivision
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 12

      C
      DU DU DU DU
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "unsupported subdivision"

  Scenario: Reject an unsupported count style
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 16
      count: triplet

      C
      D-U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "unsupported count"
