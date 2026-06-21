//! Pool query engine — filter the question pool using a `PackFilter`.
//!
//! All filter operators are ANDed together. Mirrors `query.ts`.

use super::types::*;

/// Returns question IDs matching all filter criteria.
pub fn query_pool(ds: &LoadedDataset, filter: &PackFilter) -> Vec<String> {
    let matched: Vec<&Entry<Question>> = ds
        .questions
        .values()
        .filter(|entry| matches_filter(&entry.item, filter))
        .collect();

    let ids: Vec<String> = matched.iter().map(|e| e.item.id().to_owned()).collect();

    match filter.limit {
        Some(limit) => ids.into_iter().take(limit).collect(),
        None => ids,
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
