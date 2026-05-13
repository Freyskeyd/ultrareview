use crate::store::FindingsStore;
use crate::types::StoreEvent;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct BridgeState {
    pub store: Arc<RwLock<FindingsStore>>,
    pub event_tx: broadcast::Sender<StoreEvent>,
    project_root: Arc<RwLock<Option<PathBuf>>>,
}

impl BridgeState {
    pub fn new(store: FindingsStore) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            store: Arc::new(RwLock::new(store)),
            event_tx,
            project_root: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_project_root(&self, project_root: PathBuf) {
        *self.project_root.write().await = Some(project_root);
    }

    pub async fn project_root(&self) -> Option<PathBuf> {
        self.project_root.read().await.clone()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<StoreEvent> {
        self.event_tx.subscribe()
    }

    pub fn notify_change(&self, project: PathBuf, affected_files: Vec<String>) {
        if affected_files.is_empty() {
            return;
        }

        let _ = self.event_tx.send(StoreEvent::FindingsChanged {
            project,
            affected_files,
        });
    }
}
