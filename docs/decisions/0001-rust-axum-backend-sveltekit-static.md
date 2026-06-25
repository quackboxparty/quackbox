# Rust + axum backend; SvelteKit becomes a pure static frontend

We moved the backend from a SvelteKit server (TS, valibot, remote functions) to
**Rust + axum**, which owns content loading/validation/query/board and the live
game runtime. SvelteKit is now built static (`adapter-static`) and served by
axum; `ts-rs` exports Rust types to TS, making Rust the single source of truth
for all shared types.

Chosen over keeping the data layer in TS (or running both) because two
validators guarantee the desync the data model explicitly forbids
(`data-model.md`: "no desync possible"), and the realtime game runtime
(per-room actor tasks, WebSocket fan-out, timers) is a better fit for Rust/tokio
than SvelteKit remote functions. The cost: the existing TS data layer
(`src/lib/server/data/`, `src/lib/schemas/`) and remote-function plan are
discarded, and contributors now need Rust. Full architecture in
`docs/architecture.md`.

This supersedes the SvelteKit-server/remote-function design described in
`AGENTS.md` (which now carries a migration banner) and `data-model.md`'s
implementation-order step 10.
