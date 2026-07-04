use super::*;

pub(super) fn check_game_refs(ds: &Dataset, issues: &mut Vec<LoadIssue>) {
    for entry in ds.games.values() {
        let gc = &entry.item;
        for (game_idx, game_entry) in gc.games.iter().enumerate() {
            match game_entry {
                GameEntry::GridQuiz(g) => {
                    for (cat_idx, cat) in g.board.categories.iter().enumerate() {
                        let ctx = format!("game '{}' entry[{game_idx}] category[{cat_idx}]", gc.id);

                        if let Some(pack_id) = &cat.pack_ref
                            && !ds.packs.contains_key(pack_id)
                        {
                            issues.push(LoadIssue::msg(
                                &entry.file,
                                format!("{ctx} references unknown pack '{pack_id}'"),
                            ));
                        }

                        for qid in cat.question_ids.iter().flat_map(|map| map.values()) {
                            if !ds.questions.contains_key(qid) {
                                issues.push(LoadIssue::msg(
                                    &entry.file,
                                    format!("{ctx} references unknown question '{qid}'"),
                                ));
                            }
                        }

                        if let Some(filter) = &cat.filter {
                            super::check_filter_tags(ds, filter, &entry.file, &ctx, issues);
                        }
                    }

                    if let Some(diff_map) = &g.board.difficulty_map {
                        for (point, tags) in diff_map {
                            for tag in tags {
                                if !ds.tags.contains_key(tag) {
                                    issues.push(LoadIssue::msg(
                                        &entry.file,
                                        format!(
                                            "unknown tag '{tag}' on game '{}' entry[{game_idx}] difficulty_map[{point}]",
                                            gc.id
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
                GameEntry::Linear(g) => match &g.questions {
                    LinearSource::Questions { question_ids } => {
                        for qid in question_ids {
                            if !ds.questions.contains_key(qid) {
                                issues.push(LoadIssue::msg(
                                    &entry.file,
                                    format!(
                                        "game '{}' entry[{game_idx}] references unknown question '{qid}'",
                                        gc.id
                                    ),
                                ));
                            }
                        }
                    }
                    LinearSource::Pack { pack_id } => {
                        if !ds.packs.contains_key(pack_id) {
                            issues.push(LoadIssue::msg(
                                &entry.file,
                                format!(
                                    "game '{}' entry[{game_idx}] references unknown pack '{pack_id}'",
                                    gc.id
                                ),
                            ));
                        }
                    }
                    LinearSource::Filter { filter } => {
                        let ctx = format!("game '{}' entry[{game_idx}] linear filter", gc.id);
                        super::check_filter_tags(ds, filter, &entry.file, &ctx, issues);
                    }
                },
            }
        }
    }
}
