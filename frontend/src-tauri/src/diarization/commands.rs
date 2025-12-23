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

        // Lazy load model if needed (this might be slow on first call)
        // In a real implementation we should load it once.
        // For now let's try to load it if not loaded.
        // But load_model is async and takes &self (non-mut) but locks internal RwLock.
        engine.load_model(model_path).await.map_err(|e| e.to_string())?;

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
