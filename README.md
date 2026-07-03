<p align="center">
  <img src="src/lib/assets/logo.svg" alt="Quackbox logo" width="200" />
</p>

# Quackbox

Self-hostable, multi-gamemode, open quiz platform — think Kahoot/ClassQuiz, but
with multiple gamemodes (classic, battle royale, survival, music quiz, jeopardy,
…) sharing a single pool of community-contributed, translatable questions.

Content (questions, packs, tags, media) lives in the repo as human-editable,
PR-reviewable YAML. A self-hosted instance runs the core game loop **offline**
after setup — set it up at home, play on a LAN, no internet required.

## Architecture

- **Rust + axum backend** (`api/`) — owns content loading/validation/query/board
  and the live game runtime (rooms over WebSocket). Single source of truth for
  shared types via `ts-rs`.
- **SvelteKit static frontend** (`adapter-static`) — built to `build/`, served
  by the Rust backend. No SvelteKit server, no SSR.

See the docs for the full picture:

- `docs/architecture.md` — runtime: backend ownership, transport, concurrency,
  game-session model. **Canonical runtime reference.**
- `docs/data-model.md` — content shape: schemas, i18n, tagging, packs, media,
  gamemodes, boards. **Canonical content reference.**
- `docs/game-flow.md` — host & player UX journeys.
- `docs/glossary.md` — domain vocabulary.
- `docs/decisions/` — architecture decision records.

## Frontend (SvelteKit)

```sh
pnpm install
pnpm dev          # vite dev server
pnpm build        # static build → build/
pnpm check        # svelte-kit sync + svelte-check
pnpm lint         # prettier --check + eslint
pnpm test:unit    # vitest
```

## Backend (Rust)

```sh
cd api
cargo run         # loads ../data (validates, logs issues), serves ../build + the API
cargo test
```

The data layer (content load/validate/query/board) lives in Rust
(`api/src/data/`); the legacy TS implementation has been removed now that Rust
reached parity.
