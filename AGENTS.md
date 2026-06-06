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

Early. Design sketch is in `docs/data-model.md` (canonical reference for
schema, i18n, tagging, packs, media, gamemodes). Code so far is the
default SvelteKit scaffold + paraglide; data layer / gamemodes not yet
implemented.

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
    schemas/        # valibot schemas (planned) — question, pack, gamemode, overlay
    data/           # loader: YAML → validate → merge overlays → index (planned)
    paraglide/      # generated UI i18n runtime — do not edit
  routes/           # SvelteKit routes
  hooks.ts          # paraglide locale handling
  hooks.server.ts
messages/           # paraglide UI strings (en, de)
project.inlang/     # paraglide config
docs/
  data-model.md     # canonical design doc — read first
data/               # content (planned): questions/, i18n/, packs/, tags/, media/
schemas/            # generated JSON Schemas (planned), committed
gamemodes/          # code per gamemode + manifest.yaml (planned)
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
- **Media refs use `prefix:value`** (`media:`, `url:`, `youtube:`),
  same shape as tags.
- **Licenses are SPDX from an allowlist**, schema-validated.
- **IDs are public API** — `q_<slug>`, `pack_<slug>`, gamemode bare slug.
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
```

Planned (not yet implemented): `pnpm gen:schemas`,
`pnpm validate-data`, `pnpm new-question`.

## Conventions

- Svelte 5 runes (`$state`, `$derived`, `$effect`), not legacy reactive `$:`.
- Server logic in remote functions, not ad-hoc `+server.ts` where
  avoidable — keeps validation (valibot) co-located with the call.
- New content schema = valibot first, JSON Schema generated, never
  hand-edited.
- No comments restating what code does; comment only non-obvious *why*.
- Don't edit `src/lib/paraglide/**` — generated.

## Implementation order (from data-model.md §Implementation order)

1. Valibot schemas in `src/lib/schemas/`.
2. `pnpm gen:schemas` JSON Schema export.
3. Data loader in `src/lib/data/`.
4. CI: `check && lint && test && validate-data`.
5. Example dataset proving the pipeline end-to-end.
6. First gamemode (`classic`) wired via remote functions + `query.live`.
7. Second gamemode (`battle_royale` or `music_quiz`) to validate the
   gamemode-agnostic claim.

## Open questions

Tracked in `docs/data-model.md` §Open questions — community pack pipeline,
question `revision` field, RTL support, web editor UI, duplicate-fact
detection, loader merge/filter/shuffle semantics.
