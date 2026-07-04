use std::path::Path;

use super::*;

pub(super) fn load_packs(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<Pack>, Vec<LoadIssue>), LoadError> {
    load_yaml_dir(
        &data_dir.join("packs"),
        rel,
        |raw: Pack| vec![raw],
        Some(garde_issues),
        |p| p.id.as_str(),
        "pack",
    )
}
