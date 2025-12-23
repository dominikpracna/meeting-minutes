pub mod capture;
pub use capture::AudioInput;

#[derive(Debug, Clone)]
pub enum GpuType {
    None,
    Cuda,
    Metal,
    Vulkan,
    OpenCL,
}

#[derive(Debug, Clone)]
pub enum PerformanceTier {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone)]
pub struct WhisperConfig {
    pub use_gpu: bool,
    pub beam_size: u32,
    pub temperature: f32,
    pub max_threads: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct HardwareProfile {
    pub gpu_type: GpuType,
    pub performance_tier: PerformanceTier,
}

impl HardwareProfile {
    pub fn detect() -> Self {
        // Simplified detection
        let gpu_type = if cfg!(target_os = "macos") {
            GpuType::Metal
        } else if cfg!(feature = "cuda") {
            GpuType::Cuda
        } else {
            GpuType::None
        };

        Self {
            gpu_type,
            performance_tier: PerformanceTier::Medium, // Default
        }
    }

    pub fn get_whisper_config(&self) -> WhisperConfig {
        WhisperConfig {
            use_gpu: !matches!(self.gpu_type, GpuType::None),
            beam_size: 5,
            temperature: 0.0,
            max_threads: Some(4),
        }
    }
}
