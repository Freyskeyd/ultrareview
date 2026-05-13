use std::path::PathBuf;

use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString};
use ultrareview_bridge::lsp_server::finding_to_diagnostic;
use ultrareview_bridge::types::{Finding, Severity};

#[test]
fn finding_maps_to_lsp_diagnostic() {
    let finding = Finding {
        id: "ultrareview-security-0".to_string(),
        source: "ultrareview".to_string(),
        project: PathBuf::from("/tmp/project"),
        file: "src/main.rs".to_string(),
        line: 12,
        col: 3,
        severity: Severity::Error,
        category: "security".to_string(),
        title: "Unsanitized input".to_string(),
        evidence: Some("The request body reaches SQL directly".to_string()),
        rationale: None,
        suggestion: Some("Use parameterized SQL".to_string()),
        verified_by: Some("gpt-5.5".to_string()),
        dismissed: false,
    };

    let diagnostic = finding_to_diagnostic(&finding);

    assert_eq!(diagnostic.range.start.line, 11);
    assert_eq!(diagnostic.range.start.character, 2);
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(diagnostic.source.as_deref(), Some("ultrareview"));
    assert_eq!(
        diagnostic.code,
        Some(NumberOrString::String("security".to_string()))
    );
    assert!(diagnostic.message.contains("Unsanitized input"));
    assert!(diagnostic
        .message
        .contains("Evidence: The request body reaches SQL directly"));
    assert!(diagnostic
        .message
        .contains("Suggestion: Use parameterized SQL"));
    assert!(diagnostic.message.contains("[verified by gpt-5.5]"));
}

#[test]
fn warning_and_info_severities_map_to_lsp() {
    let base = Finding {
        id: "id".to_string(),
        source: "clippy".to_string(),
        project: PathBuf::from("/tmp/project"),
        file: "src/lib.rs".to_string(),
        line: 1,
        col: 1,
        severity: Severity::Warning,
        category: "style".to_string(),
        title: "Style issue".to_string(),
        evidence: None,
        rationale: None,
        suggestion: None,
        verified_by: None,
        dismissed: false,
    };

    assert_eq!(
        finding_to_diagnostic(&base).severity,
        Some(DiagnosticSeverity::WARNING)
    );

    let info = Finding {
        severity: Severity::Info,
        ..base
    };
    assert_eq!(
        finding_to_diagnostic(&info).severity,
        Some(DiagnosticSeverity::INFORMATION)
    );
}
