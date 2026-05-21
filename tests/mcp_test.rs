use std::path::PathBuf;

use open_ultrareview_bridge::bridge::BridgeState;
use open_ultrareview_bridge::mcp_server::{
    ClearFindingsParams, McpServer, PostFindingsParams, PostFindingsResult, RestoreFindingsParams,
};
use open_ultrareview_bridge::store::FindingsStore;
use open_ultrareview_bridge::types::{FindingInput, SeverityInput};

fn input(title: &str) -> FindingInput {
    FindingInput {
        file: "src/lib.rs".to_string(),
        line: 7,
        col: 1,
        severity: SeverityInput::Warning,
        category: "maintainability".to_string(),
        title: title.to_string(),
        evidence: None,
        rationale: Some("rationale".to_string()),
        suggestion: Some("suggestion".to_string()),
        verified_by: None,
    }
}

#[test]
fn mcp_result_schema_uses_standard_integer_format() {
    let schema = schemars::schema_for!(PostFindingsResult);
    let json = serde_json::to_value(schema).expect("schema should serialize");

    assert_eq!(json["properties"]["added"]["type"], "integer");
    assert_ne!(json["properties"]["added"]["format"], "uint");
}

#[tokio::test]
async fn mcp_tools_mutate_store_and_emit_events() {
    let bridge = BridgeState::new(FindingsStore::new());
    let server = McpServer::new(bridge.clone());
    let project = PathBuf::from("/tmp/mcp-test");
    let mut rx = bridge.subscribe();

    let post = server
        .post_findings_direct(PostFindingsParams {
            source: "open-ultrareview".to_string(),
            project: project.to_string_lossy().to_string(),
            findings: vec![input("Finding A")],
        })
        .await;
    assert_eq!(post.added, 1);
    assert_eq!(post.affected_files, vec!["src/lib.rs"]);
    assert!(rx.try_recv().is_ok());

    let clear = server
        .clear_findings_direct(ClearFindingsParams {
            project: project.to_string_lossy().to_string(),
            source: "open-ultrareview".to_string(),
        })
        .await;
    assert_eq!(clear.cleared_files, vec!["src/lib.rs"]);
    assert!(rx.try_recv().is_ok());

    let store = bridge.store.read().await;
    assert!(store.get_file_findings(&project, "src/lib.rs").is_empty());
}

#[tokio::test]
async fn mcp_tools_map_relative_project_to_active_lsp_root() {
    let bridge = BridgeState::new(FindingsStore::new());
    let server = McpServer::new(bridge.clone());
    let project = PathBuf::from("/tmp/mcp-active-root");
    bridge.set_project_root(project.clone()).await;

    let post = server
        .post_findings_direct(PostFindingsParams {
            source: "open-ultrareview".to_string(),
            project: "sample-project".to_string(),
            findings: vec![input("Finding A")],
        })
        .await;

    assert_eq!(post.added, 1);
    let store = bridge.store.read().await;
    assert_eq!(store.get_file_findings(&project, "src/lib.rs").len(), 1);
    assert!(store
        .get_file_findings(PathBuf::from("sample-project").as_path(), "src/lib.rs")
        .is_empty());
}

#[tokio::test]
async fn restore_tool_undismisses_findings() {
    let bridge = BridgeState::new(FindingsStore::new());
    let server = McpServer::new(bridge.clone());
    let project = PathBuf::from("/tmp/mcp-test");

    server
        .post_findings_direct(PostFindingsParams {
            source: "open-ultrareview".to_string(),
            project: project.to_string_lossy().to_string(),
            findings: vec![input("Finding A")],
        })
        .await;

    let id = bridge
        .store
        .read()
        .await
        .get_file_findings(&project, "src/lib.rs")[0]
        .id
        .clone();
    bridge.store.write().await.dismiss_finding(&id);

    let result = server
        .restore_findings_direct(RestoreFindingsParams {
            project: project.to_string_lossy().to_string(),
        })
        .await;

    assert_eq!(result.restored_files, vec!["src/lib.rs"]);
    assert_eq!(
        bridge
            .store
            .read()
            .await
            .get_file_findings(&project, "src/lib.rs")
            .len(),
        1
    );
}
