# Score is derived from an append-only judgment log, not accumulated

A Room's score is **folded from an append-only judgment log** (`submission →
verdict`, with supersedes), recomputed on each change — never stored as a
running counter. Revising a ruling means appending a superseding verdict and
refolding, not mutating a total.

Chosen over the obvious running-total counter because a Moderator must be able
to revise an earlier decision (a mistaken ruling, or overruling a timer
expiry). With a counter, undoing a several-questions-old ruling is fragile;
with a derived fold it is a single append. It also composes cleanly with the
full-snapshot broadcast — a refold simply produces a new snapshot, so there is
no special "score correction" message. This mirrors the data model's
"correctness lives on data, derive don't duplicate" principle. The cost: score
is never read straight from a field; every read folds the log (cheap at quiz
scale). Detail in `docs/architecture.md` (Adjudication).
