use std::path::PathBuf;

use crate::data::load_dataset;

#[test]
fn loads_real_dataset_without_issues() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../data");
    if !path.exists() {
        return;
    }
    let mut ds = load_dataset(&path).expect("load_dataset failed");
    let cross = crate::data::validate::run_cross_file_checks(&ds);
    ds.issues.extend(cross);
    assert!(ds.issues.is_empty(), "real dataset issues: {:?}", ds.issues);
    assert!(ds.questions.len() >= 20);
    assert!(ds.packs.len() >= 1);
    assert!(ds.tags.len() >= 10);
}
