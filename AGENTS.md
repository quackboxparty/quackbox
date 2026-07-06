# Quackbox — Agent Guide

## What this is

Quackbox is a **self-hostable, multi-gamemode, open quiz platform** —
think Kahoot/ClassQuiz but with multiple gamemodes (classic, battle royale,
survival, music quiz, quiz duel/jeopardy, who wants to be a millionair, higher or lower, …) sharing a single pool of community-
contributed, translatable questions.

Content (questions, packs, tags, media) lives in the repo as
human-editable YAML and is PR-reviewable. The runtime loads, validates,
merges translation overlays, and serves it to gamemodes that declare what
content they accept.

## Status

> **Backend is Rust + axum**; SvelteKit is a pure static frontend (no server, no
> remote functions). The data layer lives in Rust (`api/src/data/`); `ts-rs`
> exports Rust types to TS. The legacy TS data layer (`src/lib/server/data/`,
> `src/lib/schemas/`) and its valibot/effect schemas have been **removed** — Rust
> reached parity. **`docs/architecture.md` is the canonical runtime reference**
> (backend ownership, transport, concurrency, game-session model);
> `docs/data-model.md` remains canonical for content shape.

Data layer implemented in Rust: types, garde validation, YAML loader, cross-file validation, pool query engine, board builder, and example dataset (20 questions, German overlays, 1 pack, grid_quiz gamemode). Game runtime and UI not yet built. JSON Schema export for editor YAML LSP is not yet re-sourced from Rust (the old TS-generated `schemas/*.json` were removed).

## Tech stack

**Backend (Rust):**

- **Rust + axum** (`api/`) — data layer + live game runtime. Serves the static
  frontend and the API (REST for cold content, WebSocket for live game state).
- **`garde`** for validation, **`serde_yaml`** for content parsing.
- **`ts-rs`** exports Rust types to TS — single source of truth for shared types.
- **`tokio`** for the per-room actor concurrency model.

**Frontend (SvelteKit, static):**

- **SvelteKit 2 + Svelte 5** (runes), **`@sveltejs/adapter-static`** — no server,
  no SSR, built to `build/` and served by the Rust backend.
- **TypeScript** (strict), **Vite** for build/dev.
- **Paraglide JS** (inlang) for UI i18n; messages in `messages/{en,de}.json`.
- **Vitest** + **Playwright** for tests; **ESLint + Prettier**.
- **pnpm** workspaces.

Runtime concurrency: each game room is an isolated `tokio` task (mpsc in,
broadcast out); state streams to clients over WebSocket as role-specific
projections. See `docs/architecture.md`.

## Layout (current + planned)

```
api/                # Rust backend
  src/
    main.rs         # axum: load data, serve build/ + API
    config.rs
    data/           # loader, validate, query, board, error, types/
src/                # SvelteKit static frontend
  lib/
    paraglide/      # generated UI i18n runtime — do not edit
    themes/         # CSS theme tokens + per-theme stylesheets
    components/     # shared Svelte components
  routes/           # SvelteKit routes
  hooks.ts          # paraglide locale handling
messages/           # paraglide UI strings (en, de)
project.inlang/     # paraglide config
docs/
  architecture.md   # canonical runtime reference
  data-model.md     # canonical content reference
  glossary.md       # domain vocabulary
  decisions/        # ADRs
data/               # content: questions/, i18n/, packs/, tags/, media/
gamemodes/          # grid_quiz/ with manifest.yaml + boards/
```

## Architectural rules (from data-model.md)

- **Three layers, never mixed:** Questions (raw facts) · Packs (curated
  lists / filters) · Gamemodes (rules + presentation).
- **Kinds + variants** on questions, not one-type-per-question. Same fact
  plays as MC, T/F, open, numeric_input, range, order, etc.
- **Correctness lives on data** (`correct: true`, `position: N`,
  `answer: <num>`) — no separate answer key, no desync possible.
- **Translation overlays mirror canonical shape** under `data/i18n/<lang>/`.
  Overlays may only touch translatable fields; schema rejects edits to
  `correct`, `position`, numeric `answer`, `tolerance`, `min/max/step`.
- **Tags are `category:slug`** with closed-enum categories
  (`subject`, `difficulty`, `audience`, `region`, `format`, `warning`).
  Registry split one file per category under `data/tags/`.
- **Media refs use `prefix:value`** (`local:`, `url:`, `youtube:`),
  same shape as tags.
- **Licenses are SPDX from an allowlist**, schema-validated.
- **IDs are public API** — `q_<slug>`, `pack_<slug>`, `board_<slug>`, gamemode bare slug.
  No rename/delete without deprecation marker.
- **Validation runs at:** CI (validate step) and runtime (server start, via
  `cargo run` loading `../data`). Editor YAML LSP is pending a Rust-sourced
  JSON Schema export. Same Rust schema definitions everywhere.

When in doubt about content shape, **read `docs/data-model.md`**; for the
runtime, **read `docs/architecture.md`** — they are the spec, this file is just
orientation.

## Scripts

```sh
pnpm dev              # vite dev server
pnpm build            # production build
pnpm preview          # preview built app
pnpm check            # svelte-kit sync + svelte-check
pnpm lint             # prettier --check + eslint
pnpm format           # prettier --write
pnpm test:unit        # vitest
pnpm test:e2e         # playwright (auto-installs browsers)
pnpm test             # unit + e2e
```

Backend (Rust), from `api/`:

```sh
cargo run             # load ../data (validates, logs issues), serve ../build + API
cargo test
```

Planned (not yet implemented): `new-question` scaffolding, Rust JSON Schema
export for editor YAML LSP.

## Conventions

- Svelte 5 runes (`$state`, `$derived`, `$effect`), not legacy reactive `$:`.
- Frontend talks to the backend over REST (cold content) + WebSocket (live game
  state); no SvelteKit server logic (the adapter is static).
- Schema definitions are the source of truth and live in Rust
  (`api/src/data/types/`, validated by `garde`); TS types are generated via
  `ts-rs`, never hand-edited.
- Rust code uses `Result` / `thiserror` idiomatically for fallible paths.
- No comments restating what code does; comment only non-obvious _why_.
- Don't edit `src/lib/paraglide/**` — generated.
- Read the task list at session start from the `quackbox` notebook in `nb`:
  `nb todos quackbox:`. Priorities are tags (`p1`/`p2`/`p3`), projects are tags
  (`runtime`, `frontend`, `tooling`, `gamemodes`). Each todo body holds
  context + file refs; large items carry `--task` subtasks (see them with
  `nb todos quackbox:tasks <id>`). Complete with `nb todo do quackbox:<id>`
  (or `nb todo do quackbox:<id> <task#>` for one subtask).

## Implementation order (from data-model.md §Implementation order)

Data layer (schemas, loader, validation, query, board, example dataset, first
gamemode) is done and ported to Rust (`api/src/data/`); the legacy TS layer has
been removed. See `docs/data-model.md` for details. Remaining:

1. ✅ Data layer in Rust: types, loader, cross-file validation, query, board.
2. ✅ Example dataset (20 questions, German overlays, 1 pack, grid_quiz).
3. ✅ First gamemode: `grid_quiz` (manifest + boards; no runtime wiring yet).
4. ○ Game runtime (rooms, WebSocket, scoring) — see `docs/architecture.md`.
5. ○ Second gamemode to validate gamemode-agnostic claim.
6. ○ `new-question` scaffolding script.
7. ○ Rust JSON Schema export for editor YAML LSP.

## Open questions

Tracked in `docs/data-model.md` §Open questions — community pack pipeline,
question `revision` field, RTL support, web editor UI, duplicate-fact
detection, overlay merge semantics, gamemode schema export.
