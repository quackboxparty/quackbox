# Quackbox — Runtime Architecture

> Status: **design**. The data layer (content load/validate/query/board) is
> implemented in Rust (`api/src/data/`); the game runtime described here is not
> built yet. Scope: how the backend serves content and drives live multiplayer
> game sessions. Content shape is in `data-model.md`; UX flow is in
> `game-flow.md`. This doc is the canonical reference for backend ownership,
> transport, concurrency, and the game-session model.

## Stack shift (supersedes AGENTS.md)

AGENTS.md describes a SvelteKit-server architecture (remote functions, valibot,
TS data layer). That is **superseded**. The current architecture:

- **Rust + axum backend** (`api/`) owns everything server-side: content loading,
  validation, pool query, board building, and the live game runtime.
- **SvelteKit is a pure static frontend** — built with `adapter-static`, served
  by axum as static files (`axum::serve` over `../build`). No SvelteKit server,
  no SSR, no remote functions.
- **`ts-rs`** exports Rust types to TypeScript. Rust is the single source of
  truth for all shared types (content + wire protocol); the frontend imports
  generated `.ts`.

### Offline-capable

The **core game loop must run with no internet** after setup — a moderator
self-hosts at home (binary + content + local media) and plays on a LAN or single
box. Internet-only features (`youtube:`/`url:` media, OTLP telemetry, a future
LLM judge) are opt-in and **gracefully absent offline**, never a hard dependency
for core play. Keep the core play path free of outbound calls. See
`docs/decisions/0003-offline-capable-self-host.md`.

The legacy TS data layer (`src/lib/server/data/`) and its schemas
(`src/lib/schemas/`) have been **removed** — the Rust port in `api/src/data/`
reached parity and is now the sole data layer.

### Why Rust owns the data layer

Single owner eliminates the two-validator desync the data model fights against
(`data-model.md`: "no desync possible"). Rust loads YAML once
(`serde_yaml` + `garde` validation), exposes types via `ts-rs`, and the frontend
consumes generated types. One schema, one validator, one truth.

## Frontend ↔ backend contract

Two transports, split by temperature of the data:

- **REST** for **cold content** — questions, packs, boards, gamemode manifests.
  Cacheable, stateless reads. Served under `/api/*`.
- **WebSocket** for **hot game state** — buzzes, answers, scores, timer, phase.
  One WS connection per client into a live room (axum `ws` feature).

`ts-rs` exports both the REST DTOs and the WS message types, so the Svelte
client is fully typed against the Rust definitions.

## Game runtime — concurrency model

### Actor-per-room

Each live game room is **one owning tokio task** holding the room's mutable
`GameState`. Rooms are fully **isolated** — one room's task hanging or panicking
cannot touch another's state. No shared lock on the hot path.

- **`mpsc` (many→one): players → room.** Every socket in a room holds a clone of
  the room's `mpsc::Sender`. A player action (`Buzz`, `Answer`, …) is sent as a
  `Command` into the channel. The room task is the single receiver, pulling
  commands one at a time — input is serialized, so there are no data races on
  `GameState` and **first-buzz-wins falls out of channel ordering** (no explicit
  race handling).
- **One `broadcast` (one→many): room → players.** The room task holds a
  `broadcast::Sender<GameState>`; every socket holds a `Receiver`. On each state
  change the room broadcasts the **full-truth `GameState` once**; all sockets
  receive a copy.
- **Room task loop** is a `tokio::select!` racing: next `Command` from the mpsc,
  the next timer deadline (`sleep_until`), and a shutdown signal. Whichever fires
  first mutates state, broadcasts, loops.

```
players --mpsc.send(Command)--> [ROOM TASK owns GameState] --broadcast(GameState)--> sockets
                                  loop { select! { cmd | timer | quit } }    each socket: project(state, grants) -> ws.send
```

### Room registry & lifecycle

A central `Arc<DashMap<JoinCode, RoomHandle>>` lives in `AppState`. `RoomHandle`
holds the room's `mpsc::Sender` and `broadcast::Sender` (subscribe for a
receiver). The map is touched **only** on create / join / destroy — never on the
hot buzz/timer path.

- **Create** → spawn room task, insert handle.
- **Join** → look up by join code, clone the mpsc sender, subscribe to broadcast.
- **Destroy** → the room task owns its own death (empty-room timeout via the same
  `select!`) and **removes its own entry** on exit. No central reaper, no orphan
  tasks.

### Persistence

**None in v1.** Rooms are pure in-memory tokio tasks + channels. A server
restart drops all live games — acceptable: a quiz session is minutes long and
ephemeral, like Kahoot. The only durable state is content (YAML on disk).

`ponytail:` rooms in-memory only; add SQLite snapshot+restore when
crash-recovery matters. When added: the room task is already the single owner of
its state, so it can snapshot per state-transition without locks, and boot
restores the DashMap from rows. Persist timer **deadline timestamps**, not
remaining seconds, so clocks survive wall-clock gaps.

## Room identity & joining

- **One identifier: a 6-char join code** (e.g. `A1B2C3`). It is the registry key
  and the room's identity for its whole life. Regenerated on live collision. No
  separate UUID — a dead room's code lookup simply fails, which is correct.
  `ponytail:` single join-code identity; add a stable opaque id only if reconnect
  tokens or persistence need it.
- **Creation gate** — server-level secret in config/env. Controls whether anyone
  may spin up a game on this instance. Open if unset.
- **Join gate** — optional per-game password, set by the host in the UI at
  creation. Empty = the code is the only secret.
- **Capacity cap** — joins rejected when full. Host can lock the room after start
  (no late joins) — a quiz mid-question can't cleanly absorb new players.

`ponytail:` join code is guessable; add entry throttling when the instance is
public.

## Reconnection

Phones sleep and wifi blips mid-quiz, so reconnection is required, not optional.

- On join the server issues an **opaque reconnect token**; the client stores it
  in `localStorage`.
- A dropped socket **detaches** but the player stays present-but-disconnected —
  slot, score, and grants persist on the room, keyed by token.
- The next socket presenting that token **reattaches** to the exact slot.
  Survives a page refresh too.
- The **player** needs a stable token even though the **room** does not need a
  stable id — different concerns, no conflict with the single-join-code decision.

## Wire protocol & per-role projection

### Tagged-enum envelopes

- **Client → server:** a `Command` enum (`Join`, `Buzz`, `Answer`, `Next`,
  `StartGame`, `Grant`, `ExtendTimer`, …).
- **Server → client:** a `ServerMsg` enum (`Snapshot(ClientView)`, `Joined`,
  `Error`, …).

serde `#[serde(tag = "type")]` → clean discriminated unions in TS via `ts-rs`.

### Full snapshot, not deltas

Each broadcast carries the **complete current view**, not a diff. A quiz state is
small (players, scores, current question, timer deadline, phase), so the client
is a pure function `render(view)` — killing an entire class of client/server
delta-desync bugs (same spirit as the data model's "no desync"). `ponytail:`
full-snapshot broadcast; add deltas only if a room's state outgrows a WS frame.

### Capabilities, not fixed roles

There is **no single game state to send** — a phone player, a presenter screen,
and a moderator each see different things, and roles **compose** (a moderator can
also play). Modeled as **grants**, not roles:

- **`Play`** — occupy a slot, buzz, submit answers, own a score.
- **`Present`** — see question / options / scoreboard / timer (the big-screen
  view).
- **`Moderate`** — advance the game, manage players, **see correct answers**,
  controls.

A connection holds a **grant set**. Named roles are grant bundles that compose by
union: "player" = `{Play}`, "presenter" = `{Present}`, "host" =
`{Moderate, Present}`, "playing moderator" = `{Moderate, Present, Play}`.

### Projection is the trust boundary

The room broadcasts the **full-truth `GameState`** on one channel. Each socket
runs **one** function `project(&state, &grants) -> ClientView` before `ws.send`.
`ClientView` is a single `ts-rs`-exported type with **optional sections**
(`question?`, `buzzer?`, `answer_input?`, `correct_answer?`, `controls?`,
`scoreboard?`); `project` fills each section only if the grants permit it.

- The **correct answer is stripped server-side** before it reaches a player
  socket — never sent-then-hidden in the client. `project` is the single gate.
- This is the one place security is **not** simplified away: `project` carries a
  test asserting a `{Play}`-only view never contains `correct_answer`.
- Composition is free: a playing-moderator's `ClientView` simply has buzzer +
  controls + correct_answer populated. One channel, one `project`, any grant
  combination — no per-role channels (which would explode combinatorially for
  composite roles).

`ponytail:` full-truth broadcast + `project(state, grants)`; one channel, grants
compose freely.

### Granting & authorization (least privilege)

- The **creator** of a room gets `Moderate` (+ `Present`, optionally `Play`),
  bound to their reconnect token. Creating requires the server creation secret,
  so `Moderate` is transitively gated by config/env.
- **Every joiner gets exactly `{Play}`** — no requesting, no choosing. Just the
  join code (+ password if set).
- **All other grants** (`Present`, `Moderate`, revoking `Play`) are assigned
  **after** join by an existing moderator via a `Command::Grant { token, grants }`
  that **only moderators may send**. Server-enforced.
- **The client never asserts its own grants.** It sends a token; the server
  stores grants per token and looks them up. Devtools can forge a `Command` but
  not a grant. Reconnect restores the slot's stored grants.

A presenter (big screen) therefore also joins as `{Play}` first, then a
moderator flips it to `{Present}` — one join path, least privilege, no self-claim
surface.

## Gamemode model

The **data layer** is already gamemode-agnostic (pool query + board builder).
The **runtime** is not yet — and deliberately so.

- **v1: grid_quiz is hardcoded** inside the room task. Buzz/lockout/timer/scoring
  policy lives directly in the loop.
- **Later: extract a `Gamemode` trait** when gamemode #2 lands and reveals the
  real seams (AGENTS.md uses the second gamemode to validate the
  gamemode-agnostic claim). Designing the trait now would guess seams before
  feeling them.

`ponytail:` grid_quiz hardcoded in the room task; extract the `Gamemode` trait
when #2 lands and shows the real seams. The doc records the **intent**
(runtime becomes gamemode-pluggable); the **code** stays concrete for v1.

### Buzz / order policy

A **policy axis** owned by the gamemode (its default) and **overridable by the
moderator** per game:

- **open-floor first-buzz** — question goes live, any `Play` grant may buzz, the
  first buzz to arrive wins the floor (channel-ordered).
- **turn-order** — players take turns; on a **failed** question the floor opens
  to everyone **except those already wrong**.

**Lockout is orthogonal** to the initial-floor policy: a wrong answer locks that
player out and reopens the floor to the rest, until correct / timeout / all
locked out. Same lockout mechanism under both policies.

`ponytail:` open-floor first-buzz + lockout-on-wrong hardcoded for grid_quiz;
buzz/order/timer policy moves into the `Gamemode` trait when #2 lands.

#### Side note — rotating announcer

A future variant: the read-aloud `Present` grant **rotates round-robin per
question**; only the current announcer reads, and stops on buzz. No new
machinery — just reassign `Present` per question on the existing capability
model. `ponytail:` build when a gamemode wants it.

## Adjudication — the judge axis

"Was this answer correct / in time / valid" is **its own pluggable axis,
orthogonal to gamemode.** A game is `(Gamemode × Judge)`, chosen independently.

### Submission ≠ score; decisions are revisable

A submission enters a `Pending` state; a **judgment** resolves it to
`Correct` / `Incorrect` / `Void`. Because a moderator can make a mistake and
revise an earlier ruling, **score is never an accumulated counter.** Instead:

- An **append-only judgment log**: each entry is
  `(player, question, submission?, verdict, supersedes?)`.
- **`score = fold(judgment_log)`** — recomputed from the log, never mutated in
  place.
- **Revising** = append a new verdict that supersedes the old one, refold,
  rebroadcast. Because the client is `render(view)` (full snapshot), a refold
  just produces a new snapshot — no special "score correction" message.

`submission` is **nullable**: `Some(typed_answer)` when answers are typed,
`None` when spoken (the moderator's verdict stands alone). This mirrors the data
model's "correctness lives on data, derive don't duplicate."

### v1 judges (two — behind a `Judge` trait)

Two real implementations justify building the trait now (unlike the gamemode
trait, which has only one v1 case). The two are the genuinely distinct
*mechanisms*:

- **`Auto`** — static matcher against the data layer (`correct: true`, numeric
  `answer` + `tolerance`). Returns a resolved verdict synchronously. Covers
  MC / T-F / numeric. The default.
- **`Moderator`** — submission lands `Pending`; a `Moderate`-granted human rules
  `Correct` / `Incorrect` / `Void` (and in-time or not) via a command. Covers
  open/freeform answers `Auto` can't match, **and** is the override path on top
  of `Auto` (a moderator revising an auto-verdict is the same append-to-log).

Trait shape these two reveal:
`Judge::verdict(submission, &question_data) -> Verdict` where
`Verdict ∈ {Correct, Incorrect, Void, Pending}`. Both feed the **same**
append-only judgment log, so override/revision is identical regardless of which
judge produced the original verdict — the symmetry that proves the seam is right.

Deferred judges (same trait, later):

- `ponytail:` **Quorum** = the `Moderator` verdict path with vote aggregation;
  add when a sole-judge-conflict session (playing moderator) needs it.
- `ponytail:` **Llm** = `Auto` with a model matcher + human fallback
  (`NeedsHuman`); add when freeform answers outgrow moderator throughput.

### Which judge runs — `judge = f(answer_input, question_kind)`

The judge is resolved **per question**, not picked once per session and not a
free per-question toggle. The only session knob is the **answer-input axis**:

- **spoken** → always `Moderator` (nothing to match; `submission = None`).
- **typed** → `Auto` if the question kind is auto-matchable (MC / numeric), else
  `Moderator` for open kinds.
- **hybrid** → a moderator may override any `Auto` verdict (the revision path).

**Presence (co-located vs remote) is not a session mode.** It falls out of grants
+ projection already: if a `Present` screen is connected, players' views can omit
the question text; if remote, each player's view includes it. A view-population
choice, not runtime state. So "online" is **not** a mode — only answer-input
(spoken / typed / hybrid) is.

This means the same machine plays the full spectrum with different sections
populated: a fully-vocal game is just **buzzer + moderator** (`Buzz` only, no
typed answer, `Moderator` judge), a fully-typed game is buzzer + `answer_input`
+ `Auto`, and hybrids mix per question.

## Timers

Two distinct clocks, both expressed as **deadline timestamps** in `GameState`
(not countdowns), so any reconnecting or late-joining socket computes remaining
time as `deadline - now` — a consistent clock for every view, and forward-
compatible with the future SQLite path.

- **Question timer** — how long the question stays open before expiring
  unanswered.
- **Answer timer** — once a player has buzzed, how long they have to answer
  before forfeit.

The room task's `select!` races `mpsc recv` against `sleep_until(deadline)`.

### Moderator overrides

- **Extend** — `Command::ExtendTimer { deadline_delta }` pushes the deadline
  out; rebroadcast snapshot updates every view's clock for free.
- **Overrule the ring** — timer expiry is **not** final-and-instant. It resolves
  automatically as an **auto-verdict on the judgment log** (forfeit/expire), and
  a moderator overturns it via the **same verdict-revision path** as any other
  ruling. No separate `RungOut` pending state — expiry-overrule and
  verdict-revision are one mechanism. `ponytail:` timer expiry = auto-verdict;
  mod overrule reuses the revision path.

## Decision summary

| Decision | Choice | Deferred / future |
| --- | --- | --- |
| Backend owner | Rust + axum owns data + runtime | — |
| Types | `ts-rs` Rust → TS, Rust is source of truth | — |
| Transport | REST (cold content) + WS (hot state) | — |
| Concurrency | actor-per-room: owning task + mpsc in + broadcast out | — |
| Room registry | `DashMap<JoinCode, RoomHandle>`, self-reaping | — |
| Persistence | in-memory only | SQLite snapshot/restore for crash recovery |
| Room identity | single 6-char join code, regen on collision | opaque id only if needed |
| Auth gates | creation secret (config), optional join password (UI) | entry throttling |
| Reconnect | opaque token in localStorage, slot persists | — |
| State delivery | full snapshot, tagged-enum envelopes | deltas only if state outgrows a frame |
| Authorization | capabilities (`Play`/`Present`/`Moderate`), compose by union | — |
| Trust boundary | single `project(state, grants)`, server strips secrets | — |
| Grant assignment | joiner = `{Play}`; others via moderator-only `Grant` | — |
| Gamemode | grid_quiz hardcoded | extract `Gamemode` trait at #2 |
| Buzz/order | open-floor first-buzz + lockout (gamemode default, mod override) | turn-order, rotating announcer |
| Adjudication | `(Gamemode × Judge)`, append-only judgment log, `score = fold(log)` | — |
| v1 judges | `Auto` + `Moderator` behind a `Judge` trait | Quorum, Llm |
| Judge selection | `f(answer_input, question_kind)`; spoken/typed/hybrid | — |
| Timers | two deadline-timestamp clocks; mod extend / overrule via revision | — |
