use std::collections::HashMap;
use std::path::Path;

use super::*;

pub(super) fn load_games(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<GameConfig>, Vec<LoadIssue>), LoadError> {
    let files = walk_yaml(&data_dir.join("games"))?;
    let results: Vec<_> = files
        .iter()
        .map(|path| {
            parse_file(path, &rel(path), |raw: GameConfig| vec![raw], Some(garde_issues))
        })
        .collect();
    Ok(collect_registry(
        results,
        |g: &GameConfig| &g.id,
        "game config",
    ))
}

pub(super) fn load_game_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<GameOverlay>, Vec<LoadIssue>), LoadError> {
    let dir = locale_dir.join("games");
    if !dir.exists() {
        return Ok((HashMap::new(), Vec::new()));
    }
    let files = walk_yaml(&dir)?;
    let results: Vec<_> = files
        .iter()
        .map(|path| parse_file(path, &rel(path), |raw: GameOverlay| vec![raw], None))
        .collect();
    Ok(collect_registry(results, |g| &g.id, "game overlay"))
}
