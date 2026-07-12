use std::collections::HashMap;
use std::path::Path;

use super::*;

pub(super) fn load_overlays(
    data_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Overlays, Vec<LoadIssue>), LoadError> {
    let i18n_dir = data_dir.join("i18n");
    if !i18n_dir.exists() {
        return Ok((HashMap::new(), Vec::new()));
    }

    let mut overlays: Overlays = HashMap::new();
    let mut issues = Vec::new();

    for locale_entry in read_dir_sorted(&i18n_dir)? {
        if !locale_entry.is_dir() {
            continue;
        }
        let locale = filename(&locale_entry);
        let locale_overlays = overlays.entry(locale).or_default();

        let (q_ovl, q_iss) = load_question_overlays(&locale_entry, rel)?;
        let (p_ovl, p_iss) = load_pack_overlays(&locale_entry, rel)?;
        let (t_ovl, t_iss) = load_tag_overlays(&locale_entry, rel)?;
        let (g_ovl, g_iss) = load_game_overlays(&locale_entry, rel)?;

        locale_overlays.questions.extend(q_ovl);
        locale_overlays.packs.extend(p_ovl);
        locale_overlays.tags.extend(t_ovl);
        locale_overlays.games.extend(g_ovl);
        issues.extend([q_iss, p_iss, t_iss, g_iss].concat());
    }

    Ok((overlays, issues))
}

fn read_dir_sorted(dir: &Path) -> Result<Vec<PathBuf>, LoadError> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    Ok(entries)
}

fn filename(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn load_question_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<QuestionOverlay>, Vec<LoadIssue>), LoadError> {
    load_yaml_dir(
        &locale_dir.join("questions"),
        rel,
        |raw: Vec<QuestionOverlay>| raw,
        None,
        |q| q.id.as_str(),
        "question overlay",
    )
}

fn load_pack_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<PackOverlay>, Vec<LoadIssue>), LoadError> {
    load_yaml_dir(
        &locale_dir.join("packs"),
        rel,
        |raw: PackOverlay| vec![raw],
        None,
        |p| p.id.as_str(),
        "pack overlay",
    )
}

fn load_tag_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<TagOverlay>, Vec<LoadIssue>), LoadError> {
    let dir = locale_dir.join("tags");

    let results: Vec<_> = TAG_CATEGORIES
        .iter()
        .filter_map(|cat| {
            let path = dir.join(format!("{cat}.yaml"));
            path.exists()
                .then(|| parse_file(&path, &rel(&path), |raw: Vec<TagOverlay>| raw, None))
        })
        .collect();

    Ok(collect_registry(results, |t| &t.id, "tag overlay"))
}

fn load_game_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<GameConfigOverlay>, Vec<LoadIssue>), LoadError> {
    load_yaml_dir(
        &locale_dir.join("games"),
        rel,
        |raw: GameConfigOverlay| vec![raw],
        None,
        |g| g.id.as_str(),
        "game overlay",
    )
}
