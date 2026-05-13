use crate::bridge::BridgeState;
use crate::types::{Finding, Severity, StoreEvent};
use std::path::{Path, PathBuf};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tracing::info;

#[derive(Clone)]
pub struct LspBackend {
    client: Client,
    bridge: BridgeState,
    project_root: std::sync::Arc<tokio::sync::RwLock<Option<PathBuf>>>,
}

impl LspBackend {
    pub fn new(client: Client, bridge: BridgeState) -> Self {
        Self {
            client,
            bridge,
            project_root: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    fn spawn_event_listener(&self) {
        let backend = self.clone();
        let mut rx = backend.bridge.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    StoreEvent::FindingsChanged {
                        project,
                        affected_files,
                    } => {
                        for file in affected_files {
                            backend.publish_file_diagnostics(&project, &file).await;
                        }
                    }
                }
            }
        });
    }

    async fn set_project_root(&self, params: &InitializeParams) {
        let root = params
            .workspace_folders
            .as_ref()
            .and_then(|folders| folders.first())
            .and_then(|folder| folder.uri.to_file_path().ok())
            .or_else(|| {
                params
                    .root_uri
                    .as_ref()
                    .and_then(|uri| uri.to_file_path().ok())
            });

        if let Some(root) = root {
            self.bridge.set_project_root(root.clone()).await;
            let migrated_files = {
                let mut store = self.bridge.store.write().await;
                store.migrate_relative_projects_to_root(&root)
            };
            if !migrated_files.is_empty() {
                info!(
                    ?root,
                    files = migrated_files.len(),
                    "migrated relative project findings to LSP root"
                );
                self.bridge.notify_change(root.clone(), migrated_files);
            }
            *self.project_root.write().await = Some(root);
        }
    }

    async fn publish_file_diagnostics(&self, project: &Path, file: &str) {
        let diagnostics = self.file_diagnostics(project, file).await;
        info!(
            ?project,
            file,
            count = diagnostics.len(),
            "publishing diagnostics"
        );

        if let Ok(uri) = Url::from_file_path(project.join(file)) {
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn publish_all_diagnostics(&self) {
        let Some(project) = self.project_root.read().await.clone() else {
            return;
        };
        let files = {
            let store = self.bridge.store.read().await;
            store.get_active_files(&project)
        };
        for file in files {
            self.publish_file_diagnostics(&project, &file).await;
        }
    }

    async fn file_diagnostics(&self, project: &Path, file: &str) -> Vec<Diagnostic> {
        let store = self.bridge.store.read().await;
        store
            .get_file_findings(project, file)
            .into_iter()
            .map(finding_to_diagnostic)
            .collect()
    }

    async fn file_from_uri(&self, uri: &Url) -> Option<(PathBuf, String)> {
        let project = self.project_root.read().await.clone()?;
        let file = uri
            .to_file_path()
            .ok()
            .and_then(|path| path.strip_prefix(&project).ok().map(path_to_lsp_file))?;
        Some((project, file))
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LspBackend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.set_project_root(&params).await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        inter_file_dependencies: false,
                        workspace_diagnostics: true,
                        ..Default::default()
                    },
                )),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "ultrareview-bridge.dismiss".to_string(),
                        "ultrareview-bridge.dismiss-file".to_string(),
                    ],
                    ..Default::default()
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("ultrareview-bridge LSP initialized");
        self.client
            .log_message(MessageType::INFO, "ultrareview-bridge LSP initialized")
            .await;
        self.spawn_event_listener();
        self.publish_all_diagnostics().await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let diagnostics = match self.file_from_uri(&params.text_document.uri).await {
            Some((project, file)) => self.file_diagnostics(&project, &file).await,
            None => Vec::new(),
        };

        Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
                related_documents: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: None,
                    items: diagnostics,
                },
            }),
        ))
    }

    async fn workspace_diagnostic(
        &self,
        _: WorkspaceDiagnosticParams,
    ) -> Result<WorkspaceDiagnosticReportResult> {
        let Some(project) = self.project_root.read().await.clone() else {
            return Ok(WorkspaceDiagnosticReportResult::Report(
                WorkspaceDiagnosticReport { items: Vec::new() },
            ));
        };

        let files = {
            let store = self.bridge.store.read().await;
            store.get_active_files(&project)
        };
        let mut items = Vec::new();
        for file in files {
            if let Ok(uri) = Url::from_file_path(project.join(&file)) {
                items.push(WorkspaceDocumentDiagnosticReport::Full(
                    WorkspaceFullDocumentDiagnosticReport {
                        uri,
                        version: None,
                        full_document_diagnostic_report: FullDocumentDiagnosticReport {
                            result_id: None,
                            items: self.file_diagnostics(&project, &file).await,
                        },
                    },
                ));
            }
        }

        Ok(WorkspaceDiagnosticReportResult::Report(
            WorkspaceDiagnosticReport { items },
        ))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let Some(project) = self.project_root.read().await.clone() else {
            return Ok(None);
        };
        let Some(file) = params
            .text_document
            .uri
            .to_file_path()
            .ok()
            .and_then(|path| path.strip_prefix(&project).ok().map(path_to_lsp_file))
        else {
            return Ok(None);
        };

        let matching: Vec<Finding> = {
            let store = self.bridge.store.read().await;
            store
                .get_file_findings(&project, &file)
                .into_iter()
                .filter(|finding| finding.line.saturating_sub(1) == params.range.start.line)
                .cloned()
                .collect()
        };

        if matching.is_empty() {
            return Ok(None);
        }

        let mut actions = Vec::new();
        for finding in &matching {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Dismiss this finding: {}", finding.title),
                kind: Some(CodeActionKind::QUICKFIX),
                command: Some(Command {
                    title: "Dismiss this finding".to_string(),
                    command: "ultrareview-bridge.dismiss".to_string(),
                    arguments: Some(vec![serde_json::Value::String(finding.id.clone())]),
                }),
                diagnostics: Some(vec![finding_to_diagnostic(finding)]),
                ..Default::default()
            }));
        }

        if let Some(first) = matching.first() {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("Dismiss all {} findings in this file", first.source),
                kind: Some(CodeActionKind::QUICKFIX),
                command: Some(Command {
                    title: "Dismiss all findings in this file".to_string(),
                    command: "ultrareview-bridge.dismiss-file".to_string(),
                    arguments: Some(vec![
                        serde_json::Value::String(first.source.clone()),
                        serde_json::Value::String(file),
                    ]),
                }),
                ..Default::default()
            }));
        }

        Ok(Some(actions))
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            "ultrareview-bridge.dismiss" => {
                let Some(id) = params.arguments.first().and_then(|arg| arg.as_str()) else {
                    return Ok(None);
                };
                let affected = self.bridge.store.write().await.dismiss_finding(id);
                if let Some((project, file)) = affected {
                    self.publish_file_diagnostics(&project, &file).await;
                }
            }
            "ultrareview-bridge.dismiss-file" => {
                let Some(project) = self.project_root.read().await.clone() else {
                    return Ok(None);
                };
                let Some(source) = params.arguments.first().and_then(|arg| arg.as_str()) else {
                    return Ok(None);
                };
                let Some(file) = params.arguments.get(1).and_then(|arg| arg.as_str()) else {
                    return Ok(None);
                };
                let affected = self
                    .bridge
                    .store
                    .write()
                    .await
                    .dismiss_file_findings(&project, source, file);
                for affected_file in affected {
                    self.publish_file_diagnostics(&project, &affected_file)
                        .await;
                }
            }
            _ => {}
        }

        Ok(None)
    }
}

pub fn finding_to_diagnostic(finding: &Finding) -> Diagnostic {
    let start = Position {
        line: finding.line.saturating_sub(1),
        character: finding.col.saturating_sub(1),
    };
    let range = Range {
        start,
        end: Position {
            line: start.line,
            character: start.character + 20,
        },
    };

    Diagnostic {
        range,
        severity: Some(match finding.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
            Severity::Info => DiagnosticSeverity::INFORMATION,
        }),
        code: Some(NumberOrString::String(finding.category.clone())),
        source: Some(finding.source.clone()),
        message: diagnostic_message(finding),
        ..Default::default()
    }
}

fn diagnostic_message(finding: &Finding) -> String {
    let mut message = finding.title.clone();
    if let Some(evidence) = &finding.evidence {
        message.push_str(&format!("\n\nEvidence: {evidence}"));
    }
    if let Some(rationale) = &finding.rationale {
        message.push_str(&format!("\n\nRationale: {rationale}"));
    }
    if let Some(suggestion) = &finding.suggestion {
        message.push_str(&format!("\n\nSuggestion: {suggestion}"));
    }
    if let Some(verified_by) = &finding.verified_by {
        message.push_str(&format!("\n\n[verified by {verified_by}]"));
    }
    message
}

fn path_to_lsp_file(path: &Path) -> String {
    path.to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}
