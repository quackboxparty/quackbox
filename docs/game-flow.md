# Game Flow — Host & Player Journeys

> Status: **design sketch**. Covers the UX flow from landing on the site through
> gameplay. Data model is in `data-model.md`; this doc covers the interactive
> layer on top.

## Screens

### 1. Home / Start

The first thing a user sees.

- **Quackbox logo + tagline**
- **Host Game** button (primary CTA)
- **Join Game** section: text field for room code + submit, inline on this
  screen — no separate page

Join is currently a no-op placeholder; the input field is there to validate the
UX shape.

### 2. Gamemode Selection

After clicking "Host Game".

Grid of gamemode cards. Each card shows:

- **Preview image** — decorative illustration (not a screenshot). Stored in
  `gamemodes/<id>/preview.webp` (or SVG). If missing, render a colored
  placeholder with the gamemode icon.
- **Name** — from manifest `name`
- **Short description** — from manifest `description`
- **Player count** — derived from `requires.min_players` / `requires.max_players`

The grid should work with 1–8 gamemodes. Currently only `grid_quiz` exists.

Selecting a card → advance to settings for that gamemode.

### 3. Game Configuration

Game configuration has two layers that are always present, regardless of
gamemode:

1. **Content** — _what_ is played (boards, packs, question lists). Varies by
   gamemode — grid quiz picks boards, other gamemodes pick packs or question
   lists.
2. **Rules** — _how_ the game plays (timer, buzz-in, display mode, theme).

Together, content + rules form a **game config** that can be saved and shared
(see §Game configs).

#### 3a. Content Selection

Gamemode-specific. Each gamemode defines what "content" means.

**Grid Quiz** — board selection:

- Pick **one or two boards** that are played together (e.g. a "capitals" board
  - a "science" board). Two boards are played on after the other and the second board should give more points.
- Boards are loaded from `gamemodes/grid_quiz/boards/*.yaml`. Currently only
  `school` exists.
- MVP: select from pre-made boards only. A visual board editor (pick
  categories, swap individual questions, compose custom categories) is planned
  for a later iteration — see §Board editor.

**Other gamemodes** (future) — pack / question selection:

- Gamemodes like `classic` or `battle_royale` would pick one or more packs,
  or a flat list of question IDs. Exact UI TBD per gamemode.

#### 3b. Game Rules

Settings are **ad-hoc per gamemode** for now (not declared in the manifest
schema). Each gamemode's rules section is a Svelte component. Common patterns
may be extracted later.

**Grid Quiz rules:**

| Setting                  | Type             | Default            | Notes                                                                                     |
| ------------------------ | ---------------- | ------------------ | ----------------------------------------------------------------------------------------- |
| **Time per question**    | number (seconds) | 30                 | Countdown after buzz-in. 0 = no timer.                                                    |
| **Point values visible** | toggle           | on                 | Show/hide point values on the board.                                                      |
| **Buzz-in mode**         | select           | `free_for_all`     | See §Buzz-in modes.                                                                       |
| **Display mode**         | select           | `screen_is_player` | See §Display & admin modes.                                                               |
| **Default language**     | select           | `en`               | Game-level default for i18n. Players can override per-device (see §Per-player overrides). |
| **Default theme**        | select           | `modern`           | Game-level default theme. Players can override per-device (see §Per-player overrides).    |

#### Display & admin modes

The "big screen" can serve three roles. This is the **display mode** setting:

| Mode                | Description                                                                                                                                                                                                        |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `screen_is_player`  | The big screen is a full participant — it shows the game board and is also a player (e.g. host plays along on the TV).                                                                                             |
| `screen_is_display` | The big screen shows the game board only, is **not** a player. Players interact on their own devices. An admin must be designated from the player list to control game flow (advance questions, mark right/wrong). |
| `screen_is_admin`   | The big screen shows the board **and** has admin controls built in. No separate admin player needed. The screen is not a player. Think classroom / bar trivia — one person at the laptop runs the whole thing.     |

**Admin controls** (available in `screen_is_display` via designated player, or
directly in `screen_is_admin`):

- Advance to next question / next round
- Mark answer as correct or incorrect (override if needed)
- Skip question
- Pause / resume timer
- End game early

In `screen_is_player` mode, no admin controls exist — game follows normal
player flow with automated scoring.

#### Buzz-in modes

| Mode             | Description                                                                                                                                                                     |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `free_for_all`   | Any player can buzz. First buzz locks out others. If wrong, remaining players can re-buzz for half points.                                                                      |
| `turn_based`     | Players take turns selecting a tile and answering. No buzz-in.                                                                                                                  |
| `turn_then_buzz` | Players take turns selecting a tile. After the active player answers wrong, the tile opens to buzz-in for half points. Combines turn-based selection with free-for-all re-buzz. |

Buzz-in semantics are gamemode-specific. Grid Quiz uses the half-points
re-buzz rule; other gamemodes may have no buzz-in, direct simultaneous
response, or elimination on wrong answer.

### 4. Lobby

After clicking "Create Game" on the settings screen.

- **Join code** — 6-character alphanumeric, prominently displayed at top
- **QR code** — next to join code, encodes the join URL
- **Player list** — auto-updates as players join; each player has an
  auto-generated name (see §Player identity)
- **Start Game** button (primary) — always visible
- **Start Early** subtext — "Start with N players now" shown when at least 1
  player has joined
- **Start Without Players** — available in `screen_is_admin` mode only; skips
  the lobby entirely or starts with 0 joined players, going straight to admin
  controls
- **Settings summary** — collapsible, shows chosen settings from previous screen

The lobby is a live-updating view — players appear in real-time as they join.
No page refresh needed.

#### Player identity

Players get an **auto-generated animal name** on joining:

- Format: `<emoji> <animal>` — e.g. 🦆 Ducky, 🐼 Panda, 🦊 Foxy, 🦉 Owly
- Randomly assigned from a curated list (no duplicates in the same game)
- Player can rename themselves at any time via an inline edit on their name
- Name is stored in the game session, not a persistent account (no auth for MVP)

The animal list lives in `src/lib/data/animals.ts` — a simple array of
`{ emoji, name }` pairs. Enough entries for at least 50 players without repeats.

#### Per-player overrides

Language and theme are set at the game level as defaults, but each player
can override them on their own device without affecting others:

- **Language** — a player can switch their interface and question language
  independently. The game default determines what new joiners see initially,
  but a German player can play in German while others play in English.
- **Theme** — same pattern. The host sets `neon` as the game default, but a
  player who prefers `modern-dark` can switch locally. The big screen always
  uses the game default.

These overrides are device-local (stored in browser localStorage or session).
They don't modify the game config.

#### Game configs

A game config = content selection + rules + default language + default theme.
Configs can be **saved and shared**:

- **Copy config** (MVP) — a "Copy config" button serializes the current config
  as a shareable string (JSON, base64, or a URL fragment). Anyone with the
  string can paste it to recreate the same game setup. No server-side storage.
- **Persisted configs** (future) — named presets stored server-side, browsable
  and reusable. Could include community-shared configs ("Bar Trivia Night",
  "Classroom Geography 101").

Config sharing is opt-in — the default flow is manual selection. No account
required for MVP.

#### Board editor (future)

A visual editor for building custom grid quiz boards:

- See the board layout (categories × point values) and what questions fill each
  slot.
- Pick categories from existing packs or tag filters.
- Swap individual questions (drag-drop or click-to-replace).
- Compose custom categories with hand-picked questions.
- Save the board as a new YAML file (download, or server-side save later).

Not in MVP. The editor will be a dedicated screen/flow, likely replacing the
content selection step when invoked.

### 5. Gameplay

Gameplay varies significantly by gamemode and display mode. Detailed gameplay
flow is gamemode-specific and will be documented per gamemode.

General principles:

- State is streamed via SvelteKit `query.live` — all clients see updates in
  real-time
- Players on phones see the **player view** (their answer input)
- The big screen shows the **board/public view** (question, scores, animations)
- Admin controls are an overlay on the host/display view, not a separate page

## Decisions

These were debated; the design above reflects the chosen path.

- **Join code inline on home screen** — not a separate "Join" page. One field,
  one button, stays on the start screen. Reduces navigation depth.
- **Content + rules as separate layers** — content (boards, packs) and rules
  (timer, buzz-in) are conceptually distinct and edited in separate sections of
  the config screen. Content is gamemode-specific; rules follow gamemode
  conventions. Together they form a saveable game config.
- **Ad-hoc settings per gamemode** — no manifest-level settings schema for now.
  Gamemodes define their own settings UI component. A common schema may emerge
  later once patterns stabilize.
- **Three display modes, not two** — the distinction between "screen is display"
  (needs a phone-wielding admin from the player list) and "screen is admin"
  (laptop at the front of a bar) is real and important. Merging them would force
  bar-trivia hosts to also be players, which they don't want.
- **Per-player language and theme overrides** — the game sets defaults, but each
  player can switch independently. No global lock — a German player shouldn't be
  forced to play in English because the host chose it.
- **Copyable game configs (MVP)** — serialize as a shareable string. No
  server-side persistence needed yet. The sharing primitive exists; storage is
  the later addition.
- **Animal names with emoji** — low-friction onboarding. No name prompt before
  joining, no blank names, instant visual identity. Renaming is opt-in.
- **Buzz-in: half points on re-buzz** — rewards speed but doesn't fully punish
  wrong fast answers. Grid-quiz-specific; other gamemodes define their own
  semantics.
- **Start without players** — the admin-only / bar-trivia flow needs this.
  Zero-player start means the admin drives everything from the display screen.
- **Board editor deferred** — MVP uses pre-made boards only. The visual board
  editor is a second-iteration feature; the content selection UI is designed to
  accommodate it later without restructuring.

## Open questions

1. **Join code format** — 6-char alphanumeric is common (Kahoot, ClassQuiz).
   Should we use a word-based code for easier verbal communication
   (e.g. "blue-duck-forty")? Trade-off: readability vs. collision space.
2. **QR code content** — encodes `https://<host>/join/<code>` (full URL). More
   convenient — players just scan and land on the join page. Host is known at
   generation time since the game server constructs the URL from its own origin.
3. **Spectator mode** — the manifest schema has `spectator_view`. Should the
   lobby support a "join as spectator" option? Likely yes, but deferred.
4. **Reconnection** — if a player's phone disconnects, can they rejoin with the
   same identity? Needs session token or local storage. Deferred until we have
   real-time working.
5. **Max players** — what's the practical limit? Depends on gamemode. Grid quiz
   could support 20+; battle royale might cap at 50. Manifest `max_players` is
   optional for now.
6. **Late join** — can a player join after the game has started? Gamemode-
   specific. Grid quiz probably yes (pick an unclaimed tile on next round).
   Battle royale probably no. Deferred.
7. **Teams** — we need to think about how to do teams probably at the player join screen.
8. **Config serialization format** — JSON? Base64-encoded JSON? URL fragment? Need
   to decide before implementing "Copy config". Should be compact enough to paste
   in a chat message.
9. **Two-board play** — when two boards are combined, how are categories merged?
   Concatenated? Interleaved? Host picks the order? Need to define before
   implementing multi-board selection.
