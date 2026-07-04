use std::path::Path;

use super::*;

pub(super) fn load_packs(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<Pack>, Vec<LoadIssue>), LoadError> {
    let files = walk_yaml(&data_dir.join("packs"))?;

    let results: Vec<_> = files
        .iter()
        .map(|path| parse_file(path, &rel(path), |raw: Pack| vec![raw], Some(garde_issues)))
        .collect();

    Ok(collect_registry(results, |p| &p.id, "pack"))
}
