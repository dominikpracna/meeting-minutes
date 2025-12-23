use anyhow::{anyhow, Result};
use ort::{session::{Session, builder::GraphOptimizationLevel}, value::Value};
use std::path::PathBuf;

pub struct EmbeddingModel {
    session: Session,
}

impl EmbeddingModel {
    pub fn new(model_path: &PathBuf) -> Result<Self> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;

        Ok(Self { session })
    }

    pub fn extract_embedding(&mut self, audio_samples: &[f32]) -> Result<Vec<f32>> {
        // Wespeaker ResNet34 expects [Batch, Samples] or [Batch, 1, Samples] depending on export
        // Typically for ONNX exported from Wespeaker it expects [Batch, Samples]

        let input_shape = vec![1, audio_samples.len() as i64];
        let input_tensor = Value::from_array(
            (input_shape, audio_samples.to_vec())
        )?;

        let inputs = ort::inputs!["speech" => input_tensor];
        let outputs = self.session.run(inputs)?;

        // Output is usually [Batch, EmbeddingDim]
        // Get the first output
        let output_tensor = outputs[0].try_extract_tensor::<f32>()?;

        // output_tensor is (&Shape, &[f32])
        let (_shape, data) = output_tensor;
        let embedding = data.to_vec();

        Ok(embedding)
    }
}
