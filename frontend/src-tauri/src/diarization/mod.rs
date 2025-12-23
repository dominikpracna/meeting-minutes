pub mod model;
pub mod extraction;
pub mod clustering;
pub mod commands;

use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;
use crate::diarization::model::EmbeddingModel;
use crate::diarization::clustering::OnlineClustering;

pub struct DiarizationEngine {
    model: Arc<RwLock<Option<EmbeddingModel>>>,
    clustering: Arc<RwLock<OnlineClustering>>,
}

impl DiarizationEngine {
    pub fn new() -> Self {
        Self {
            model: Arc::new(RwLock::new(None)),
            clustering: Arc::new(RwLock::new(OnlineClustering::new(0.5))), // 0.5 threshold
        }
    }

    pub async fn load_model(&self, path: PathBuf) -> anyhow::Result<()> {
        let model = EmbeddingModel::new(&path)?;
        *self.model.write().await = Some(model);
        Ok(())
    }
}
