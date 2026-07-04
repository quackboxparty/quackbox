//! Pool query engine — filter the question pool using a `PackFilter`.
//!
//! All filter operators are ANDed together. Mirrors `query.ts`.

use std::collections::{HashMap, HashSet};

use super::types::*;

/// Cache of pack id → resolved question ids, reused across board slots.
pub type PackCache = HashMap<String, Vec<String>>;

/// Resolve a pack to its full list of question IDs: included packs first
/// (recursively), then explicit `questions`, then `filter` matches.
/// Order-preserving dedup. Unknown packs resolve to empty.
pub fn resolve_pack(ds: &Dataset, cache: &mut PackCache, pack_id: &str) -> Vec<String> {
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

    let mut seen = HashSet::new();
    ids.retain(|id| seen.insert(id.clone()));

    cache.insert(pack_id.to_owned(), ids.clone());
    ids
}

/// Returns question IDs matching all filter criteria.
pub fn query_pool(ds: &Dataset, filter: &PackFilter) -> Vec<String> {
    let ids = ds
        .questions
        .values()
        .filter(|entry| matches_filter(&entry.item, filter))
        .map(|entry| entry.item.id().to_owned());

    match filter.limit {
        Some(limit) => ids.take(limit).collect(),
        None => ids.collect(),
    }
}

fn matches_filter(q: &Question, f: &PackFilter) -> bool {
    if let Some(ref kinds) = f.kinds {
        if !kinds.is_empty() && !kinds.contains(&q.kind()) {
            return false;
        }
    }

    let tags = q.tags();

    if let Some(ref tags_all) = f.tags_all {
        if !tags_all.is_empty() && !tags_all.iter().all(|t| tags.contains(t)) {
            return false;
        }
    }

    if let Some(ref tags_any) = f.tags_any {
        if !tags_any.is_empty() && !tags_any.iter().any(|t| tags.contains(t)) {
            return false;
        }
    }

    if let Some(ref tags_none) = f.tags_none {
        if tags_none.iter().any(|t| tags.contains(t)) {
            return false;
        }
    }

    if let Some(ref variants_any) = f.variants_any {
        if !variants_any.is_empty() {
            let defined = q.variant_names();
            if !variants_any.iter().any(|v| defined.contains(v)) {
                return false;
            }
        }
    }

    true
}
