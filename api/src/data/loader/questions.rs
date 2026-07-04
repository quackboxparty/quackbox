use std::path::Path;

use super::*;

pub(super) fn load_questions(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<Question>, Vec<LoadIssue>), LoadError> {
    let files = walk_yaml(&data_dir.join("questions"))?;

    let results: Vec<_> = files
        .iter()
        .map(|path| {
            parse_file(
                path,
                &rel(path),
                |raw: Vec<Question>| raw,
                Some(garde_issues),
            )
        })
        .collect();

    Ok(collect_registry(results, |q| q.id(), "question"))
}
