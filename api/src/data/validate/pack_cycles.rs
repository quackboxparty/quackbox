use std::collections::{HashMap, HashSet};

use super::*;

pub(super) fn check_pack_cycles(ds: &Dataset, issues: &mut Vec<LoadIssue>) {
    let graph: HashMap<&str, &[String]> = ds
        .packs
        .values()
        .map(|e| {
            let includes: &[String] = match &e.item.includes {
                Some(v) => v.as_slice(),
                None => &[],
            };
            (e.item.id.as_str(), includes)
        })
        .collect();

    let file_by_id: HashMap<&str, &str> = ds
        .packs
        .values()
        .map(|e| (e.item.id.as_str(), e.file.as_str()))
        .collect();

    let mut visited = HashSet::new();
    let mut in_progress = HashSet::new();

    for id in graph.keys() {
        dfs_cycle(
            id,
            &graph,
            &file_by_id,
            &mut visited,
            &mut in_progress,
            &mut Vec::new(),
            issues,
        );
    }
}

fn dfs_cycle<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, &'a [String]>,
    file_by_id: &HashMap<&'a str, &'a str>,
    visited: &mut HashSet<&'a str>,
    in_progress: &mut HashSet<&'a str>,
    stack: &mut Vec<&'a str>,
    issues: &mut Vec<LoadIssue>,
) {
    if visited.contains(node) {
        return;
    }
    if in_progress.contains(node) {
        let cycle_start = stack.iter().position(|&n| n == node).unwrap_or(0);
        let mut cycle: Vec<&str> = stack[cycle_start..].to_vec();
        cycle.push(node);
        let cycle_str = cycle.join(" -> ");
        issues.push(LoadIssue::msg(
            file_by_id.get(node).copied().unwrap_or("(unknown)"),
            format!("pack includes cycle: {cycle_str}"),
        ));
        return;
    }

    in_progress.insert(node);
    stack.push(node);

    if let Some(deps) = graph.get(node) {
        for next in *deps {
            if graph.contains_key(next.as_str()) {
                dfs_cycle(next, graph, file_by_id, visited, in_progress, stack, issues);
            }
        }
    }

    stack.pop();
    in_progress.remove(node);
    visited.insert(node);
}
