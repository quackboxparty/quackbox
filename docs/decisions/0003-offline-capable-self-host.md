# Offline-capable self-hosting: core play needs no internet, extras degrade

A self-hosted Quackbox instance must run the **core game loop** (questions,
buzz, floor, submission, verdict, score) with **no internet connection** after
setup — a moderator sets it up at home (binary + content + local media) and runs
games on a LAN or single box, air-gapped if they like. Features that
intrinsically need internet are **available when online and gracefully absent
offline**, never a hard dependency for core play.

Internet-only features that must degrade, not block:
- **`youtube:` / `url:` media** — only `local:` media is guaranteed offline;
  remote refs simply don't render offline (host may mirror them).
- **Telemetry** (`opentelemetry-otlp`) — strictly opt-in; a no-op when no
  collector is configured, never blocking startup or play on an export.
- **Future LLM-as-judge** — must be optional and/or local-model only; the
  `Auto` and `Moderator` judges cover offline adjudication fully.

Chosen over a fully air-gapped hard requirement (which would force every present
and future feature to ship a local fallback) and over a cloud-assumed design.
The home/LAN quiz use case is the product's reason to exist, so the core loop is
offline-hard; richer features stay opt-in. The cost: contributors must keep the
core play path free of outbound calls, and any internet-using feature must have
a defined offline-absent behavior. Already aligned: the static frontend is
served by axum (no CDN/SSR), fonts are vendored into the build, and content is
local YAML.
