use tauri::{command, AppHandle, Emitter, Manager};
use crate::diarization::DiarizationEngine;
use std::sync::Arc;
use tokio::sync::Mutex;
use lazy_static::lazy_static;
use log::{info, error};
use reqwest::Client;
use std::path::PathBuf;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use std::fs;

// Global instance wrapped in lazy_static because Mutex::new is not const in tokio
lazy_static! {
    pub static ref DIARIZATION_ENGINE: Mutex<Option<Arc<DiarizationEngine>>> = Mutex::new(None);
}

const WESPEAKER_MODEL_URL: &str = "https://huggingface.co/snowkylin/wespeaker-voxceleb-resnet34-LM-onnx/resolve/main/voxceleb_resnet34_LM.onnx";

const MODEL_FILENAME: &str = "wespeaker_resnet34.onnx";

#[command]
pub async fn diarization_init() -> Result<(), String> {
    let mut guard = DIARIZATION_ENGINE.lock().await;
    if guard.is_none() {
        *guard = Some(Arc::new(DiarizationEngine::new()));
        info!("Diarization engine initialized");
    }
    Ok(())
}

#[command]
pub async fn diarize_segment(
    app: AppHandle,
    audio_data: Vec<f32>,
    _segment_id: String
) -> Result<usize, String> {
    let guard = DIARIZATION_ENGINE.lock().await;
    if let Some(engine) = guard.as_ref() {
        // Ensure model is loaded
        let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let model_path = app_data_dir.join("models").join("diarization").join(MODEL_FILENAME);

        if !model_path.exists() {
            return Ok(0); // No model, no speaker ID
        }

        // Lazy load model if not already loaded
        // We check internal state indirectly or rely on engine to handle it gracefully.
        // But engine.load_model overwrites.
        // Let's add a check in engine (which we can't easily access from here without exposing a method)
        // OR assume engine handles it.
        // Ideally DiarizationEngine should have `ensure_model_loaded(path)`.
        // For now, let's just load it. The engine logic locks write, so it's safe but slow if repeated.
        // The optimization is to expose `is_model_loaded` in DiarizationEngine.

        // Actually, DiarizationEngine implementation (in mod.rs) stores model in RwLock.
        // I can just call load_model. To optimize, I should check first.
        // But `DiarizationEngine` struct definition is public but fields are private/public?
        // Let's just update load_model in mod.rs to be smart (I already did that? No, I implemented load_model to overwrite).
        // Let's check mod.rs again.

        // In mod.rs:
        // pub async fn load_model(&self, path: PathBuf) -> anyhow::Result<()> {
        //    let model = EmbeddingModel::new(&path)?;
        //    *self.model.write().await = Some(model);
        //    Ok(())
        // }
        // It always reloads.

        // I should fix mod.rs to check first.

        engine.load_model_if_needed(model_path).await.map_err(|e| e.to_string())?;

        let speaker_id = engine.process_segment(&audio_data).await.map_err(|e| e.to_string())?;
        Ok(speaker_id)
    } else {
        Err("Diarization engine not initialized".to_string())
    }
}

#[command]
pub async fn diarization_download_model(app: AppHandle) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let models_dir = app_data_dir.join("models").join("diarization");

    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;
    }

    let model_path = models_dir.join(MODEL_FILENAME);
    if model_path.exists() {
        info!("Diarization model already exists");
        let _ = app.emit("diarization-model-download-complete", ());
        return Ok(());
    }

    info!("Downloading diarization model from {}", WESPEAKER_MODEL_URL);
    let client = Client::new();
    let response = client.get(WESPEAKER_MODEL_URL).send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Failed to download model: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(&model_path).await.map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
            if progress % 5 == 0 { // Emit every 5%
                 let _ = app.emit("diarization-model-download-progress", progress);
            }
        }
    }

    info!("Diarization model downloaded successfully");
    let _ = app.emit("diarization-model-download-complete", ());
    Ok(())
}
