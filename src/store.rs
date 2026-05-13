use crate::types::{DismissKey, Finding, FindingInput};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

type StoreKey = (PathBuf, String);

#[derive(Debug, Default)]
pub struct FindingsStore {
    findings: HashMap<StoreKey, Vec<Finding>>,
    dismissed: HashSet<DismissKey>,
    id_counters: HashMap<(String, String), usize>,
    persist: bool,
    cache_path: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CacheStore {
    findings: Vec<CacheEntry>,
    dismissed: HashSet<DismissKey>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    project: PathBuf,
    source: String,
    findings: Vec<Finding>,
}

impl FindingsStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cache_path(cache_path: PathBuf) -> Self {
        Self {
            persist: true,
            cache_path: Some(cache_path),
            ..Self::new()
        }
    }

    pub fn load_from_disk() -> Self {
        Self::load_from_path(Self::default_cache_path())
    }

    pub fn load_from_path(path: PathBuf) -> Self {
        let mut store = match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<CacheStore>(&contents) {
                Ok(cache) => {
                    info!(?path, "loaded findings from disk cache");
                    Self::from_cache(cache)
                }
                Err(error) => {
                    warn!(?path, %error, "corrupt disk cache, starting fresh");
                    Self::new()
                }
            },
            Err(_) => Self::new(),
        };
        store.persist = true;
        store.cache_path = Some(path);
        store.rebuild_id_counters();
        store
    }

    pub fn save_to_disk(&self) {
        if !self.persist {
            return;
        }

        let path = self
            .cache_path
            .clone()
            .unwrap_or_else(Self::default_cache_path);
        if let Some(parent) = path.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                warn!(%error, "failed to create cache directory");
                return;
            }
        }

        match serde_json::to_string_pretty(&self.to_cache()) {
            Ok(json) => {
                if let Err(error) = std::fs::write(&path, json) {
                    warn!(?path, %error, "failed to write disk cache");
                }
            }
            Err(error) => warn!(%error, "failed to serialize findings store"),
        }
    }

    fn default_cache_path() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("ultrareview-bridge")
            .join("findings.json")
    }

    pub fn post_findings(
        &mut self,
        project: PathBuf,
        source: String,
        inputs: Vec<FindingInput>,
    ) -> Vec<String> {
        let mut affected_files = Vec::new();
        let key = (project.clone(), source.clone());

        for input in inputs {
            let id = self.next_id(&source, &input.category);
            let dismiss_key = DismissKey {
                project: project.clone(),
                source: source.clone(),
                file: input.file.clone(),
                line: input.line,
                category: input.category.clone(),
                title: input.title.clone(),
            };
            let dismissed = self.dismissed.contains(&dismiss_key);

            if !affected_files.contains(&input.file) {
                affected_files.push(input.file.clone());
            }

            self.findings.entry(key.clone()).or_default().push(Finding {
                id,
                source: source.clone(),
                project: project.clone(),
                file: input.file,
                line: input.line,
                col: input.col,
                severity: input.severity.into(),
                category: input.category,
                title: input.title,
                evidence: input.evidence,
                rationale: input.rationale,
                suggestion: input.suggestion,
                verified_by: input.verified_by,
                dismissed,
            });
        }

        self.save_to_disk();
        affected_files
    }

    pub fn clear_findings(&mut self, project: &Path, source: &str) -> Vec<String> {
        let key = (project.to_path_buf(), source.to_string());
        let affected_files = self
            .findings
            .get(&key)
            .map(|findings| unique_files(findings))
            .unwrap_or_default();

        self.findings.remove(&key);
        self.save_to_disk();
        affected_files
    }

    pub fn dismiss_finding(&mut self, finding_id: &str) -> Option<(PathBuf, String)> {
        for findings in self.findings.values_mut() {
            if let Some(finding) = findings.iter_mut().find(|finding| finding.id == finding_id) {
                finding.dismissed = true;
                self.dismissed.insert(finding.dismiss_key());
                let affected = (finding.project.clone(), finding.file.clone());
                self.save_to_disk();
                return Some(affected);
            }
        }
        None
    }

    pub fn dismiss_file_findings(
        &mut self,
        project: &Path,
        source: &str,
        file: &str,
    ) -> Vec<String> {
        let key = (project.to_path_buf(), source.to_string());
        let mut affected = false;
        if let Some(findings) = self.findings.get_mut(&key) {
            for finding in findings.iter_mut().filter(|finding| finding.file == file) {
                finding.dismissed = true;
                self.dismissed.insert(finding.dismiss_key());
                affected = true;
            }
        }
        self.save_to_disk();
        if affected {
            vec![file.to_string()]
        } else {
            Vec::new()
        }
    }

    pub fn restore_findings(&mut self, project: &Path) -> Vec<String> {
        self.dismissed
            .retain(|dismiss_key| dismiss_key.project != project);
        let mut affected_files = Vec::new();

        for ((finding_project, _), findings) in &mut self.findings {
            if finding_project == project {
                for finding in findings.iter_mut().filter(|finding| finding.dismissed) {
                    finding.dismissed = false;
                    if !affected_files.contains(&finding.file) {
                        affected_files.push(finding.file.clone());
                    }
                }
            }
        }

        affected_files.sort();
        self.save_to_disk();
        affected_files
    }

    pub fn migrate_relative_projects_to_root(&mut self, root: &Path) -> Vec<String> {
        let relative_keys: Vec<StoreKey> = self
            .findings
            .keys()
            .filter(|(project, _)| project.is_relative())
            .cloned()
            .collect();

        let mut affected_files = Vec::new();
        for (relative_project, source) in relative_keys {
            let Some(mut findings) = self
                .findings
                .remove(&(relative_project.clone(), source.clone()))
            else {
                continue;
            };

            for finding in &mut findings {
                finding.project = root.to_path_buf();
                if !affected_files.contains(&finding.file) {
                    affected_files.push(finding.file.clone());
                }
            }

            self.findings
                .entry((root.to_path_buf(), source))
                .or_default()
                .extend(findings);
        }

        if !affected_files.is_empty() {
            self.dismissed = self
                .dismissed
                .drain()
                .map(|mut dismiss_key| {
                    if dismiss_key.project.is_relative() {
                        dismiss_key.project = root.to_path_buf();
                    }
                    dismiss_key
                })
                .collect();
            affected_files.sort();
            self.save_to_disk();
        }

        affected_files
    }

    pub fn get_file_findings(&self, project: &Path, file: &str) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|((finding_project, _), _)| finding_project == project)
            .flat_map(|(_, findings)| findings.iter())
            .filter(|finding| finding.file == file && !finding.dismissed)
            .collect()
    }

    pub fn get_project_findings(&self, project: &Path) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|((finding_project, _), _)| finding_project == project)
            .flat_map(|(_, findings)| findings.iter())
            .collect()
    }

    pub fn get_active_files(&self, project: &Path) -> Vec<String> {
        let mut files: Vec<String> = self
            .findings
            .iter()
            .filter(|((finding_project, _), _)| finding_project == project)
            .flat_map(|(_, findings)| findings.iter())
            .filter(|finding| !finding.dismissed)
            .map(|finding| finding.file.clone())
            .collect();
        files.sort();
        files.dedup();
        files
    }

    fn next_id(&mut self, source: &str, category: &str) -> String {
        let key = (source.to_string(), category.to_string());
        let counter = self.id_counters.entry(key).or_insert(0);
        let id = format!("{}-{}-{}", source, category, counter);
        *counter += 1;
        id
    }

    fn rebuild_id_counters(&mut self) {
        self.id_counters.clear();
        let counters: Vec<(String, String)> = self
            .findings
            .values()
            .flat_map(|findings| findings.iter())
            .map(|finding| (finding.source.clone(), finding.category.clone()))
            .collect();

        for (source, category) in counters {
            let _ = self.next_id(&source, &category);
        }
    }

    fn from_cache(cache: CacheStore) -> Self {
        let findings = cache
            .findings
            .into_iter()
            .map(|entry| ((entry.project, entry.source), entry.findings))
            .collect();

        Self {
            findings,
            dismissed: cache.dismissed,
            id_counters: HashMap::new(),
            persist: false,
            cache_path: None,
        }
    }

    fn to_cache(&self) -> CacheStore {
        let findings = self
            .findings
            .iter()
            .map(|((project, source), findings)| CacheEntry {
                project: project.clone(),
                source: source.clone(),
                findings: findings.clone(),
            })
            .collect();

        CacheStore {
            findings,
            dismissed: self.dismissed.clone(),
        }
    }
}

fn unique_files(findings: &[Finding]) -> Vec<String> {
    let mut files: Vec<String> = findings
        .iter()
        .map(|finding| finding.file.clone())
        .collect();
    files.sort();
    files.dedup();
    files
}
