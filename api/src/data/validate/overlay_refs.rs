use super::*;

pub(super) fn check_overlay_refs(ds: &Dataset, issues: &mut Vec<LoadIssue>) {
    for (locale, locale_overlays) in &ds.overlays {
        for (qid, entry) in &locale_overlays.questions {
            if !ds.questions.contains_key(qid) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!(
                        "overlay of locale '{locale}' references unknown question '{qid}'"
                    ),
                    path: None,
                });
            }
        }
        for (pid, entry) in &locale_overlays.packs {
            if !ds.packs.contains_key(pid) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!(
                        "overlay of locale '{locale}' references unknown pack '{pid}'"
                    ),
                    path: None,
                });
            }
        }
        for (tid, entry) in &locale_overlays.tags {
            if !ds.tags.contains_key(tid) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!("overlay of locale '{locale}' references unknown tag '{tid}'"),
                    path: None,
                });
            }
        }
        for (gid, entry) in &locale_overlays.games {
            let Some(base_game) = ds.games.get(gid) else {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!(
                        "overlay of locale '{locale}' references unknown game '{gid}'"
                    ),
                    path: None,
                });
                continue;
            };

            if entry.item.games.len() > base_game.item.games.len() {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!(
                        "overlay of locale '{locale}' game '{gid}' has {} game entries but base has {}",
                        entry.item.games.len(),
                        base_game.item.games.len()
                    ),
                    path: None,
                });
            }

            for (idx, ovl_game) in entry.item.games.iter().enumerate() {
                let Some(base_entry) = base_game.item.games.get(idx) else {
                    continue;
                };

                if let Some(board_ovl) = &ovl_game.board {
                    match base_entry {
                        GameEntry::GridQuiz(base_grid) => {
                            if board_ovl.categories.len() > base_grid.board.categories.len() {
                                issues.push(LoadIssue {
                                    file: entry.file.clone(),
                                    message: format!(
                                        "overlay of locale '{locale}' game '{gid}' entry[{idx}] has {} categories but base has {}",
                                        board_ovl.categories.len(),
                                        base_grid.board.categories.len()
                                    ),
                                    path: None,
                                });
                            }
                        }
                        GameEntry::Linear(_) => {
                            issues.push(LoadIssue {
                                file: entry.file.clone(),
                                message: format!(
                                    "overlay of locale '{locale}' game '{gid}' entry[{idx}] cannot define board for non-grid mode"
                                ),
                                path: None,
                            });
                        }
                    }
                }
            }
        }
    }
}
