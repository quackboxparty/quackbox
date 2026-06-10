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

Data layer implemented: valibot schemas, JSON Schema export, YAML loader, cross-file validation, pool query engine, board builder, and example dataset (20 questions, German overlays, 1 pack, grid_quiz gamemode). Runtime wiring (SvelteKit remote functions) and UI not yet built. Design doc is `docs/data-model.md` (canonical reference for schema, i18n, tagging, packs, media, gamemodes, boards).

## Tech stack

- **SvelteKit 2 + Svelte 5** (runes), Node adapter (`@sveltejs/adapter-node`)
- **TypeScript** (strict)
- **Vite 8** for build/dev
- **Valibot** for schemas — single source of truth for TS types **and**
  JSON Schema (via `@valibot/to-json-schema`) exported to `schemas/*.json`
  for YAML LSP editor support
- **YAML** (`yaml` package) for all content files
- **Paraglide JS** (inlang) for UI i18n; messages in `messages/{en,de}.json`
- **Vitest** (unit + component via `vitest-browser-svelte`) and
  **Playwright** for e2e
- **ESLint + Prettier** (+ `prettier-plugin-svelte`, `eslint-plugin-svelte`)
- **pnpm** workspaces

Runtime concurrency uses SvelteKit **remote functions** (`query.live`,
`command`, `form`) to stream game state to clients. Data layer has zero
coupling to SvelteKit.

## Layout (current + planned)

```
src/
  lib/
    schemas/        # valibot schemas — question, question-overlay, pack, pack-overlay, gamemode, board, tag, media, common
    data/           # loader: YAML → validate → build indexes → cross-file checks → pool query → board builder
    paraglide/      # generated UI i18n runtime — do not edit
    themes/         # CSS theme tokens + per-theme stylesheets
    components/     # shared Svelte components
  routes/           # SvelteKit routes
  hooks.ts          # paraglide locale handling
  hooks.server.ts
messages/           # paraglide UI strings (en, de)
project.inlang/     # paraglide config
docs/
  data-model.md     # canonical design doc — read first
data/               # content: questions/, i18n/, packs/, tags/, media/
schemas/            # generated JSON Schemas (committed) — 16 files
gamemodes/          # grid_quiz/ with manifest.yaml + boards/
scripts/            # gen-schemas.ts, validate-data.ts
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
- **Validation runs at three points:** editor (YAML LSP), CI
  (`pnpm validate-data`), runtime (server start). Same valibot schemas
  everywhere.

When in doubt about content shape, **read `docs/data-model.md`** — it is
the spec, this file is just orientation.

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
pnpm gen:schemas     # valibot → JSON Schema export (writes schemas/)
pnpm validate-data   # load + validate all YAML data (requires nix develop)
```

Planned (not yet implemented): `pnpm new-question`.

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
- Server logic in remote functions, not ad-hoc `+server.ts` where
  avoidable — keeps validation (valibot) co-located with the call.
- New content schema = valibot first, JSON Schema generated, never
  hand-edited.
- No comments restating what code does; comment only non-obvious _why_.
- Don't edit `src/lib/paraglide/**` — generated.

## Implementation order (from data-model.md §Implementation order)

Steps 1–9 done. See `docs/data-model.md` for details.

1. ✅ Valibot schemas in `src/lib/schemas/`.
2. ✅ `pnpm gen:schemas` JSON Schema export.
3. ✅ Data loader in `src/lib/data/`.
4. ✅ CI scripts: `validate-data` and `gen:schemas`.
5. ✅ Example dataset (20 questions, German overlays, 1 pack, grid_quiz).
6. ✅ Pool query engine + board builder.
7. ✅ First gamemode: `grid_quiz` (manifest + boards; no runtime wiring yet).
8. ✅ Tests: 53 across 8 files.
9. ○ SvelteKit runtime wiring (remote functions).
10. ○ Second gamemode to validate gamemode-agnostic claim.
11. ○ `pnpm new-question` scaffolding script.

## Open questions

Tracked in `docs/data-model.md` §Open questions — community pack pipeline,
question `revision` field, RTL support, web editor UI, duplicate-fact
detection, overlay merge semantics, gamemode schema export.
