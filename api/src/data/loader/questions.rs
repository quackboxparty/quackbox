use std::path::Path;

use super::*;

pub(super) fn load_questions(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<Question>, Vec<LoadIssue>), LoadError> {
    load_yaml_dir(
        &data_dir.join("questions"),
        rel,
        |raw: Vec<Question>| raw,
        Some(garde_issues),
        |q| q.id(),
        "question",
    )
}
