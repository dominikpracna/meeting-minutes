use anyhow::{anyhow, Result};
use crate::diarization::model::EmbeddingModel;

pub struct EmbeddingExtractor {
    model: EmbeddingModel,
}

impl EmbeddingExtractor {
    pub fn new(model: EmbeddingModel) -> Self {
        Self { model }
    }

    pub fn compute_embedding(&mut self, samples: &[f32]) -> Result<Vec<f32>> {
        // Pre-processing if needed (e.g. normalization)
        // Wespeaker models typically expect raw audio, maybe normalized to -1..1 or similar.
        // Assuming samples are already float32 -1..1

        // Ensure minimum length?
        if samples.len() < 200 { // Very short segments might fail or give bad embeddings
             return Err(anyhow!("Segment too short for embedding extraction"));
        }

        self.model.extract_embedding(samples)
    }
}
