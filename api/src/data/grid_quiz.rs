//! Board builder — resolve a board definition into a 2D grid of question IDs.
//!
//! Mirrors `board.ts`: explicit IDs win, then pack refs, then filters.
//! Deterministic shuffle via mulberry32 PRNG.

use std::collections::{HashMap, HashSet};

use super::query::{query_pool, resolve_pack, PackCache};
use super::types::*;

/// Resolved board: `grid[category_idx][point_idx] = Some(question_id) | None`.
pub type BoardGrid = Vec<Vec<Option<String>>>;

/// Build a resolved NxM board grid. Unresolvable slots are `None`.
pub fn build_board(ds: &Dataset, board: &Board, seed: u32) -> BoardGrid {
    let mut rng = Mulberry32::new(seed);
    let mut used = HashSet::new();
    let mut pack_cache: PackCache = PackCache::new();

    let diff_map: HashMap<&u32, &[String]> = board
        .difficulty_map
        .as_ref()
        .map(|dm| dm.iter().map(|(k, v)| (k, v.as_slice())).collect())
        .unwrap_or_default();

    let mut grid = Vec::new();

    for cat in &board.categories {
        let mut row = Vec::new();
        for point in &board.points {
            // 1. Explicit question_ids override
            if let Some(qid) = cat.question_ids.as_ref().and_then(|m| m.get(point)) {
                used.insert(qid.clone());
                row.push(Some(qid.clone()));
                continue;
            }

            // 2. Build candidates from pack_ref + filter + difficulty_map
            let candidates = build_candidates(cat, point, ds, &mut pack_cache, &diff_map);
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

    grid
}

fn build_candidates(
    cat: &BoardCategory,
    point: &u32,
    ds: &Dataset,
    pack_cache: &mut PackCache,
    diff_map: &HashMap<&u32, &[String]>,
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

    if let Some(diff_tags) = diff_map.get(point) {
        if !diff_tags.is_empty() {
            candidates.retain(|qid| {
                ds.questions
                    .get(qid)
                    .map(|e| {
                        e.item
                            .tags()
                            .iter()
                            .any(|qtag| diff_tags.iter().any(|dtag| dtag == qtag))
                    })
                    .unwrap_or(false)
            });
        }
    }

    candidates
}

fn pick_random(pool: &[&String], rng: &mut Mulberry32) -> Option<String> {
    if pool.is_empty() {
        return None;
    }
    let idx = (rng.next() * pool.len() as f64) as usize;
    Some((*pool[idx]).clone())
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
