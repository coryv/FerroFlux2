use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelVariant {
    E4bBf16,     // Full precision, 16GB weights, ~18GB peak
    E2bBf16,     // Full precision, ~10GB weights, ~12GB peak
    E4bQ8,       // Quantized 8-bit text-only, ~9GB peak
    E4bQ4,       // Quantized 4-bit text-only, ~6.5GB peak - Target for 8GB Mac
    E2bQ8,       // Quantized 8-bit text-only, ~6GB peak
    E2bQ4,       // Quantized 4-bit text-only, ~3.5GB peak
}

pub struct ModelProfile {
    pub variant: ModelVariant,
    pub hf_repo: &'static str,
    pub base_repo: &'static str,
    pub weight_file: &'static str,
    pub peak_memory_gb: f64,
    pub is_quantized: bool,
}

pub const SAFETY_MARGIN_GB: f64 = 1.5;

pub const PROFILES: &[ModelProfile] = &[
    ModelProfile {
        variant: ModelVariant::E4bBf16,
        hf_repo: "google/gemma-4-E4B-it",
        base_repo: "google/gemma-4-E4B-it",
        weight_file: "model.safetensors",
        peak_memory_gb: 18.0,
        is_quantized: false,
    },
    ModelProfile {
        variant: ModelVariant::E2bBf16,
        hf_repo: "google/gemma-4-E2B-it",
        base_repo: "google/gemma-4-E2B-it",
        weight_file: "model.safetensors",
        peak_memory_gb: 12.0,
        is_quantized: false,
    },
    ModelProfile {
        variant: ModelVariant::E4bQ8,
        hf_repo: "unsloth/gemma-4-E4B-it-GGUF",
        base_repo: "google/gemma-4-E4B-it",
        weight_file: "gemma-4-E4B-it-Q8_0.gguf",
        peak_memory_gb: 9.0,
        is_quantized: true,
    },
    ModelProfile {
        variant: ModelVariant::E4bQ4,
        hf_repo: "unsloth/gemma-4-E4B-it-GGUF",
        base_repo: "google/gemma-4-E4B-it",
        weight_file: "gemma-4-E4B-it-Q4_K_M.gguf",
        peak_memory_gb: 6.5,
        is_quantized: true,
    },
    ModelProfile {
        variant: ModelVariant::E2bQ8,
        hf_repo: "unsloth/gemma-4-E2B-it-GGUF",
        base_repo: "google/gemma-4-E2B-it",
        weight_file: "gemma-4-E2B-it-Q8_0.gguf",
        peak_memory_gb: 6.0,
        is_quantized: true,
    },
    ModelProfile {
        variant: ModelVariant::E2bQ4,
        hf_repo: "unsloth/gemma-4-E2B-it-GGUF",
        base_repo: "google/gemma-4-E2B-it",
        weight_file: "gemma-4-E2B-it-Q4_K_M.gguf",
        peak_memory_gb: 3.5,
        is_quantized: true,
    },
];

pub fn get_profile(variant: ModelVariant) -> &'static ModelProfile {
    PROFILES
        .iter()
        .find(|p| p.variant == variant)
        .expect("Variant not found in profiles")
}

/// Select the best model variant given system resources.
/// Tries the highest-quality model that fits in available memory.
pub fn select_model(available_gb: f64) -> Result<&'static ModelProfile> {
    PROFILES
        .iter()
        .find(|p| p.peak_memory_gb <= available_gb)
        .ok_or_else(|| {
            anyhow!(
                "Insufficient memory capacity: {:.1} GB system RAM, minimum 3.5 GB required",
                available_gb
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_logic() {
        // High RAM: 32GB
        assert_eq!(select_model(32.0).unwrap().variant, ModelVariant::E4bBf16);
        
        // Mid-High RAM: 16GB
        assert_eq!(select_model(16.0).unwrap().variant, ModelVariant::E2bBf16);
        
        // Mid RAM: 10GB
        assert_eq!(select_model(10.0).unwrap().variant, ModelVariant::E4bQ8);
        
        // Target RAM for 8GB Mac: 8GB
        assert_eq!(select_model(8.0).unwrap().variant, ModelVariant::E4bQ4);
        
        // Low RAM: 6GB
        assert_eq!(select_model(6.0).unwrap().variant, ModelVariant::E2bQ8);
        
        // Extremely Low RAM: 2GB
        assert!(select_model(2.0).is_err());
    }

    #[test]
    fn test_get_profile() {
        let profile = get_profile(ModelVariant::E4bQ4);
        assert_eq!(profile.variant, ModelVariant::E4bQ4);
        assert!(profile.is_quantized);
        assert!(profile.hf_repo.contains("GGUF"));
    }
}
