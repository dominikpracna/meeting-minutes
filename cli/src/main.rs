use clap::{Parser, Subcommand};
use std::sync::Arc;
use tokio::sync::mpsc;
use std::path::PathBuf;
use log::{info, error, warn};

// Import modules
#[macro_use]
mod utils;
mod audio;
mod whisper_engine;
mod diarization;

use whisper_engine::WhisperEngine;
use diarization::DiarizationEngine;
use audio::AudioInput;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start recording and transcribing
    Record {
        /// Output directory for transcripts
        #[arg(short, long, default_value = "transcripts")]
        output: String,

        /// Model to use (tiny, base, small, medium, large-v3)
        #[arg(short, long, default_value = "base")]
        model: String,
    },
    /// List audio devices
    ListDevices,
    /// Download models
    DownloadModels,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::ListDevices => {
             // For now, just print default device info via cpal default host
             // Since we simplified audio module, we don't have list function ready
             info!("Listing default input device...");
             let host = cpal::default_host();
             use cpal::traits::{DeviceTrait, HostTrait};
             if let Some(device) = host.default_input_device() {
                 info!("Default Input Device: {}", device.name().unwrap_or_default());
             } else {
                 error!("No default input device found.");
             }
        }
        Commands::Record { output: _, model } => {
            info!("Starting recording session...");

            // Setup models directory
            let models_dir = std::env::current_dir()?.join("models");
            if !models_dir.exists() {
                std::fs::create_dir_all(&models_dir)?;
            }

            // Initialize Engines
            let whisper = Arc::new(WhisperEngine::new_with_models_dir(Some(models_dir.clone()))?);
            let diarization = Arc::new(DiarizationEngine::new());

            // Check/Load Whisper Model
            info!("Loading Whisper model: {}", model);
            if let Err(e) = whisper.load_model(model).await {
                error!("Failed to load whisper model: {}. Try running 'meetily-cli download-models'", e);
                return Ok(());
            }

            // Check/Load Diarization Model
            let diarization_model_path = models_dir.join("diarization").join("wespeaker_resnet34.onnx");
            if diarization_model_path.exists() {
                info!("Loading Diarization model...");
                if let Err(e) = diarization.load_model(diarization_model_path).await {
                     warn!("Failed to load diarization model: {}. Diarization will be disabled.", e);
                }
            } else {
                warn!("Diarization model not found at {}. Diarization will be disabled.", diarization_model_path.display());
            }

            // Start Audio Capture
            let (tx, mut rx) = mpsc::unbounded_channel::<Vec<f32>>();
            let _audio_input = AudioInput::new(tx)?;

            info!("Recording started. Press Ctrl+C to stop.");

            let mut audio_buffer: Vec<f32> = Vec::new();
            let sample_rate = 16000; // Assuming 16kHz
            let chunk_duration_s = 5; // Process every 5 seconds
            let chunk_size = sample_rate * chunk_duration_s;

            // Main processing loop
            while let Some(samples) = rx.recv().await {
                audio_buffer.extend(samples);

                if audio_buffer.len() >= chunk_size {
                    // Process chunk
                    let processing_chunk = audio_buffer.clone();
                    // Clear buffer (simple approach, ideally overlap)
                    audio_buffer.clear();

                    let whisper_clone = whisper.clone();
                    let diarization_clone = diarization.clone();

                    tokio::spawn(async move {
                         // Transcribe
                         match whisper_clone.transcribe_audio(processing_chunk.clone(), None).await {
                             Ok(text) => {
                                 if !text.trim().is_empty() {
                                     // Diarize
                                     let speaker_id = match diarization_clone.process_segment(&processing_chunk).await {
                                         Ok(id) => format!("Speaker {}", id),
                                         Err(_) => "Unknown".to_string(),
                                     };

                                     println!("[{}] {}", speaker_id, text);
                                 }
                             },
                             Err(e) => error!("Transcription failed: {}", e),
                         }
                    });
                }
            }
        }
        Commands::DownloadModels => {
             info!("Downloading models...");
             let models_dir = std::env::current_dir()?.join("models");
             let whisper = WhisperEngine::new_with_models_dir(Some(models_dir.clone()))?;

             info!("Downloading 'base' whisper model...");
             if let Err(e) = whisper.download_model("base", None).await {
                 error!("Failed to download base model: {}", e);
             }

             // Download Diarization model (manual implementation since we removed Diarization commands)
             let diarization_dir = models_dir.join("diarization");
             std::fs::create_dir_all(&diarization_dir)?;
             let model_url = "https://huggingface.co/snowkylin/wespeaker-voxceleb-resnet34-LM-onnx/resolve/main/voxceleb_resnet34_LM.onnx";
             let model_path = diarization_dir.join("wespeaker_resnet34.onnx");

             if !model_path.exists() {
                 info!("Downloading diarization model...");
                 let response = reqwest::get(model_url).await?;
                 let content = response.bytes().await?;
                 std::fs::write(model_path, content)?;
                 info!("Diarization model downloaded.");
             } else {
                 info!("Diarization model already exists.");
             }

             info!("All models ready.");
        }
    }

    Ok(())
}
