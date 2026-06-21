//! Board builder — resolve a board definition into a 2D grid of question IDs.
//!
//! Mirrors `board.ts`: explicit IDs win, then pack refs, then filters.
//! Deterministic shuffle via mulberry32 PRNG.

use std::collections::{HashMap, HashSet};

use super::query::query_pool;
use super::types::*;

/// Resolved board: `grid[category_idx][point_idx] = Some(question_id) | None`.
pub type BoardGrid = Vec<Vec<Option<String>>>;

pub struct BuildBoardOpts {
    pub locale: String,
    pub seed: u32,
}

impl Default for BuildBoardOpts {
    fn default() -> Self {
        Self {
            locale: "en".into(),
            seed: 42,
        }
    }
}

/// Build a resolved NxM board grid. Unresolvable slots are `None`.
pub fn build_board(
    ds: &LoadedDataset,
    board: &BoardFile,
    opts: &BuildBoardOpts,
) -> (BoardGrid, Vec<LoadIssue>) {
    let issues = Vec::new();
    let mut rng = Mulberry32::new(opts.seed);
    let mut used = HashSet::new();
    let mut pack_cache: HashMap<String, Vec<String>> = HashMap::new();

    let diff_map: HashMap<&str, &[String]> = board
        .difficulty_map
        .as_ref()
        .map(|dm| {
            dm.iter()
                .map(|(k, v)| (k.as_str(), v.as_slice()))
                .collect()
        })
        .unwrap_or_default();

    let mut grid = Vec::new();

    for cat in &board.categories {
        let mut row = Vec::new();
        for point in &board.points {
            let point_key = point.to_string();

            // 1. Explicit question_ids override
            if let Some(qid) = cat.question_ids.as_ref().and_then(|m| m.get(&point_key)) {
                used.insert(qid.clone());
                row.push(Some(qid.clone()));
                continue;
            }

            // 2. Build candidates from pack_ref + filter + difficulty_map
            let candidates = build_candidates(cat, &point_key, ds, &mut pack_cache, &diff_map);
            let unused: Vec<&String> = candidates.iter().filter(|id| !used.contains(*id)).collect();
            let pool: Vec<&String> = if unused.is_empty() {
                candidates.iter().collect()
            } else {
                unused
            };

            if let Some(picked) = pick_random(&pool, &mut rng) {
                used.insert(picked.clone());
                row.push(Some(picked));
            } else {
                row.push(None);
            }
        }
        grid.push(row);
    }

    (grid, issues)
}

fn build_candidates(
    cat: &BoardCategory,
    point_key: &str,
    ds: &LoadedDataset,
    pack_cache: &mut HashMap<String, Vec<String>>,
    diff_map: &HashMap<&str, &[String]>,
) -> Vec<String> {
    let mut candidates = Vec::new();

    if let Some(ref pack_id) = cat.pack_ref {
        candidates = resolve_pack(ds, pack_cache, pack_id);
    }

    if let Some(ref filter) = cat.filter {
        let filtered: HashSet<String> = query_pool(ds, filter).into_iter().collect();
        if candidates.is_empty() {
            candidates = filtered.into_iter().collect();
        } else {
            candidates.retain(|id| filtered.contains(id));
        }
    }

    if let Some(diff_tags) = diff_map.get(point_key) {
        if !diff_tags.is_empty() {
            candidates.retain(|qid| {
                ds.questions
                    .get(qid)
                    .map(|e| diff_tags.iter().any(|tag| e.item.tags().contains(&tag.as_str().to_owned())))
                    .unwrap_or(false)
            });
        }
    }

    candidates
}

fn resolve_pack(
    ds: &LoadedDataset,
    cache: &mut HashMap<String, Vec<String>>,
    pack_id: &str,
) -> Vec<String> {
    if let Some(cached) = cache.get(pack_id) {
        return cached.clone();
    }

    let Some(entry) = ds.packs.get(pack_id) else {
        return Vec::new();
    };
    let pack = &entry.item;

    let mut ids = Vec::new();

    for incl in pack.includes.iter().flatten() {
        ids.extend(resolve_pack(ds, cache, incl));
    }
    ids.extend(pack.questions.iter().flatten().cloned());
    if let Some(ref filter) = pack.filter {
        ids.extend(query_pool(ds, filter));
    }

    // Deduplicate preserving order
    let mut seen = HashSet::new();
    ids.retain(|id| seen.insert(id.clone()));

    cache.insert(pack_id.to_owned(), ids.clone());
    ids
}

fn pick_random(pool: &[&String], rng: &mut Mulberry32) -> Option<String> {
    if pool.is_empty() {
        return None;
    }
    let mut shuffled: Vec<&String> = pool.to_vec();
    for i in (1..shuffled.len()).rev() {
        let j = (rng.next() * (i + 1) as f64) as usize;
        shuffled.swap(i, j);
    }
    shuffled.first().map(|s| (*s).clone())
}

// ─── Mulberry32 PRNG ────────────────────────────────────────────────────────

struct Mulberry32 {
    state: u32,
}

impl Mulberry32 {
    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6D2B_79F5);
        let mut t = self.state;
        t = (t ^ (t >> 15)).wrapping_mul(1 | t);
        t = (t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t))) ^ t;
        ((t ^ (t >> 14)) as f64) / 4_294_967_296.0
    }
}
