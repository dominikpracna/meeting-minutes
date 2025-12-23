use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct AudioInput {
    stream: cpal::Stream,
}

impl AudioInput {
    pub fn new(sender: mpsc::UnboundedSender<Vec<f32>>) -> Result<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device found"))?;

        log::info!("Using input device: {}", device.name()?);

        let config = device.default_input_config()?;
        log::info!("Default config: {:?}", config);

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => Self::build_stream::<f32>(&device, &config.into(), sender)?,
            cpal::SampleFormat::I16 => Self::build_stream::<i16>(&device, &config.into(), sender)?,
            cpal::SampleFormat::U16 => Self::build_stream::<u16>(&device, &config.into(), sender)?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format")),
        };

        stream.play()?;

        Ok(Self { stream })
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        sender: mpsc::UnboundedSender<Vec<f32>>,
    ) -> Result<cpal::Stream>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: From<T>,
    {
        let channels = config.channels as usize;
        let sender = sender.clone();

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &_| {
                // Convert to f32 and mix to mono if needed
                let samples: Vec<f32> = data.chunks(channels)
                    .map(|chunk| {
                        let sum: f32 = chunk.iter().map(|&s| f32::from(s)).sum();
                        sum / channels as f32
                    })
                    .collect();

                if let Err(e) = sender.send(samples) {
                    log::error!("Failed to send audio samples: {}", e);
                }
            },
            |err| log::error!("Stream error: {}", err),
            None,
        )?;

        Ok(stream)
    }
}
