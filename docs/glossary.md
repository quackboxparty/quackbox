# Quackbox

Self-hostable multi-gamemode quiz platform. Content (questions, packs, tags,
media) lives as PR-reviewable YAML; the runtime serves it to live multiplayer
game sessions. This glossary covers the **runtime** domain (live sessions,
participants, adjudication). Content-shape terms live in `data-model.md`.

## Language

### Session & participants

**Room**:
A single live game session. Players join it by its join code; it owns the game
state for one playthrough and ceases to exist when emptied.
_Avoid_: game (ambiguous with gamemode), lobby, session (use for the whole
playthrough, not the container).

**Join code**:
The 6-character code that identifies a Room and is the only thing a player needs
to enter it. It is the Room's sole identity for its whole life.
_Avoid_: room id, game pin, UUID.

**Reconnect token**:
An opaque secret issued to a participant on join, stored client-side, that
re-binds a new connection to the same slot, score, and grants after a drop.
_Avoid_: session id, auth token.

**Player**:
A participant who occupies a slot, buzzes, and owns a score — i.e. holds the
Play grant.
_Avoid_: contestant, user, guest.

**Moderator**:
A participant who advances the game, manages participants, sees correct answers,
and rules on answers — i.e. holds the Moderate grant. May also be a Player.
_Avoid_: host (use for the human running the session colloquially, not the grant),
admin, judge (a Judge is a mechanism, not a person).

**Presenter**:
A participant whose view is the big-screen output (question, options,
scoreboard, timer) — i.e. holds the Present grant. Typically a shared display.
_Avoid_: screen, board, host.

**Announcer**:
The participant currently entitled to read the question aloud. In a rotating
setup this shifts each question. A use of the Present grant, not a separate one.
_Avoid_: reader, narrator.

### Authorization

**Grant**:
An atomic capability held by a connection: Play, Present, or Moderate.
Authorization is a set of grants, never a fixed role — roles are just named
bundles that compose by union.
_Avoid_: role, permission, scope.

**ClientView**:
The role-specific projection of game state sent to one connection. Sections
(buzzer, question, correct answer, controls, scoreboard) appear only if the
connection's grants permit. The point where server-only secrets are stripped.
_Avoid_: state, snapshot (a ClientView is the *projected* snapshot), payload.

### Adjudication

**Judge**:
The mechanism that decides whether a submission is correct, in time, and valid.
A pluggable axis independent of gamemode. v1: Auto (static matcher) and Moderator
(human ruling).
_Avoid_: scorer, validator, checker, referee.

**Submission**:
What a Player offers in answer to a question. Nullable: present when typed,
absent when the answer is spoken and only the Moderator's verdict is recorded.
_Avoid_: answer (reserve for the *correct* answer on content), response, guess.

**Verdict**:
A Judge's ruling on a submission: Correct, Incorrect, Void, or Pending.
_Avoid_: result, outcome, judgment (reserve for the log entry).

**Judgment log**:
The append-only record of verdicts. Score is folded from it, never accumulated,
so a Moderator can revise an earlier ruling by appending a superseding verdict.
_Avoid_: score table, history, audit log.

### Play mechanics

**Floor**:
The right to answer the current question. Won by buzzing; lost on a wrong answer
(see Lockout); may be open to all or follow a turn order.
_Avoid_: turn, control, lock.

**Buzz**:
A Player's claim on the floor. The first buzz to arrive wins. Distinct from a
submission — a buzz claims the right to answer, it is not the answer.
_Avoid_: ring, press, hit.

**Lockout**:
Exclusion of a Player from re-buzzing a question after they answered it wrong.
Orthogonal to whether the floor started open or turn-ordered.
_Avoid_: ban, timeout (reserve for the clock), block.

**Answer-input mode**:
The Room-level setting that determines how answers are given — spoken, typed, or
hybrid — and thereby which Judge runs per question.
_Avoid_: game mode (that's gamemode), online/offline mode, input type.

**Gamemode**:
A set of rules and presentation for play (grid_quiz, battle royale, …). Owns the
buzz/order policy and scoring shape. Distinct from the Judge (how answers are
ruled) and the answer-input mode (how answers are given).
_Avoid_: mode, game type, ruleset.
