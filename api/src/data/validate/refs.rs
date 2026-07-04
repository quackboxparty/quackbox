use super::*;

pub(super) fn check_refs(ds: &Dataset, issues: &mut Vec<LoadIssue>) {
    for entry in ds.packs.values() {
        let pack = &entry.item;
        for qid in pack.questions.iter().flatten() {
            if !ds.questions.contains_key(qid) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!("pack '{}' references unknown question '{qid}'", pack.id),
                    path: None,
                });
            }
        }
        for pid in pack.includes.iter().flatten() {
            if !ds.packs.contains_key(pid) {
                issues.push(LoadIssue {
                    file: entry.file.clone(),
                    message: format!("pack '{}' includes unknown pack '{pid}'", pack.id),
                    path: None,
                });
            }
        }
    }

    for entry in ds.questions.values() {
        if let Some(ref dep) = entry.item.base().deprecated
            && let Some(ref replaced_by) = dep.replaced_by
            && !ds.questions.contains_key(replaced_by)
        {
            issues.push(LoadIssue {
                file: entry.file.clone(),
                message: format!(
                    "question '{}' replaced_by unknown question '{replaced_by}'",
                    entry.item.id()
                ),
                path: None,
            });
        }
    }
}
