use crate::config::AgentConfig;
use crate::device::{select_device, select_dtype};
use crate::model_selector::{get_profile, select_model, ModelProfile};
use crate::parse::ResponseParser;
use crate::syscheck::SystemResources;
use crate::template::PromptTemplate;
use crate::types::{AgentResponse, ChatMessage, ToolDefinition};

use anyhow::{anyhow, Result};
use candle_core::{DType, Device, Tensor};
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::gemma4::{config::Gemma4TextConfig, text::TextModel};
use crate::quantized_gemma4::ModelWeights;
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

enum LoadedModel {
    Full(TextModel),
    Quantized(ModelWeights),
}

pub struct GemmaAgent {
    model: LoadedModel,
    tokenizer: Tokenizer,
    config: AgentConfig,
    profile: &'static ModelProfile,
    device: Device,
}

impl GemmaAgent {
    /// Loads the best model for the system and initializes the agent.
    pub fn load(config: AgentConfig) -> Result<Self> {
        let resources = SystemResources::probe();
        let profile = match &config.model_variant {
            Some(v) => get_profile(*v),
            None => select_model(resources.total_gb())?,
        };

        tracing::info!(
            "Loading model variant: {:?} (Peak RAM: {}GB)",
            profile.variant,
            profile.peak_memory_gb
        );

        let device = select_device()?;
        let _dtype = select_dtype(&device);

        // Fetch weights from HF
        let api = Api::new()?;
        let repo = api.repo(Repo::with_revision(
            profile.hf_repo.to_string(),
            RepoType::Model,
            config.revision.clone(),
        ));

        let base_repo = api.repo(Repo::with_revision(
            profile.base_repo.to_string(),
            RepoType::Model,
            config.revision.clone(),
        ));

        let tokenizer_file = base_repo.get("tokenizer.json")?;
        let tokenizer = Tokenizer::from_file(tokenizer_file).map_err(|e| anyhow!(e))?;

        let model = if profile.is_quantized {
            let weight_file = repo.get(profile.weight_file)?;
            let mut file = std::fs::File::open(weight_file)?;
            let content = candle_core::quantized::gguf_file::Content::read(&mut file)?;
            let weights = ModelWeights::from_gguf(content, &mut file, &device)?;
            LoadedModel::Quantized(weights)
        } else {
            // Text-only loading for Gemma 4 full-precision
            let config_file = repo.get("config.json")?;
            let config_json: serde_json::Value =
                serde_json::from_reader(std::fs::File::open(config_file)?)?;

            let text_config: Gemma4TextConfig = if let Some(text_cfg) = config_json.get("text_config")
            {
                serde_json::from_value(text_cfg.clone())?
            } else {
                serde_json::from_value(config_json)?
            };

            // Download weights
            let weight_files = match repo.get("model.safetensors.index.json") {
                Ok(_) => hub_load_safetensors(&repo, "model.safetensors.index.json")?,
                Err(_) => vec![repo.get("model.safetensors")?],
            };

            let vb = unsafe {
                candle_nn::VarBuilder::from_mmaped_safetensors(&weight_files, _dtype, &device)?
            };
            let text_model = TextModel::new(&text_config, vb)?;
            LoadedModel::Full(text_model)
        };

        Ok(Self {
            model,
            tokenizer,
            config,
            profile,
            device,
        })
    }

    /// Performs a full chat interaction.
    pub fn chat(
        &mut self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> Result<AgentResponse> {
        let prompt = PromptTemplate::format(messages, tools, self.config.enable_thinking);
        let tokens = self.tokenizer.encode(prompt, true).map_err(|e| anyhow!(e))?;
        let mut tokens = tokens.get_ids().to_vec();

        let mut logits_processor = {
            let sampling = if self.config.temperature <= 0.0 {
                Sampling::ArgMax
            } else {
                Sampling::TopKThenTopP {
                    k: self.config.top_k,
                    p: self.config.top_p,
                    temperature: self.config.temperature,
                }
            };
            LogitsProcessor::from_sampling(self.config.seed, sampling)
        };

        let mut generated_tokens = Vec::new();
        let start_time = std::time::Instant::now();

        // EOS tokens for Gemma 4
        let eos_tokens = [1u32, 106u32, 50u32];

        for i in 0..self.config.max_new_tokens {
            let context_size = if i > 0 { 1 } else { tokens.len() };
            let pos = tokens.len().saturating_sub(context_size);
            let input = Tensor::new(&tokens[pos..], &self.device)?.unsqueeze(0)?;

            let logits = match &mut self.model {
                LoadedModel::Full(m) => m.forward(&input, pos)?,
                LoadedModel::Quantized(m) => m.forward(&input, pos)?,
            };

            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;

            // Repeat penalty
            let logits = if self.config.repeat_penalty == 1.0 {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.config.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.config.repeat_penalty,
                    &tokens[start_at..],
                )?
            };

            let next_token = logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens.push(next_token);

            if eos_tokens.contains(&next_token) {
                break;
            }
        }

        let duration = start_time.elapsed();
        let raw_output = self
            .tokenizer
            .decode(&generated_tokens, true)
            .map_err(|e| anyhow!(e))?;

        Ok(ResponseParser::parse(
            &raw_output,
            self.profile.variant,
            generated_tokens.len(),
            duration,
        ))
    }

    /// Clears the model's internal KV cache.
    pub fn reset(&mut self) {
        match &mut self.model {
            LoadedModel::Full(m) => m.clear_kv_cache(),
            LoadedModel::Quantized(_) => {
                // Quantized model forward resets it internally if pos=0.
            }
        }
    }

    /// Returns the active model profile
    pub fn model_profile(&self) -> &'static ModelProfile {
        self.profile
    }
}

/// Helper function to load multi-file safetensors from the hub index.
fn hub_load_safetensors(
    repo: &hf_hub::api::sync::ApiRepo,
    json_file: &str,
) -> Result<Vec<std::path::PathBuf>> {
    let json_file = repo.get(json_file)?;
    let json: serde_json::Value = serde_json::from_reader(std::fs::File::open(json_file)?)?;
    let weight_map = json
        .get("weight_map")
        .and_then(|v| v.as_object())
        .ok_or_else(|| anyhow!("no weight map in index"))?;

    let mut safetensors_files = std::collections::HashSet::new();
    for value in weight_map.values() {
        if let Some(file) = value.as_str() {
            safetensors_files.insert(file.to_string());
        }
    }
    let mut files = Vec::new();
    for file in safetensors_files {
        files.push(repo.get(&file)?);
    }
    Ok(files)
}
