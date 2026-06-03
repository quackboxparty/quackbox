# Quackbox — Data Model

> Status: **design sketch**, not yet implemented. Scope: how quiz content
> (questions, packs, gamemodes, media, translations) is stored on disk,
> validated, and loaded at runtime.

## Goals

1. **Content lives in the repo as files** — YAML, human-editable, PR-reviewable.
2. **Questions are reusable across gamemodes** — no copy-pasting answers.
3. **Multiple gamemodes** (classic, battle royale, survival, music quiz, …) each
   declare which question types they accept; loader filters automatically.
4. **Multi-media questions** — text, image, audio, video on prompts _and_
   choices.
5. **i18n** — questions can be translated, language-locked, or locale-relevant
   (cultural). Same workflow as ClassQuiz/GNOME (Weblate-compatible).
6. **Validated** — schemas in valibot, JSON Schema exported for editor support
   (YAML LSP) and CI checks.
7. **Community-contributable** — clear file layout, autocomplete in editors, PRs
   only touch the file they care about.

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
      tags/
        registry.yaml
    fr/
      …
  packs/              # curated playlists (lists of question IDs)
    official/
      britpop-trivia.yaml
      capitals-easy.yaml
    community/
      …
  tags/               # tag registry — slugs, categories, labels
    registry.yaml
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

| Layer         | Purpose              | Owns                                     |
| ------------- | -------------------- | ---------------------------------------- |
| **Questions** | Raw facts            | content + correct answer + tags          |
| **Packs**     | Curated playlists    | list of question IDs (or a filter query) |
| **Gamemodes** | Rules / presentation | scoring, timing, accepted question types |

A question never knows which gamemode it'll be played in. A gamemode never
hard-codes question content. Packs glue them at runtime.

## Question schema

### Canonical (English example)

```yaml
# data/questions/geography/capitals.yaml
# yaml-language-server: $schema=../../../schemas/question.schema.json

- id: q_capital_france_paris # stable, globally unique (see ID strategy)
  type: multiple_choice # see "Question types"
  tags: [geography, capitals, europe, general, region_global]
  #     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ subject       ^^^^^^^^^^^^^^^^ difficulty + region
  source: https://example.org/... # optional, for attribution
  license: CC-BY-4.0 # optional, per-question license override
  # lang_locked: en               # optional — question only valid in this lang
  answer: [a] # references choice IDs; array supports multi-select
  content:
    default_lang: en # language of the canonical strings below
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
`default_lang` lives _inside_ `content` because it describes the canonical
strings; the answer key, tags, and license do not depend on it.

**Answer placement by type.** For `multiple_choice` / `true_false` / `order` /
`match`, `answer` references stable choice IDs and lives **outside** `content`
— same key works in every language. For `numeric` / `range`, `answer` is a
number + tolerance, also outside `content`. For `open`, the accepted strings
are per-language and live **inside** `content.answer.accepted[]`; overlays
supply translations like any other text field. See [Question types](#question-types-initial-set)
for the per-type sub-schemas.

```yaml
# open-answer example
- id: q_tallest_tower_paris
  type: open
  answer: { match: fuzzy, normalize: [lowercase, strip_diacritics] }
  content:
    default_lang: en
    prompt: { text: "Tallest tower in Paris?" }
    answer:
      accepted: ["Eiffel Tower", "Eiffel"]
```

**No `difficulty` field.** Difficulty is subjective and context-dependent. It's
expressed via tags from the `difficulty` category (`easy`, `general`,
`niche`, `expert`, `trick`, …) — see [Tags](#tags). Gamemodes that need a
hard ordering (e.g. Quiz Duell tile values, Survival ramp-up) get it from the
pack's curated placement, not from the question itself.

### Translation overlay

```yaml
# data/i18n/de/questions/geography/capitals.yaml
# yaml-language-server: $schema=../../../../../schemas/question-overlay.schema.json

- id: q_capital_france_paris
  content:
    prompt:
      text: "Was ist die Hauptstadt von Frankreich?"
    choices:
      - { id: a, text: Paris }
      - { id: b, text: London }
      - { id: c, text: Berlin }
      - { id: d, text: Madrid }
    explanation: "Paris ist seit 987 die Hauptstadt."
```

Overlay uses the **same shape** as canonical (choices stay as objects keyed by
`id`), so choice-level fields like `media.alt` can be localized without a
second schema. Choice IDs match canonical → answer key never duplicated,
never drifts. Missing keys fall back to canonical per the resolution rules
below.

## Tags

Tags do two jobs:

1. **Stable identifier** for filtering (`tags: [chemistry]` on a question).
2. **Translatable display label** shown in UI ("Chemie" for a German player).

Mixing them is pain. So: tags in question files are stable slugs; their human
labels live in a separate registry, translated like any other content.

### Registry

```yaml
# data/tags/registry.yaml
# yaml-language-server: $schema=../../schemas/tag-registry.schema.json

- slug: chemistry
  category: subject
  default_lang: en
  label: Chemistry
  description: Chemical elements, reactions, compounds

- slug: general
  category: difficulty
  default_lang: en
  label: General knowledge
  description: Most casual players are expected to know this

- slug: niche
  category: difficulty
  default_lang: en
  label: Niche
  description: Specialist or fan knowledge

- slug: trick
  category: difficulty
  default_lang: en
  label: Trick question
  description: Sounds harder/easier than it is; wordplay or misdirection

- slug: region_dach
  category: region
  default_lang: en
  label: DACH (DE/AT/CH)
  description: Culturally most relevant in German-speaking countries

- slug: region_global
  category: region
  default_lang: en
  label: Global
```

### Overlay

```yaml
# data/i18n/de/tags/registry.yaml
- slug: chemistry
  label: Chemie
  description: Chemische Elemente, Reaktionen, Verbindungen

- slug: general
  label: Allgemeinwissen

- slug: niche
  label: Nischenwissen

- slug: trick
  label: Fangfrage

- slug: region_dach
  label: DACH (DE/AT/CH)
```

Same Weblate workflow as questions. Translators see `label` + `description`,
never `slug` or `category`.

### Categories (axes)

| Category     | Examples                                      | Purpose                                          |
| ------------ | --------------------------------------------- | ------------------------------------------------ |
| `subject`    | `chemistry`, `geography`, `pop_music`         | What it's about. Filter / theme packs.           |
| `difficulty` | `easy`, `general`, `niche`, `expert`, `trick` | Qualitative hint, replaces 1–5 scale.            |
| `audience`   | `requires_stem`, `kids_friendly`, `adults`    | Who's expected to know it.                       |
| `region`     | `region_dach`, `region_uk`, `region_us`, `region_global` | Cultural relevance (soft hint, not hard locale lock). |
| `format`     | `wordplay`, `visual`, `audio`                 | Mechanical hint for gamemode compatibility.      |

Category lives on the registry entry, not on the question — keeps question
files terse. Region tags replace the earlier `locales:` field on questions:
one mechanism, same expressiveness. `lang_locked` stays — it's a hard loader
rule, not a soft theme.

### CI rules

- All tags used in questions/packs must exist in the registry → typo prevention.
- Slugs are lowercase, underscores only. Region slugs prefixed `region_`.
- Slugs are part of the public API (same rule as question IDs): no rename
  without a deprecation marker.
- Adding a tag = one PR touching `registry.yaml` + relevant overlay files.

### Question types (initial set)

| Type              | Description                                           | Example tags         |
| ----------------- | ----------------------------------------------------- | -------------------- |
| `multiple_choice` | One or more correct answers from a fixed list         | most common          |
| `true_false`      | Binary                                                |                      |
| `open`            | Free-text answer, fuzzy match against accepted values | trivia               |
| `numeric`         | Number, exact or range/tolerance                      | dates, stats         |
| `order`           | Arrange choices in correct order                      | historical events    |
| `match`           | Pair items between two columns                        | capitals ↔ countries |
| `range`           | Pick value on a slider                                | year, percentage     |

Each type has its own sub-schema (e.g. `multiple_choice` requires `choices[]`,
`numeric` requires `answer.value` + optional `answer.tolerance`).

## Media

### Schema

Media is an array on `prompt` (and optionally on individual `choices`, e.g.
image-answer questions).

```yaml
content:
  prompt:
    text: "Which band released this song?"
    media:
      - kind: audio # image | audio | video
        ref: media/audio/wonderwall-clip.ogg
        duration_ms: 8000 # type-specific metadata
        alt: "8-second clip of a guitar riff" # accessibility
  choices:
    - id: a
      text: Oasis
      media: # choices can also have media
        - { kind: image, ref: media/img/oasis-logo.svg, alt: Oasis logo }
```

### Storage strategy

Recommended: **hybrid**.

| Asset type                        | Storage                                           | Reason                  |
| --------------------------------- | ------------------------------------------------- | ----------------------- |
| Small images (< 100 KB, SVG/WebP) | In-repo at `data/media/img/`                      | Fast, no bandwidth cost |
| Audio / video clips               | **External refs** or release-tarball assets       | Git is bad at binaries  |
| User-uploaded media (future)      | Object storage (S3/MinIO) configured by self-host | Not part of OSS dataset |

External ref formats to support:

- `youtube:VIDEO_ID?start=10&end=18`
- `https://…` (direct URL)
- `media/audio/foo.ogg` (relative path, resolved against `data/media/`)

The loader handles each via pluggable resolvers. Self-hosters can pin or mirror
media as they like.

### Per-locale media

Sometimes media itself needs translation (narrated audio, image with text).
Overlay can replace the media array:

```yaml
# data/i18n/de/questions/music/podcasts.yaml
- id: q_podcast_intro_clip
  content:
    prompt:
      media:
        - { kind: audio, ref: media/audio/de/q-podcast-intro-clip.ogg }
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
  - q_oasis_wonderwall_year
  - q_blur_parklife_album
  - q_pulp_common_people_singer

# Option B: dynamic filter (mutually exclusive with `questions`)
# filter:
#   tags_all: [music, britpop]     # must have all of these
#   tags_any: [general, niche]  # at least one of these
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

The runtime (`gamemodes/battle_royale/index.ts`) exports rules, scoring, state
machine, and uses remote functions (`query.live`, `command`, `form`) to push
state to clients.

### Filtering questions per gamemode

```
pool_for(gamemode, pack, locale) =
  pack.questions
    .filter(q => gamemode.accepts.types.includes(q.type))
    .filter(q => q.choices.length <= gamemode.accepts.max_choices)
    .filter(q => !q.lang_locked || q.lang_locked === locale)
    .map(q => resolve_translation(q, locale))
```

Gamemodes that care about difficulty consume it via tags (e.g. Survival could
request `tags_any: [easy, general]` for the first round, `niche` later).
Quiz Duell ignores it — tile value comes from the pack's board layout.

Pack authors don't need to know which gamemodes exist. Adding a new gamemode
"just works" against existing question pool, modulo compatibility filtering.

## i18n

### Classification

Questions fall into three categories — schema must express all three:

| Category            | Field / mechanism       | Example                                                           |
| ------------------- | ----------------------- | ----------------------------------------------------------------- |
| **Universal**       | (no marker)             | "Capital of France?" — translate freely                           |
| **Language-locked** | `lang_locked: en`       | "Rhymes with 'cat'?" — pinned to one lang, loader hides elsewhere |
| **Locale-relevant** | `region_*` tag          | "Bundesliga 2020 winner?" — `region_dach`, soft hint not a filter |

### Loader behavior

**Fallback chain** (resolved in order, first hit wins per field):

1. Overlay for `userLocale` (`data/i18n/<userLocale>/…`).
2. Overlay for the repo-wide fallback locale (`en` — see policy below).
3. Canonical strings (which are in the question's own `content.default_lang`).

```ts
const REPO_FALLBACK = "en"; // repo-wide policy, see below

function resolve(questionId: string, userLocale: string): Question | null {
  const base = loadCanonical(questionId);
  if (base.lang_locked && base.lang_locked !== userLocale) return null;

  const overlays = [
    loadOverlay(questionId, userLocale),
    userLocale !== REPO_FALLBACK ? loadOverlay(questionId, REPO_FALLBACK) : null,
  ].filter(Boolean);

  const merged = overlays.reduceRight((acc, o) => merge(acc, o), base);
  merged.served_lang = pickServedLang(base, overlays, userLocale);
  merged.is_fallback = merged.served_lang !== userLocale;
  return merged;
}

function buildPool(gamemode, pack, locale) {
  return pack.questions
    .map((id) => resolve(id, locale))
    .filter((q) => q !== null)
    .filter((q) => gamemode.accepts(q));
}
```

**Repo policy:** canonical content _should_ be authored in English whenever
possible (so the fallback always works), but `content.default_lang` is
per-question because some content is genuinely born in another language (a
German-only quiz pack contributed by a German author shouldn't be blocked
waiting for a translation). The UI surfaces `is_fallback` so players know
they're seeing a non-localized question.

### Tooling

The repo layout is designed to plug **Weblate** or **Crowdin** directly:

- Translation files live under `data/i18n/<lang>/` mirroring canonical paths.
- Translators only see translatable fields (the schema enforces this).
- New languages = new directory; no canonical files modified.
- Partial translations are fine; loader silently falls back.

## IDs

Globally unique, stable, never reused.

Format: **type prefix + descriptive slug**. The slug is a hint for humans
reading diffs/logs; it is **not authoritative** and carries no semantic
meaning the loader can rely on.

```
q_<descriptive_slug>                 e.g. q_capital_france_paris
pack_<slug>                          e.g. pack_britpop
gm_<slug>                            e.g. gm_battle_royale
```

Rules:

- Lowercase, underscores only.
- Slug describes the question's _content_, not its category. Moving a
  question from `geography/` to `history/` does **not** require renaming
  it — the ID is a historical label, not a path.
- Concurrent-PR collisions are resolved by uniqueness of the descriptive
  slug itself; CI enforces global uniqueness, the later PR renames.
- IDs are part of the public API of the repo — no removals or renames without
  a deprecation marker (CI check).

Alternative: ULIDs (`q_01HXYZ…`). More collision-proof but unreadable in
PRs and logs. **Going with descriptive slugs.**

## File grouping

**One file per topic-subtopic**, target 20–50 questions, soft cap ~100.

Why:

- Translation tools (Weblate) work per-file. Few large files > thousands of
  tiny.
- PR review: 1 file changed when adding 5 related questions, not 5 files.
- Filesystem: 10k+ tiny files break IDE indexing.
- Sibling questions in the same file help catch duplicates and tone drift.

Question IDs are flat-namespaced, so files can be reshuffled without breaking
pack references.

## Validation

### Schemas live in `src/lib/schemas/`

Written in **valibot**. Single source of truth → both:

1. **TS types** via `v.InferOutput`
2. **JSON Schema** via `@valibot/to-json-schema`, written to
   `schemas/*.schema.json`, committed for editor support (YAML LSP)

### Validation points

| Where                              | What                                                           |
| ---------------------------------- | -------------------------------------------------------------- |
| Editor (developer/contributor)     | YAML LSP reads `schemas/*.json` → autocomplete + inline errors |
| CI (`pnpm validate-data`)          | Loads every YAML, runs valibot, fails build on errors          |
| Runtime (server start / pack load) | Same valibot schemas, fail loudly with question ID context     |

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
import { command, query } from "$app/server";
import * as v from "valibot";

export const gameState = query.live(
  v.object({ roomCode: v.string() }),
  async function* ({ roomCode }) {
    const room = rooms.get(roomCode);
    for await (const state of room.subscribe()) yield state;
  },
);

export const submitAnswer = command(
  v.object({ roomCode: v.string(), choiceId: v.string() }),
  async ({ roomCode, choiceId }) => {/* … */},
);
```

Data layer (loader, pool builder, schemas) has zero coupling to SvelteKit.
Gamemodes consume the data layer + SvelteKit remote functions.

## Open questions

1. **Default language policy** — English-canonical (force all questions to have
   English) or per-question `default_lang`? Going with per-question + soft
   "prefer English" repo policy.
2. **Partial translation behavior** — silent fallback (friendly) vs hide
   incomplete questions (purer). Going with silent fallback, surface a
   completeness % in the pack metadata UI.
3. **Pack license vs question license** — questions can carry their own license,
   pack carries pack-level license. CI should warn on incompatible combinations.
4. **Community packs** — same repo (PR-reviewed) or separate submission
   pipeline? Start with same repo under `data/packs/community/`.
5. **Versioning** — should questions carry a `revision` field for "this answer
   was updated"? Probably yes once we hit the first real correction.
6. **RTL languages** (Arabic, Hebrew) — schema is fine, UI must support from day
   one to avoid retrofit pain.
7. **Editor UI** (web-based quiz builder) — out of scope for first pass, but the
   schema must be friendly to it. Valibot schemas double as form validation via
   Superforms-equivalent.

## Implementation order (suggested)

1. Define valibot schemas in `src/lib/schemas/` (question, pack, gamemode,
   overlay variants).
2. JSON Schema export script: `pnpm gen:schemas`.
3. Data loader in `src/lib/data/` (parse YAML → validate → merge overlays →
   build indexes).
4. CI workflow: `pnpm check && pnpm lint && pnpm test && pnpm validate-data`.
5. Example dataset: 1 canonical question file (~10 Qs), 1 German overlay, 1
   pack, 1 lang-locked question — proves the pipeline end to end.
6. First gamemode (`classic`) using remote functions + live query for room
   state. Wires data layer to gameplay.
7. Second gamemode (`battle_royale` or `music_quiz`) to validate the
   gamemode-agnostic claim.
