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
        let (g_ovl, g_iss) = super::games::load_game_overlays(&locale_entry, rel)?;
        locale_overlays.games.extend(g_ovl);
        issues.extend(g_iss);

        let (q_ovl, q_iss) = load_question_overlays(&locale_entry, rel)?;
        let (p_ovl, p_iss) = load_pack_overlays(&locale_entry, rel)?;
        let (t_ovl, t_iss) = load_tag_overlays(&locale_entry, rel)?;

        locale_overlays.questions.extend(q_ovl);
        locale_overlays.packs.extend(p_ovl);
        locale_overlays.tags.extend(t_ovl);
        issues.extend([q_iss, p_iss, t_iss].concat());
    }

    Ok((overlays, issues))
}

fn load_question_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<QuestionOverlay>, Vec<LoadIssue>), LoadError> {
    let dir = locale_dir.join("questions");
    if !dir.exists() {
        return Ok((HashMap::new(), Vec::new()));
    }
    let files = walk_yaml(&dir)?;

    let results: Vec<_> = files
        .iter()
        .map(|path| parse_file(path, &rel(path), |raw: Vec<QuestionOverlay>| raw, None))
        .collect();

    Ok(collect_registry(results, |q| &q.id, "question overlay"))
}

fn load_pack_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<PackOverlay>, Vec<LoadIssue>), LoadError> {
    let dir = locale_dir.join("packs");
    if !dir.exists() {
        return Ok((HashMap::new(), Vec::new()));
    }
    let files = walk_yaml(&dir)?;

    let results: Vec<_> = files
        .iter()
        .map(|path| parse_file(path, &rel(path), |raw: PackOverlay| vec![raw], None))
        .collect();

    Ok(collect_registry(results, |p| &p.id, "pack overlay"))
}

fn load_tag_overlays(
    locale_dir: &Path,
    rel: &dyn Fn(&Path) -> String,
) -> Result<(Registry<TagOverlay>, Vec<LoadIssue>), LoadError> {
    let dir = locale_dir.join("tags");
    if !dir.exists() {
        return Ok((HashMap::new(), Vec::new()));
    }

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
