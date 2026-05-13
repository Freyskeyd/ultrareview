use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub source: String,
    pub project: PathBuf,
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub severity: Severity,
    pub category: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_by: Option<String>,
    #[serde(default)]
    pub dismissed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
pub struct FindingInput {
    pub file: String,
    pub line: u32,
    #[serde(default = "default_col")]
    pub col: u32,
    pub severity: SeverityInput,
    pub category: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_by: Option<String>,
}

fn default_col() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SeverityInput {
    Error,
    Warning,
    Info,
}

impl From<SeverityInput> for Severity {
    fn from(severity: SeverityInput) -> Self {
        match severity {
            SeverityInput::Error => Self::Error,
            SeverityInput::Warning => Self::Warning,
            SeverityInput::Info => Self::Info,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DismissKey {
    pub project: PathBuf,
    pub source: String,
    pub file: String,
    pub line: u32,
    pub category: String,
    pub title: String,
}

impl Finding {
    pub fn dismiss_key(&self) -> DismissKey {
        DismissKey {
            project: self.project.clone(),
            source: self.source.clone(),
            file: self.file.clone(),
            line: self.line,
            category: self.category.clone(),
            title: self.title.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreEvent {
    FindingsChanged {
        project: PathBuf,
        affected_files: Vec<String>,
    },
}
