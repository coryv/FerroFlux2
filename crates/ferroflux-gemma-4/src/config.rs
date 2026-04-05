use crate::model_selector::ModelVariant;
use std::path::PathBuf;

pub struct AgentConfig {
    /// Override automatic model selection (None = smart auto-select)
    pub model_variant: Option<ModelVariant>,
    /// Override HuggingFace model ID (optional, defaults to profile ID)
    pub model_id: Option<String>,
    /// Local directory for cached/pre-downloaded weights (optional)
    pub cache_dir: Option<PathBuf>,
    /// HF revision (default: "main")
    pub revision: String,
    /// Max tokens to generate per response (default: 10000)
    pub max_new_tokens: usize,
    /// Sampling parameters (model card defaults)
    pub temperature: f64,   // Default: 1.0
    pub top_p: f64,         // Default: 0.95
    pub top_k: usize,       // Default: 64
    /// Repeat penalty (default: 1.1)
    pub repeat_penalty: f32,
    pub repeat_last_n: usize, // Default: 64
    /// Enable thinking mode (triggers <|think|> token)
    pub enable_thinking: bool,
    /// RNG seed (default: fixed for reproducibility)
    pub seed: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model_variant: None,
            model_id: None,
            cache_dir: None,
            revision: "main".to_string(),
            max_new_tokens: 10000,
            temperature: 1.0,
            top_p: 0.95,
            top_k: 64,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            enable_thinking: false,
            seed: 299792458,
        }
    }
}
