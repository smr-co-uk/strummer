# Copyright 2026 smr.co.uk ltd
# SPDX-License-Identifier: Apache-2.0

Feature: Use subdivision and count metadata for strumming patterns

  The program supports both eighth-note and sixteenth-note strumming
  patterns in 4/4 while preserving the existing compact beat-pattern format.

  Background:
    Given the program is called "strum2midi"

  @SUBDIV-001
  Scenario: Convert an eighth-note 1&2&3&4& pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      | C
      | DU DU DU DU
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And a file named "song.mid" should exist
    And the MIDI file should contain eight evenly spaced strums in the bar

  @SUBDIV-002
  Scenario: Convert multiple bars on one line using bar separators
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      | C             Am             F              G
      | DU DU DU DU | D- DU -U DU | DU D- DU -U | D- DU DU -U
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain four bars

  @SUBDIV-003
  Scenario: Repeat the previous bar pattern with a new chord
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      | C             Am    F     G
      | DU DU DU DU | ... | ... | ...
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And each repeated bar should use the previous bar pattern
    And each repeated bar should use its own chord

  @SUBDIV-004
  Scenario: Repeat the previous full bar
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      | C
      | DU DU DU DU | ... | ... | ...
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And each repeated bar should use the previous chord and bar pattern

  @SUBDIV-005
  Scenario: Reject a repeat marker before any bar pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      | C
      | ...
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "repeat marker requires a previous bar"

  @SUBDIV-006
  Scenario: Convert a sixteenth-note 1e&a count pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 16
      count: 1e&a

      | C
      | D-U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should place each slot on the sixteenth-note grid

  @SUBDIV-007
  Scenario: Convert a sixteenth-note 1a&a count pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 16
      count: 1a&a

      | C
      | D-U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI timing should match the same pattern with count "1e&a"

  @SUBDIV-008
  Scenario: Default subdivision remains sixteenth-note compatible
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4

      | C
      | D-U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should place each slot on the sixteenth-note grid

  @SUBDIV-009
  Scenario: Convert a 3/4 eighth-note 1&2&3& pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 3/4
      subdivision: 8
      count: 1&

      | C
      | DU DU DU
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain six evenly spaced strums in the bar

  @SUBDIV-010
  Scenario: Convert a 3/4 sixteenth-note 1e&a pattern
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 3/4
      subdivision: 16
      count: 1e&a

      | C
      | D-U- D-U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should place each slot on the sixteenth-note grid for three beats

  @SUBDIV-011
  Scenario: Convert a 6/8 compound 1&a2&a pattern
    Given a file named "song.strum" containing:
      """
      tempo: 72
      time: 6/8
      beat: dotted-quarter
      subdivision: 8
      count: 1&a

      | C
      | D-U D-U
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should succeed
    And the MIDI file should contain two dotted-quarter beats
    And each beat should contain three eighth-note slots

  @SUBDIV-012
  Scenario: Reject 6/8 compound meter without a beat
    Given a file named "song.strum" containing:
      """
      tempo: 72
      time: 6/8
      subdivision: 8
      count: 1&a

      | C
      | D-U D-U
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "missing beat"

  @SUBDIV-013
  Scenario: Reject a pattern with too many slots for eighth-note subdivision
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 8
      count: 1&

      | C
      | D-U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "expected 2 slots"
    And the error should mention "line 7"

  @SUBDIV-014
  Scenario: Reject an unsupported subdivision
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 12

      | C
      | DU DU DU DU
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "unsupported subdivision"

  @SUBDIV-015
  Scenario: Reject an unsupported count style
    Given a file named "song.strum" containing:
      """
      tempo: 92
      time: 4/4
      subdivision: 16
      count: triplet

      | C
      | D-U- D-U- --U- D-U-
      """
    When I run "strum2midi song.strum song.mid"
    Then the command should fail
    And the error should mention "unsupported count"
