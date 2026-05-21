use std::path::PathBuf;

use open_ultrareview_bridge::bridge::BridgeState;
use open_ultrareview_bridge::store::FindingsStore;
use open_ultrareview_bridge::types::{FindingInput, SeverityInput, StoreEvent};

#[tokio::test]
async fn full_bridge_flow_sends_events_and_updates_store() {
    let bridge = BridgeState::new(FindingsStore::new());
    let project = PathBuf::from("/tmp/integration-test");
    let mut rx = bridge.subscribe();

    let affected = {
        let mut store = bridge.store.write().await;
        store.post_findings(
            project.clone(),
            "open-ultrareview".to_string(),
            vec![FindingInput {
                file: "src/main.rs".to_string(),
                line: 10,
                col: 1,
                severity: SeverityInput::Error,
                category: "logic-bugs".to_string(),
                title: "Bug A".to_string(),
                evidence: Some("evidence A".to_string()),
                rationale: None,
                suggestion: None,
                verified_by: Some("gpt-5.5".to_string()),
            }],
        )
    };
    bridge.notify_change(project.clone(), affected);

    let event = rx.try_recv().expect("store event should be sent");
    assert_eq!(
        event,
        StoreEvent::FindingsChanged {
            project: project.clone(),
            affected_files: vec!["src/main.rs".to_string()],
        }
    );

    let id = {
        let store = bridge.store.read().await;
        let findings = store.get_file_findings(&project, "src/main.rs");
        assert_eq!(findings.len(), 1);
        findings[0].id.clone()
    };

    {
        let mut store = bridge.store.write().await;
        assert_eq!(
            store.dismiss_finding(&id),
            Some((project.clone(), "src/main.rs".to_string()))
        );
    }

    let store = bridge.store.read().await;
    assert!(store.get_file_findings(&project, "src/main.rs").is_empty());
}
