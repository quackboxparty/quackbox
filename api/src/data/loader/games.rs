use std::path::Path;

use super::*;

pub(super) fn load_games(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<GameConfig>, Vec<LoadIssue>), LoadError> {
    load_yaml_dir(
        &data_dir.join("games"),
        rel,
        |raw: GameConfig| vec![raw],
        Some(garde_issues),
        |g| g.id.as_str(),
        "game config",
    )
}
