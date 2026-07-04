//! Cross-file validation that can't be expressed inside a single YAML schema.
//!
//! Mirrors `validate.ts`: tag refs, pack refs, overlay refs, pack cycles,
//! and media file checks. Returns issues without failing — we accumulate.

use super::types::*;

mod game_refs;
mod media;
mod overlay_refs;
mod pack_cycles;
mod refs;
mod tag_refs;

#[cfg(test)]
mod tests;

/// Run all cross-file checks, returning accumulated issues.
pub fn run_cross_file_checks(ds: &Dataset) -> Vec<LoadIssue> {
    let mut issues = Vec::new();
    tag_refs::check_tag_refs(ds, &mut issues);
    refs::check_refs(ds, &mut issues);
    game_refs::check_game_refs(ds, &mut issues);
    overlay_refs::check_overlay_refs(ds, &mut issues);
    pack_cycles::check_pack_cycles(ds, &mut issues);
    media::check_media_files(ds, &mut issues);
    issues
}

fn check_filter_tags(
    ds: &Dataset,
    filter: &PackFilter,
    file: &str,
    context: &str,
    issues: &mut Vec<LoadIssue>,
) {
    let all_tags = filter
        .tags_all
        .iter()
        .flatten()
        .chain(filter.tags_any.iter().flatten())
        .chain(filter.tags_none.iter().flatten());

    for tag in all_tags {
        if !ds.tags.contains_key(tag) {
            issues.push(LoadIssue::msg(
                file,
                format!("unknown tag '{tag}' on {context}"),
            ));
        }
    }
}
