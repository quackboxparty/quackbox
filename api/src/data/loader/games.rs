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

pub(super) fn load_game_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<GameOverlay>, Vec<LoadIssue>), LoadError> {
    load_yaml_dir(
        &locale_dir.join("games"),
        rel,
        |raw: GameOverlay| vec![raw],
        None,
        |g| g.id.as_str(),
        "game overlay",
    )
}
