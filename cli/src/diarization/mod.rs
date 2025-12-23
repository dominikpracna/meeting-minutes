pub mod model;
pub mod clustering;

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

    // Smart loading: only load if not already loaded
    pub async fn load_model_if_needed(&self, path: PathBuf) -> anyhow::Result<()> {
        // Read lock first to check
        if self.model.read().await.is_some() {
            return Ok(());
        }

        // Load
        self.load_model(path).await
    }

    pub async fn process_segment(&self, audio_samples: &[f32]) -> anyhow::Result<usize> {
        let mut model_guard = self.model.write().await;
        if let Some(model) = model_guard.as_mut() {
            if audio_samples.len() < 200 {
                 return Ok(0); // Too short
            }

            // Call extract_embedding directly on the model
            let embedding = model.extract_embedding(audio_samples)?;

            let mut clustering = self.clustering.write().await;
            let speaker_id = clustering.process_segment(&embedding);
            Ok(speaker_id)
        } else {
            Err(anyhow::anyhow!("Diarization model not loaded"))
        }
    }
}
