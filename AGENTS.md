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

> **Architecture migration in progress.** The backend is moving to **Rust +
> axum**; SvelteKit is becoming a pure static frontend (no server, no remote
> functions). The data layer is being ported from TS/valibot to Rust
> (`api/src/data/`); `ts-rs` exports Rust types to TS. The TS data layer
> (`src/lib/server/data/`, `src/lib/schemas/`) is **legacy**, to be removed at
> parity. The tech-stack and remote-function notes below predate this shift and
> apply only to the legacy TS side. **`docs/architecture.md` is the canonical
> runtime reference** (backend ownership, transport, concurrency, game-session
> model); `docs/data-model.md` remains canonical for content shape.

Data layer implemented (TS, being ported to Rust): schemas, JSON Schema export, YAML loader, cross-file validation, pool query engine, board builder, and example dataset (20 questions, German overlays, 1 pack, grid_quiz gamemode). Game runtime and UI not yet built.

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

**Legacy (being removed at parity):** the TS data layer
(`src/lib/server/data/`) and valibot schemas (`src/lib/schemas/`), superseded
by the Rust port in `api/src/data/`.

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
    schemas/        # LEGACY valibot schemas — removed at Rust parity
    server/data/    # LEGACY TS data layer — removed at Rust parity
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
schemas/            # generated JSON Schemas (committed) — 16 files
gamemodes/          # grid_quiz/ with manifest.yaml + boards/
scripts/            # gen-schemas.ts, validate-data.ts (legacy TS tooling)
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
- **Validation runs at three points:** editor (YAML LSP), CI (validate-data),
  runtime (server start). Same schema definitions everywhere.

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
pnpm gen:schemas     # JSON Schema export (writes schemas/) — legacy TS tooling
pnpm validate-data   # load + validate all YAML data (requires nix develop) — legacy TS tooling
```

Backend (Rust), from `api/`:

```sh
cargo run             # load ../data, serve ../build + API
cargo test
```

Planned (not yet implemented): `new-question` scaffolding.

> The `neverthrow` guidance below applies to the **legacy TS data layer** only.
> Rust code uses `Result` / `thiserror` idiomatically.

## neverthrow usage

We use `neverthrow` (`Result` / `ResultAsync`) for operations where success and
failure need **different handling** — the caller branches on the outcome.

### When to use

- **Parse/validation boundaries** — `parse()` in `util.ts` returns
  `ResultAsync<Schema, LoadIssue[]>` because callers either use the parsed
  value or collect issues.
- **Init functions** — `loadDataset()`, `initDataset()` return
  `ResultAsync` so catastrophic failures (unreadable dir, broken YAML)
  flow through the error channel, not as thrown exceptions.
- **Accessors with precondition** — `getDataset()` returns
  `Result<LoadedDataset, 'not_initialized'>` instead of throwing,
  forcing callers to handle the uninitialized case at the type level.

### When NOT to use

- **Diagnostic accumulation** — when you always collect both files AND
  issues and never branch (e.g. `walkYaml`), use plain `{ files, issues }`
  return. Wrapping in Result adds ceremony with no real benefit.
- **Simple existence checks** — `try { await stat(f) } catch { continue }`
  is clearer than a `ResultAsync` chain when you just skip on ENOENT.
- **Side-effect-only error handling** — use `.orTee()` not `.match()`
  when only the error path matters (avoids empty ok callback, lint noise).

### Key API patterns

- `fromAsyncThrowable` over `ResultAsync.fromPromise` — catches sync throws
  AND promise rejections. Use it for wrapping async functions that could
  throw before returning a promise.
- `.map()` accepts async callbacks (`Promise<U>`) with no type change.
- `.andTee()` / `.orTee()` for side effects that must not affect the
  result (logging, cleanup).
- `_unsafeUnwrap()` in tests — neverthrow's recommended test pattern,
  cleaner than `.match()` with throw.

### Never guess at neverthrow API

**Always read the neverthrow docs before using an API.** Don't assume a
method exists (e.g. `.finally()` does not exist on `ResultAsync`) or
guess at callback signatures (e.g. `.match()` callbacks are sync-only,
can't `await` inside them; `.andThen()` already narrows to Ok so chaining
`.andThen().match()` with redundant ok unwrap is a smell). Check first.

Docs: https://github.com/supermacro/neverthrow

## Conventions

- Svelte 5 runes (`$state`, `$derived`, `$effect`), not legacy reactive `$:`.
- Frontend talks to the backend over REST (cold content) + WebSocket (live game
  state); no SvelteKit server logic (the adapter is static).
- Schema definitions are the source of truth; JSON Schema is generated, never
  hand-edited. (Definitions live in Rust `api/src/data/types/` post-port;
  valibot `src/lib/schemas/` until then.)
- No comments restating what code does; comment only non-obvious _why_.
- Don't edit `src/lib/paraglide/**` — generated.

## Implementation order (from data-model.md §Implementation order)

Steps 1–9 done on the legacy TS data layer. See `docs/data-model.md` for details.

1. ✅ Schema definitions (legacy valibot in `src/lib/schemas/`).
2. ✅ JSON Schema export.
3. ✅ Data loader.
4. ✅ CI scripts: validate-data + schema export.
5. ✅ Example dataset (20 questions, German overlays, 1 pack, grid_quiz).
6. ✅ Pool query engine + board builder.
7. ✅ First gamemode: `grid_quiz` (manifest + boards; no runtime wiring yet).
8. ✅ Tests: 53 across 8 files.
9. ○ Port the data layer to Rust (`api/src/data/`); remove the legacy TS layer.
10. ○ Game runtime (rooms, WebSocket, scoring) — see `docs/architecture.md`.
11. ○ Second gamemode to validate gamemode-agnostic claim.
12. ○ `new-question` scaffolding script.

## Open questions

Tracked in `docs/data-model.md` §Open questions — community pack pipeline,
question `revision` field, RTL support, web editor UI, duplicate-fact
detection, overlay merge semantics, gamemode schema export.
