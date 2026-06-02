# Quackbox — Data Model

> Status: **design sketch**, not yet implemented.
> Scope: how quiz content (questions, packs, gamemodes, media, translations)
> is stored on disk, validated, and loaded at runtime.

## Goals

1. **Content lives in the repo as files** — YAML, human-editable, PR-reviewable.
2. **Questions are reusable across gamemodes** — no copy-pasting answers.
3. **Multiple gamemodes** (classic, battle royale, survival, music quiz, …) each
   declare which question types they accept; loader filters automatically.
4. **Multi-media questions** — text, image, audio, video on prompts _and_ choices.
5. **i18n** — questions can be translated, language-locked, or locale-relevant
   (cultural). Same workflow as ClassQuiz/GNOME (Weblate-compatible).
6. **Validated** — schemas in valibot, JSON Schema exported for editor support
   (YAML LSP) and CI checks.
7. **Community-contributable** — clear file layout, autocomplete in editors,
   PRs only touch the file they care about.

## High-level layout

```
data/
  questions/          # canonical content, language-neutral metadata + default lang
    geography/
      capitals.yaml
      flags.yaml
    music/
      90s-pop.yaml
    history/
    science/
    …
  i18n/               # translation overlays, mirror of questions/ + packs/
    de/
      questions/
        geography/capitals.yaml
        music/90s-pop.yaml
      packs/
        official/britpop-trivia.yaml
    fr/
      …
  packs/              # curated playlists (lists of question IDs)
    official/
      britpop-trivia.yaml
      capitals-easy.yaml
    community/
      …
  media/              # binary assets referenced by questions
    img/
      flag-jp.svg
    audio/            # see "Media" section for storage strategy

gamemodes/            # code, not data; each declares compatibility metadata
  classic/
  battle_royale/
  survival/
  music_quiz/
  …

schemas/              # generated from valibot, committed for editor support
  question.schema.json
  pack.schema.json
  gamemode.schema.json
```

### Layering principle

Three independent concerns, never mixed:

| Layer | Purpose | Owns |
|---|---|---|
| **Questions** | Raw facts | content + correct answer + tags |
| **Packs** | Curated playlists | list of question IDs (or a filter query) |
| **Gamemodes** | Rules / presentation | scoring, timing, accepted question types |

A question never knows which gamemode it'll be played in. A gamemode never
hard-codes question content. Packs glue them at runtime.

## Question schema

### Canonical (English example)

```yaml
# data/questions/geography/capitals.yaml
# yaml-language-server: $schema=../../../schemas/question.schema.json

- id: q_geo_cap_001               # stable, globally unique (see ID strategy)
  type: multiple_choice           # see "Question types"
  tags: [geography, capitals, europe]
  difficulty: 1                   # 1–5
  default_lang: en
  # lang_locked: en               # optional — question only valid in this lang
  # locales: [de, at, ch]         # optional — soft cultural relevance hint
  answer: [a]                     # references choice IDs; array supports multi-select
  source: https://example.org/...  # optional, for attribution
  license: CC-BY-4.0              # optional, per-question license override
  content:
    prompt:
      text: "What is the capital of France?"
      # media: …                  # optional, see "Media" section
    choices:
      - { id: a, text: Paris }
      - { id: b, text: London }
      - { id: c, text: Berlin }
      - { id: d, text: Madrid }
    explanation: "Paris has been the capital since 987 AD."
```

**Hard rule:** everything inside `content` is translatable. Everything outside
is metadata that does not change per language. This makes Weblate config trivial
and makes it impossible for a translator to accidentally edit an answer key.

### Translation overlay

```yaml
# data/i18n/de/questions/geography/capitals.yaml
# yaml-language-server: $schema=../../../../../schemas/question-overlay.schema.json

- id: q_geo_cap_001
  content:
    prompt:
      text: "Was ist die Hauptstadt von Frankreich?"
    choices:
      a: Paris
      b: London
      c: Berlin
      d: Madrid
    explanation: "Paris ist seit 987 die Hauptstadt."
```

Overlay only carries translatable fields. Choice IDs match canonical →
answer key never duplicated, never drifts. Missing keys fall back to canonical.

### Question types (initial set)

| Type | Description | Example tags |
|---|---|---|
| `multiple_choice` | One or more correct answers from a fixed list | most common |
| `true_false` | Binary | |
| `open` | Free-text answer, fuzzy match against accepted values | trivia |
| `numeric` | Number, exact or range/tolerance | dates, stats |
| `order` | Arrange choices in correct order | historical events |
| `match` | Pair items between two columns | capitals ↔ countries |
| `range` | Pick value on a slider | year, percentage |

Each type has its own sub-schema (e.g. `multiple_choice` requires `choices[]`,
`numeric` requires `answer.value` + optional `answer.tolerance`).

## Media

### Schema

Media is an array on `prompt` (and optionally on individual `choices`,
e.g. image-answer questions).

```yaml
content:
  prompt:
    text: "Which band released this song?"
    media:
      - kind: audio                # image | audio | video
        ref: media/audio/wonderwall-clip.ogg
        duration_ms: 8000          # type-specific metadata
        alt: "8-second clip of a guitar riff"   # accessibility
  choices:
    - id: a
      text: Oasis
      media:                       # choices can also have media
        - { kind: image, ref: media/img/oasis-logo.svg, alt: Oasis logo }
```

### Storage strategy

Recommended: **hybrid**.

| Asset type | Storage | Reason |
|---|---|---|
| Small images (< 100 KB, SVG/WebP) | In-repo at `data/media/img/` | Fast, no bandwidth cost |
| Audio / video clips | **External refs** or release-tarball assets | Git is bad at binaries |
| User-uploaded media (future) | Object storage (S3/MinIO) configured by self-host | Not part of OSS dataset |

External ref formats to support:
- `youtube:VIDEO_ID?start=10&end=18`
- `https://…` (direct URL)
- `media/audio/foo.ogg` (relative path, resolved against `data/media/`)

The loader handles each via pluggable resolvers. Self-hosters can pin or
mirror media as they like.

### Per-locale media

Sometimes media itself needs translation (narrated audio, image with text).
Overlay can replace the media array:

```yaml
# data/i18n/de/questions/music/podcasts.yaml
- id: q_pod_042
  content:
    prompt:
      media:
        - { kind: audio, ref: media/audio/de/q042-clip.ogg }
```

If no localized media is provided, loader falls back to canonical.

## Packs

A pack is either an **explicit list** of question IDs or a **filter query**
against the pool.

```yaml
# data/packs/official/britpop-trivia.yaml
# yaml-language-server: $schema=../../../schemas/pack.schema.json

id: pack_britpop
title: Britpop Trivia
description: 90s UK music quiz
author: lucas
license: CC-BY-4.0
recommended_gamemodes: [music_quiz, classic, survival]
default_lang: en

# Option A: explicit list (curated)
questions:
  - q_music_90s_001
  - q_music_90s_002
  - q_music_90s_017

# Option B: dynamic filter (mutually exclusive with `questions`)
# filter:
#   tags: [music, 90s, britpop]
#   types: [multiple_choice]
#   limit: 20
#   shuffle: true
```

Pack translations are minimal — title/description only, questions are shared:

```yaml
# data/i18n/de/packs/official/britpop-trivia.yaml
id: pack_britpop
title: Britpop-Quiz
description: 90er-Jahre UK-Musikquiz
```

## Gamemodes

Gamemodes are **code**, not data, but each ships a small declarative manifest
describing what content it accepts and how it presents the game.

```yaml
# gamemodes/battle_royale/manifest.yaml
id: gm_battle_royale
name: Battle Royale
description: Last player standing. Wrong answer = elimination.
accepts:
  types: [multiple_choice, true_false]
  max_choices: 4
  min_choices: 2
requires:
  timer: true
  min_players: 2
ui:
  player_view: PlayerView.svelte
  host_view: HostView.svelte
  spectator_view: SpectatorView.svelte
```

The runtime (`gamemodes/battle_royale/index.ts`) exports rules, scoring,
state machine, and uses remote functions (`query.live`, `command`, `form`) to
push state to clients.

### Filtering questions per gamemode

```
pool_for(gamemode, pack, locale) =
  pack.questions
    .filter(q => gamemode.accepts.types.includes(q.type))
    .filter(q => q.choices.length <= gamemode.accepts.max_choices)
    .filter(q => !q.lang_locked || q.lang_locked === locale)
    .map(q => resolve_translation(q, locale))
```

Pack authors don't need to know which gamemodes exist. Adding a new gamemode
"just works" against existing question pool, modulo compatibility filtering.

## i18n

### Classification

Questions fall into three categories — schema must express all three:

| Category | Field | Example |
|---|---|---|
| **Universal** | (no flag) | "Capital of France?" — translate freely |
| **Language-locked** | `lang_locked: en` | "Rhymes with 'cat'?" — pinned to one lang |
| **Locale-relevant** | `locales: [de, at, ch]` | "Bundesliga 2020 winner?" — universal but mainly relevant to DACH |

### Loader behavior

```ts
function resolve(questionId: string, userLocale: string): Question | null {
  const base = loadCanonical(questionId);
  if (base.lang_locked && base.lang_locked !== userLocale) return null;
  const overlay = loadOverlay(questionId, userLocale);
  return merge(base, overlay ?? {}); // deep merge, overlay wins on conflicts
}

function buildPool(gamemode, pack, locale) {
  return pack.questions
    .map((id) => resolve(id, locale))
    .filter((q) => q !== null)
    .filter((q) => gamemode.accepts(q));
}
```

### Tooling

The repo layout is designed to plug **Weblate** or **Crowdin** directly:
- Translation files live under `data/i18n/<lang>/` mirroring canonical paths.
- Translators only see translatable fields (the schema enforces this).
- New languages = new directory; no canonical files modified.
- Partial translations are fine; loader silently falls back.

## IDs

Globally unique, stable, never reused.

Recommended format: **slug-style**, prefixed by type and category.

```
q_<category>_<subcategory>_<seq>     e.g. q_geo_cap_001
pack_<slug>                          e.g. pack_britpop
gm_<slug>                            e.g. gm_battle_royale
```

Rules:
- Lowercase, underscores only.
- Sequential numbers per (category, subcategory).
- IDs are part of the public API of the repo — refactoring categories must
  keep IDs intact (CI check: no removed IDs without a deprecation marker).

Alternative: ULIDs (`q_01HXYZ…`). More collision-proof but harder to recognize
in PRs and logs. **Going with slugs unless a real collision happens.**

## File grouping

**One file per topic-subtopic**, target 20–50 questions, soft cap ~100.

Why:
- Translation tools (Weblate) work per-file. Few large files > thousands of tiny.
- PR review: 1 file changed when adding 5 related questions, not 5 files.
- Filesystem: 10k+ tiny files break IDE indexing.
- Sibling questions in the same file help catch duplicates and tone drift.

Question IDs are flat-namespaced, so files can be reshuffled without
breaking pack references.

## Validation

### Schemas live in `src/lib/schemas/`

Written in **valibot**. Single source of truth → both:
1. **TS types** via `v.InferOutput`
2. **JSON Schema** via `@valibot/to-json-schema`, written to
   `schemas/*.schema.json`, committed for editor support (YAML LSP)

### Validation points

| Where | What |
|---|---|
| Editor (developer/contributor) | YAML LSP reads `schemas/*.json` → autocomplete + inline errors |
| CI (`pnpm validate-data`) | Loads every YAML, runs valibot, fails build on errors |
| Runtime (server start / pack load) | Same valibot schemas, fail loudly with question ID context |

### What CI must catch

- Schema violations
- Duplicate IDs (across all questions/packs)
- Dangling pack references (pack lists nonexistent question ID)
- Missing media files referenced by `ref: media/…`
- Answer references nonexistent choice ID
- Translation overlays referencing nonexistent question
- Markdown/text in inappropriate fields (later: sanitize)

## Runtime integration (SvelteKit remote functions)

Questions and packs are static-ish — load at server startup, validate, hold in
memory. A self-hosted instance can reload via signal/endpoint when new content
is dropped in.

Game sessions use **`query.live`** to stream state to players:

```ts
// example, not final
import { query, command } from '$app/server';
import * as v from 'valibot';

export const gameState = query.live(
  v.object({ roomCode: v.string() }),
  async function* ({ roomCode }) {
    const room = rooms.get(roomCode);
    for await (const state of room.subscribe()) yield state;
  }
);

export const submitAnswer = command(
  v.object({ roomCode: v.string(), choiceId: v.string() }),
  async ({ roomCode, choiceId }) => { /* … */ }
);
```

Data layer (loader, pool builder, schemas) has zero coupling to SvelteKit.
Gamemodes consume the data layer + SvelteKit remote functions.

## Open questions

1. **Default language policy** — English-canonical (force all questions to
   have English) or per-question `default_lang`? Going with per-question +
   soft "prefer English" repo policy.
2. **Partial translation behavior** — silent fallback (friendly) vs hide
   incomplete questions (purer). Going with silent fallback, surface a
   completeness % in the pack metadata UI.
3. **Pack license vs question license** — questions can carry their own
   license, pack carries pack-level license. CI should warn on incompatible
   combinations.
4. **Community packs** — same repo (PR-reviewed) or separate submission
   pipeline? Start with same repo under `data/packs/community/`.
5. **Versioning** — should questions carry a `revision` field for "this
   answer was updated"? Probably yes once we hit the first real correction.
6. **RTL languages** (Arabic, Hebrew) — schema is fine, UI must support
   from day one to avoid retrofit pain.
7. **Editor UI** (web-based quiz builder) — out of scope for first pass,
   but the schema must be friendly to it. Valibot schemas double as form
   validation via Superforms-equivalent.

## Implementation order (suggested)

1. Define valibot schemas in `src/lib/schemas/` (question, pack, gamemode,
   overlay variants).
2. JSON Schema export script: `pnpm gen:schemas`.
3. Data loader in `src/lib/data/` (parse YAML → validate → merge overlays →
   build indexes).
4. CI workflow: `pnpm check && pnpm lint && pnpm test && pnpm validate-data`.
5. Example dataset: 1 canonical question file (~10 Qs), 1 German overlay,
   1 pack, 1 lang-locked question — proves the pipeline end to end.
6. First gamemode (`classic`) using remote functions + live query for room
   state. Wires data layer to gameplay.
7. Second gamemode (`battle_royale` or `music_quiz`) to validate the
   gamemode-agnostic claim.
