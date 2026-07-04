use super::*;

pub(super) fn check_tag_refs(ds: &Dataset, issues: &mut Vec<LoadIssue>) {
    for entry in ds.questions.values() {
        for tag in entry.item.tags() {
            if !ds.tags.contains_key(tag) {
                issues.push(LoadIssue::msg(
                    &entry.file,
                    format!("unknown tag '{tag}' on question '{}'", entry.item.id()),
                ));
            }
        }
    }

    for entry in ds.packs.values() {
        if let Some(ref filter) = entry.item.filter {
            let ctx = format!("pack '{}'", entry.item.id);
            super::check_filter_tags(ds, filter, &entry.file, &ctx, issues);
        }
    }
}
