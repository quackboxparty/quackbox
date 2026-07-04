use std::path::Path;

use super::*;

pub(super) fn load_tags(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<Tag>, Vec<LoadIssue>), LoadError> {
    let dir = data_dir.join("tags");

    let results: Vec<_> = TAG_CATEGORIES
        .iter()
        .filter_map(|cat| {
            let path = dir.join(format!("{cat}.yaml"));
            path.exists()
                .then(|| parse_file(&path, &rel(&path), |raw: Vec<Tag>| raw, None))
        })
        .collect();

    Ok(collect_registry(results, |t| &t.id, "tag"))
}
