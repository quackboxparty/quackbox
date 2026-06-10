# Effect TS — Quackbox Fit Assessment

Research artifact and decision log. **Conclusion: adopt Effect as the unified
paradigm for schemas, errors, and the runtime — with a hard bundle boundary
between server and client.**

## What Effect is

TypeScript library for **typed, composable, structured concurrency** with
first-class effects. One type signature carries three things:

```ts
type Effect<A, E, R> = ...
//   A = success value
//   E = typed expected errors
//   R = required services (resolved by Layers at the boundary)
```

Replaces `Promise<T>` + try/catch + manual dependency injection + manual
concurrency in one type. Major modules:

| Module                         | Purpose                                                        |
| ------------------------------ | -------------------------------------------------------------- |
| `Effect`                       | Core: typed success + typed error + required services          |
| `Layer`                        | Recipes for services. Compose. Memoize. Swap for tests.        |
| `Stream`                       | Backpressured async stream. `Stream.fromPubSub` etc.           |
| `Queue` / `PubSub`             | Concurrent message passing. PubSub = broadcast.                |
| `Ref` / `SynchronizedRef`      | Lock-free concurrent mutable state.                            |
| `Fiber`                        | Lightweight cooperative thread. Cancellation propagates.       |
| `Schedule`                     | Retry/backoff/cron. `Schedule.exponential`, `Schedule.spaced`. |
| `Scope`                        | Resource lifecycle. Finalizers on exit/panic/cancel.           |
| `Schema`                       | Parse-don't-validate. Derive TS types, JSON Schema, Arbitrary. |
| `Logger` / `Tracer` / `Metric` | Structured observability with spans.                           |
| `Config`                       | Typed env-var / config-file loading.                           |

## Effect vs the current valibot + neverthrow stack

The current stack is valibot for parsing/validation and neverthrow for
typed error handling. Effect subsumes both with `effect/Schema` and
`Effect<A, E, R>` respectively.

| Concern              | valibot + neverthrow                 | Effect                                                                                       |
| -------------------- | ------------------------------------ | -------------------------------------------------------------------------------------------- |
| Schema / validation  | valibot (`v.strictObject`, etc.)     | `effect/Schema` (`S.Struct`, `S.Union`, ...)                                                 |
| Async errors         | `ResultAsync<T, E>` (separate type)  | `Effect<A, E, R>` (one type)                                                                 |
| Sync errors          | `Result<T, E>`                       | `Either<A, E>` (or `Effect.succeed` + `fail`)                                                |
| Dependencies         | Pass manually                        | Type-level via `R` + `Layer`                                                                 |
| Resources / cleanup  | Manual                               | `Scope` + guaranteed finalizers                                                              |
| Concurrency          | None                                 | Fibers, `Queue`, `PubSub`, `Semaphore`                                                       |
| Streams              | None                                 | `Stream` module, backpressured                                                               |
| Cancellation         | `AbortSignal` opt-in                 | Built-in, propagates through fibers                                                          |
| Retry / timeout      | Manual                               | `Effect.retry(Schedule)`, `Effect.timeout`                                                   |
| Logging / tracing    | pino (separate)                      | `Effect.Logger` (built-in) — spans, annotations, fiber IDs, OTEL via `@effect/opentelemetry` |
| JSON Schema export   | `@valibot/to-json-schema` (external) | `S.JSONSchema.make` (built-in)                                                               |
| Branded types        | Manual                               | `S.Brand` first-class                                                                        |
| Property-test data   | None                                 | `S.Arbitrary` module                                                                         |
| Class APIs           | None                                 | `S.Class` (data class + schema + Equal + Hash)                                               |
| Bundle cost (server) | valibot ~600B–5KB, neverthrow ~4KB   | `effect` ~50KB, `effect/Schema` ~15–20KB                                                     |
| Bundle cost (client) | Small                                | Larger — discipline required (see Bundle)                                                    |
| Learning curve       | Low (each tool)                      | High (one tool, more vocabulary)                                                             |

**Trade-off is real:** adopting Effect means accepting the steeper
learning curve in exchange for one paradigm across schemas, errors, and
concurrency. Decision: worth it for a long-lived project that will
have a runtime, server logic, and a real-time game loop. Three tools
for three concerns is the principle — Effect covers all three,
valibot+neverthrow only covers two.

For prior neverthrow knowledge: every neverthrow pattern has a direct
Effect analog. `fromAsyncThrowable` ≈ `Effect.tryPromise` with
`catchAll`. `andThen` ≈ `flatMap`. `orElse` ≈ `catchAll`. See the
official mapping at
<https://effect.website/docs/additional-resources/effect-vs-neverthrow/>.

## Quackbox's current profile (pre-migration)

### Data layer

- **16 valibot schemas** + `@valibot/to-json-schema` script in CI →
  `schemas/*.json` for YAML LSP.
- **neverthrow everywhere** in the loader. `parse()` →
  `ResultAsync<v.InferOutput, LoadIssue[]>`. `loadDataset()` →
  `ResultAsync<LoadedDataset, LoadIssue[]>`. `initDataset()` →
  `ResultAsync<LoadedDataset, LoadIssue[]>`.
- **pino** logger with `childLogger('dataset')` for subsystem scoping.
  Comments in `logger.ts` already reserve space for OTEL trace/span
  injection. (Replaced by `Effect.Logger` after migration; the OTEL
  injection point moves to `@effect/opentelemetry`.)
- **Diagnostic accumulation pattern** is deliberate and load-bearing:
  `walkYaml()` returns `{ files, issues }` (not `Result`). Quackbox
  AGENTS.md: _"We use neverthrow where success and failure need
  different handling. When you always collect both files AND issues and
  never branch, use plain `{ files, issues }`."_
- Pure functions over registries. No I/O beyond YAML read + media `stat`.

### Runtime (planned, step 10)

SvelteKit **remote functions** (`query.live`, `command`, `form`) stream
game state to clients. From the data-model doc:

> Game sessions use `query.live` to stream state to players. The
> SvelteKit remote-function wiring is not yet built.

Multi-gamemode: grid_quiz (Jeopardy), battle_royale, survival,
music_quiz, quiz_duel, etc. Each has its own scoring, timing, state
machine.

## The decision: unify on Effect

Three concerns, three tools, zero overlap:

| Concern                | Tool              | Why                                                                                              |
| ---------------------- | ----------------- | ------------------------------------------------------------------------------------------------ |
| Shape & validation     | `effect/Schema`   | parse-don't-validate, JSON Schema export, branded types, S.Class for domain models               |
| Error handling & async | `Effect<A, E, R>` | typed errors, structured concurrency, fibers, resources                                          |
| Observability          | `Effect.Logger`   | spans/annotations from fiber tree, OTEL via `@effect/opentelemetry`, replaces pino + pino-pretty |
| UI state (client)      | Svelte 5 runes    | framework layer, must stay Svelte-flavored                                                       |

Replaces: valibot → `effect/Schema`. neverthrow → `Effect<A, E, R>`.
pino → `Effect.Logger`.

## The accumulator pattern survives the migration

The data layer's load functions return values that carry `issues` as a
field, not as a typed error. This is anti-typical-Effect ("fail fast
on the first typed error in `E`") but it matches the loader's
domain: "we tried to read 50 files, here are 47 that worked and 3
that didn't, plus the registry we built from the 47."

Effect supports this cleanly. The success type is `LoadedDataset`
which contains `issues: LoadIssue[]`. The error type is `never` (we
never fail the whole load because of a few bad files). The
accumulator is captured in a closure inside `Effect.gen`:

```ts
const loadQuestions = (dataDir: string): Effect.Effect<Registry<Question>, never, FileSystem> =>
	Effect.gen(function* () {
		const issues: LoadIssue[] = [];
		const items = new Map<string, Entry<Question>>();

		const { files, issues: walkIssues } = yield* walkYaml(join(dataDir, 'questions'));
		issues.push(...walkIssues);

		yield* Effect.forEach(
			files,
			(file) =>
				parse(file, QuestionFile).pipe(
					Effect.tap((qs) =>
						Effect.sync(() => {
							for (const q of qs) {
								if (items.has(q.id)) issues.push({ file, message: `dup ${q.id}` });
								else items.set(q.id, { file: rel(file), item: q });
							}
						})
					),
					Effect.tapError((errs) => Effect.sync(() => issues.push(...errs)))
				),
			{ concurrency: 'unbounded' }
		);

		return items;
	});
```

`Effect.tap` runs a side effect and returns the original value.
`Effect.tapError` does the same on the error path. `Effect.forEach`
with `concurrency: 'unbounded'` replaces the current
`Promise.all` + per-result `match()`.

Catastrophic failures (data dir unreadable for non-ENOENT reasons)
are a different story and become typed errors. Most of the loader
keeps accumulating; the truly unrecoverable cases throw or return
`Effect.fail`.

## Bundle discipline: types on client, values on server

This is the one rule that makes the migration safe.

| Import kind                                                                           | Bundle cost                                   | Use case                                       |
| ------------------------------------------------------------------------------------- | --------------------------------------------- | ---------------------------------------------- |
| `import type { Question } from '$lib/schemas/question'`                               | **0** (TypeScript erases types)               | Client components, props, type-only usage      |
| `import { Question } from '$lib/schemas/question'` where `Question = S.Struct({...})` | **0 if unused at runtime** (Vite tree-shakes) | Server-side parsing, validation, decode/encode |
| Same import + `Question.parse(data)` in client code                                   | **~15–20 KB gz** (`effect/Schema` runtime)    | Avoid unless necessary                         |

SvelteKit `query` / `command` / `form` auto-generate typed RPC
clients. Server validates with schema, client gets typed return.
**Client almost never needs the schema at runtime.** Use it as a
type, not a value.

### Enforcing the rule

`tsconfig.json` has `verbatimModuleSyntax: true` (added as part of
this migration prep). TypeScript requires explicit `import type`
syntax for type-only imports. Catches accidental value imports at
lint time.

### What stays in the client bundle

- TS types from `$lib/schemas/` (zero cost)
- `$lib/components/`, `$lib/themes/`, `$lib/paraglide/`
- SvelteKit framework code

### What stays server-side

- `$lib/server/` — anything imported only by server code
- `*.server.ts` — SvelteKit convention, Vite tree-shakes from client
- `$lib/data/` (post-migration) — uses `node:fs/promises`, server-only
- `$lib/runtime/` (future) — Effect runtime, `ManagedRuntime`, `Layer`
- The `$lib/data/store.svelte.ts` file is split: runes for UI state
  live in `.svelte.ts`; the Effect runtime lives in `$lib/server/`
  and is reached through SvelteKit remote functions.

## Target layout

```
src/lib/
  schemas/                 # universal: S.Struct + type exports
                           # tree-shakable values, used server-side
  data/                    # server-only (Node fs): loader, validator
                           # migrates from neverthrow to Effect
  server/
    runtime/               # Effect runtime: Layers, ManagedRuntime
                           # Room, Player, GameEvent as S.Class
    data-init.ts           # server-side initDataset via Effect
  components/              # client: `import type` from $lib/schemas
  themes/                  # universal CSS
  paraglide/               # generated, do not edit
  server/
    observability/         # Effect Logger Layer (pretty dev, JSON prod)
                           # OTEL config when added
  (no $lib/logger.ts — pino replaced by Effect.Logger)
src/routes/
  *.remote.ts              # server: schema-validated commands
  *.svelte                 # client: typed props, no schema values
scripts/
  validate-data.ts         # Node CLI, uses Effect loader
  gen-schemas.ts           # uses S.JSONSchema.make
```

## Svelte 5 rune file as a boundary

`store.svelte.ts` is **universal** in Svelte 5 — it runs server (SSR)
and client (hydration). It cannot import the Effect runtime without
leaking it to the client. The split:

```ts
// src/lib/server/data-init.ts (server-only)
import { Effect, Exit, Layer, ManagedRuntime } from 'effect';
import { DataLoaderLive, loadDataset } from './loader';

const runtime = ManagedRuntime.make(DataLoaderLive);

export async function initDatasetServer(opts?: LoadOptions): Promise<LoadedDataset> {
	return runtime.runPromise(loadDataset(opts));
}
```

```ts
// src/lib/data/store.svelte.ts (universal, no Effect import)
import { getContext, setContext } from 'svelte';
import { initDatasetServer } from '$lib/server/data-init';

let dataset = $state<LoadedDataset | null>(null);
let loading = $state(false);

export async function initDataset(opts?: LoadOptions): Promise<void> {
	loading = true;
	dataset = await initDatasetServer(opts);
	loading = false;
}

export function getDataset(): LoadedDataset | null {
	return dataset;
}
```

Components call `initDataset()` and `getDataset()` — no Effect, no
`Result`, no schema values. The Effect boundary is a single line at
`$lib/server/data-init.ts`. **Effect at the bottom, plain TS at the
Svelte boundary.**

## What Effect would HURT (and the mitigations)

1. **Client bundle.** SvelteKit ships player UI to browsers. Effect
   core is ~50 KB gz; `effect/Schema` adds ~15–20 KB gz. Mitigation:
   `verbatimModuleSyntax: true` + server-only file conventions.
2. **Learning curve.** Solo dev, 1–2 month ramp for Effect's full
   vocabulary (`gen`, `yield*`, `Layer`, `Stream`, `Ref`, `Schedule`,
   `Fiber`, `Scope`, `Tracer`). Mitigation: start with loader
   migration (small surface: `Effect.gen`, `Effect.tryPromise`,
   `Effect.forEach`, `Effect.tap`); expand to runtime once familiar.
3. **Cognitive overhead.** Adding a fourth paradigm would be a smell;
   we're replacing two (valibot + neverthrow) with one (Effect). Net
   surface area decreases.
4. **`store.svelte.ts` is already a latent bundle leak** (imports
   pino, valibot, neverthrow, all 16 schemas). The migration forces
   the split that fixes this. Treat as a bug fix, not just a refactor.
   After migration: pino → `Effect.Logger` in `$lib/server/`, valibot
   → `effect/Schema`, neverthrow → `Effect<A, E, R>`. Schemas remain
   in `$lib/schemas/` but client-side usage is `import type` only
   (enforced by `verbatimModuleSyntax: true`), so the schemas
   themselves are no longer a leak risk.

## What NOT to migrate

- **`$lib/schemas/` path stays** — the content migrates from valibot
  to `effect/Schema` but the directory does not move. The 16 JSON
  Schema files in `schemas/*.json` are regenerated by
  `scripts/gen-schemas.ts` using `S.JSONSchema.make`.
- **Svelte 5 runes stay** for client-side UI state. Effect is for
  server-side and runtime concerns.

## Migration plan

### Phase 1 — prep (this commit)

- [x] Add `verbatimModuleSyntax: true` to `tsconfig.json`.
- [x] Update `docs/effect-assessment.md` (this document).

### Phase 2 — schemas

- [x] Migrate each schema from valibot to `effect/Schema`. Use `Schema.Struct`
      for content schemas (universal, tree-shakable). Reserve `S.Class` for
      runtime domain models under `$lib/server/runtime/`.
- [x] Update `scripts/gen-schemas.ts` to use `Schema.toJsonSchemaDocument`.
      Regenerate all `schemas/*.json`.
- [x] Verify: `pnpm test:unit` green. `pnpm gen:schemas` produces
      equivalent `schemas/*.json`.

### Phase 3 — data layer

- [x] Migrate `src/lib/server/data/util.ts` (parse, readYaml) to Effect.
- [x] Migrate `src/lib/server/data/load.ts` (loadQuestions, loadPacks,
      loadTags, loadOverlays, walkYaml) to Effect. Preserved the
      `{ files, issues }` accumulation in `walkYaml` and `LoadedDataset`
      using `Effect.match` for error accumulation without failing the
      `Effect.forEach`.
- [x] Migrate `src/lib/server/data/validate.ts` (cross-file checks) — kept as
      plain functions returning `LoadIssue[]`, wrapped in `Effect.Effect`
      where file `stat` is needed.
- [x] `src/lib/server/data/query.ts` and `src/lib/server/data/board.ts` are
      pure functions — updated import types only.
- [x] Update `src/lib/server/data/load.test.ts` and
      `src/lib/server/data/util.test.ts` to assert on Effect output using
      `Effect.match` for Success/Failure typing.
- [x] Code resides in `src/lib/server/data/` (already server-only by
      construction via Node fs imports).
- [x] Created `src/lib/data/store.svelte.ts` to use the `initDatasetServer`
      pattern, bridging the server Effect runtime to Svelte 5 runes.

### Phase 4 — scripts

- [x] Update `scripts/validate-data.ts` to use the Effect loader.
- [x] Update `scripts/gen-schemas.ts` to use `Schema.toJsonSchemaDocument`.

### Phase 5 — runtime (when step 10 starts)

- [ ] Add `$lib/server/runtime/` with `RoomRegistry` Layer,
      `Room` fiber, `PubSub<RoomEvent>`, `Ref<RoomState>`.
- [ ] Add SvelteKit remote functions: `gameState` (`query.live`),
      `submitAnswer` (`command`), `startGame` (`command`).
- [ ] Tests: `it.scoped` + in-memory `Layer.succeed(TestRoomRegistry)`.

### Phase 6 — observability

- [ ] Remove `src/lib/logger.ts` (pino + pino-pretty).
- [ ] Add `$lib/server/observability/logger.ts` with an Effect
      `Logger` Layer. Dev: `Logger.pretty`. Prod: `Logger.json` (or
      `Logger.logfmt` if ops prefer).
- [ ] Replace `childLogger(name)` callsites with
      `Effect.annotateLogs({ component: name })` — annotations
      travel with the fiber tree automatically.
- [ ] When OTEL is needed: add `@effect/opentelemetry`. Inject
      `trace_id` / `span_id` via the layer's `annotations` hook.

## Open questions deferred to the migration

- [ ] Should schemas live in `$lib/schemas/` (current) or move to
      `$lib/server/schemas/` since most usage is server-side? Lean:
      **keep in `$lib/schemas/`**, since the universal path allows
      `import type` from client code without the cost of an extra
      directory.
- [ ] Should `data/load.ts` move to `server/` entirely, or keep at
      `data/` with a `.server.ts` suffix? Lean: **move to
      `$lib/server/data/`** for consistency with the rest of the
      server-only code.
- [ ] How to keep the loader and runtime independently testable?
      Lean: **build a `DataLoader` service** as a `Layer` with
      dependencies on `FileSystem` (from `@effect/platform-node`).
      Tests use `Layer.succeed(FileSystem, InMemoryFileSystem)`.

## Key insight (revised)

Earlier draft said: "data layer is correct, add Effect only for
runtime." That was over-conservative. The user's pushback was right:
two tools for the same concern (errors) is a smell, and the same
applies to two tools for the same concern (schemas). The accumulator
pattern in the loader is not anti-Effect — it's `Effect.gen` with a
closure-captured `issues` array, returned as part of the success
value, never as a typed error.

The principle: **one tool per concern, pick the most general one
that fits all current and foreseeable uses.** Effect covers
validation (Schema), errors (Effect<A, E, R>), concurrency
(Fibers, Queue, PubSub), state (Ref), resources (Scope), streams
(Stream), and observability (Logger, Tracer, Metric). The current
valibot + neverthrow stack covers two of those. Migration collapses
the toolchain.

The bundle cost is the only real concern. `verbatimModuleSyntax:
true` plus the `$lib/server/` directory plus `import type` discipline
keeps Effect out of the client. Once that's enforced, the rest is
mechanical.

## References

- Effect docs: <https://effect.website/docs/>
- Effect vs neverthrow: <https://effect.website/docs/additional-resources/effect-vs-neverthrow/>
- Effect Schema: <https://effect.website/docs/schema/introduction/>
- Effect Schema JSON Schema export: <https://effect.website/docs/schema/json-schema/>
- Effect PubSub: <https://effect.website/docs/concurrency/pubsub/>
- Effect Logging: <https://effect.website/docs/observability/logging/>
- Effect Schedules: <https://effect.website/docs/scheduling/built-in-schedules/>
- SvelteKit + Effect template: <https://github.com/mateoroldos/sveltekit-effect-template>
- SvelteKit remote functions: <https://svelte.dev/docs/kit/remote-functions>
- TypeScript `verbatimModuleSyntax`: <https://www.typescriptlang.org/tsconfig/#verbatimModuleSyntax>
- neverthrow (replaced by Effect): <https://github.com/supermacro/neverthrow>
- valibot (replaced by Effect Schema): <https://valibot.dev/>
- pino (replaced by Effect Logger): <https://getpino.io/>
- Effect Logger: <https://effect.website/docs/observability/logging/>
- @effect/opentelemetry: <https://github.com/Effect-TS/effect/tree/main/packages/opentelemetry>
