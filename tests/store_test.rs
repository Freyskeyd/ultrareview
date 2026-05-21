use std::path::PathBuf;

use open_ultrareview_bridge::store::FindingsStore;
use open_ultrareview_bridge::types::{FindingInput, SeverityInput};

fn sample_input() -> FindingInput {
    FindingInput {
        file: "src/main.rs".to_string(),
        line: 42,
        col: 5,
        severity: SeverityInput::Error,
        category: "logic-bugs".to_string(),
        title: "Off-by-one in loop".to_string(),
        evidence: Some("Loop iterates one too many times".to_string()),
        rationale: None,
        suggestion: Some("Use `<` instead of `<=`".to_string()),
        verified_by: Some("gpt-5.5".to_string()),
    }
}

#[test]
fn post_and_query_findings() {
    let mut store = FindingsStore::new();
    let project = PathBuf::from("/tmp/test-project");

    let affected = store.post_findings(
        project.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );

    assert_eq!(affected, vec!["src/main.rs"]);
    let findings = store.get_file_findings(&project, "src/main.rs");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].title, "Off-by-one in loop");
    assert!(!findings[0].dismissed);
}

#[test]
fn incremental_posting_appends_findings() {
    let mut store = FindingsStore::new();
    let project = PathBuf::from("/tmp/test-project");

    store.post_findings(
        project.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );

    let mut input2 = sample_input();
    input2.line = 99;
    input2.title = "Second finding".to_string();
    store.post_findings(
        project.clone(),
        "open-ultrareview".to_string(),
        vec![input2],
    );

    let findings = store.get_file_findings(&project, "src/main.rs");
    assert_eq!(findings.len(), 2);
}

#[test]
fn clearing_one_source_keeps_other_sources() {
    let mut store = FindingsStore::new();
    let project = PathBuf::from("/tmp/test-project");

    store.post_findings(
        project.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );

    let mut clippy_input = sample_input();
    clippy_input.title = "Clippy warning".to_string();
    clippy_input.category = "clippy".to_string();
    store.post_findings(project.clone(), "clippy".to_string(), vec![clippy_input]);

    let cleared = store.clear_findings(&project, "open-ultrareview");
    assert_eq!(cleared, vec!["src/main.rs"]);

    let findings = store.get_file_findings(&project, "src/main.rs");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].source, "clippy");
}

#[test]
fn dismissed_finding_stays_dismissed_after_clear_and_repost() {
    let mut store = FindingsStore::new();
    let project = PathBuf::from("/tmp/test-project");

    store.post_findings(
        project.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );
    let id = store.get_file_findings(&project, "src/main.rs")[0]
        .id
        .clone();

    assert_eq!(
        store.dismiss_finding(&id),
        Some((project.clone(), "src/main.rs".to_string()))
    );
    store.clear_findings(&project, "open-ultrareview");
    store.post_findings(
        project.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );

    assert!(store.get_file_findings(&project, "src/main.rs").is_empty());
}

#[test]
fn restore_findings_undismisses_project_findings() {
    let mut store = FindingsStore::new();
    let project = PathBuf::from("/tmp/test-project");

    store.post_findings(
        project.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );
    let id = store.get_file_findings(&project, "src/main.rs")[0]
        .id
        .clone();
    store.dismiss_finding(&id);

    assert_eq!(store.restore_findings(&project), vec!["src/main.rs"]);
    assert_eq!(store.get_file_findings(&project, "src/main.rs").len(), 1);
}

#[test]
fn dismissed_findings_are_scoped_to_project_and_source() {
    let mut store = FindingsStore::new();
    let project_a = PathBuf::from("/tmp/project-a");
    let project_b = PathBuf::from("/tmp/project-b");

    store.post_findings(
        project_a.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );
    let id = store.get_file_findings(&project_a, "src/main.rs")[0]
        .id
        .clone();
    store.dismiss_finding(&id);
    store.clear_findings(&project_a, "open-ultrareview");

    store.post_findings(
        project_b.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );
    store.post_findings(
        project_a.clone(),
        "clippy".to_string(),
        vec![sample_input()],
    );

    assert_eq!(store.get_file_findings(&project_b, "src/main.rs").len(), 1);
    assert_eq!(store.get_file_findings(&project_a, "src/main.rs").len(), 1);
}

#[test]
fn restore_findings_only_clears_dismissals_for_that_project() {
    let mut store = FindingsStore::new();
    let project_a = PathBuf::from("/tmp/project-a");
    let project_b = PathBuf::from("/tmp/project-b");

    store.post_findings(
        project_a.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );
    store.post_findings(
        project_b.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );

    let id_a = store.get_file_findings(&project_a, "src/main.rs")[0]
        .id
        .clone();
    let id_b = store.get_file_findings(&project_b, "src/main.rs")[0]
        .id
        .clone();
    store.dismiss_finding(&id_a);
    store.dismiss_finding(&id_b);

    store.restore_findings(&project_a);

    assert_eq!(store.get_file_findings(&project_a, "src/main.rs").len(), 1);
    assert!(store
        .get_file_findings(&project_b, "src/main.rs")
        .is_empty());
}

#[test]
fn persistent_store_round_trips_findings() {
    let temp = tempfile::tempdir().expect("tempdir");
    let cache_path = temp.path().join("findings.json");
    let project = PathBuf::from("/tmp/persisted-project");

    let mut store = FindingsStore::with_cache_path(cache_path.clone());
    store.post_findings(
        project.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );

    let loaded = FindingsStore::load_from_path(cache_path);
    let findings = loaded.get_file_findings(&project, "src/main.rs");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].title, "Off-by-one in loop");
}

#[test]
fn relative_project_entries_can_be_migrated_to_absolute_root() {
    let mut store = FindingsStore::new();
    let relative_project = PathBuf::from("sample-project");
    let absolute_project = PathBuf::from("/tmp/sample-project-worktree");

    store.post_findings(
        relative_project.clone(),
        "open-ultrareview".to_string(),
        vec![sample_input()],
    );

    let affected = store.migrate_relative_projects_to_root(&absolute_project);

    assert_eq!(affected, vec!["src/main.rs"]);
    assert!(store
        .get_file_findings(&relative_project, "src/main.rs")
        .is_empty());
    assert_eq!(
        store
            .get_file_findings(&absolute_project, "src/main.rs")
            .len(),
        1
    );
}
