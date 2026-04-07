use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::{Value, json};

pub struct AgentTool;

impl Tool for AgentTool {
    fn id(&self) -> &'static str {
        "agent"
    }

    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let provider = params.get("provider").and_then(|v| v.as_str()).unwrap_or("openai");
        let model = params.get("model").and_then(|v| v.as_str()).unwrap_or("gpt-4o");
        let system = params.get("system").and_then(|v| v.as_str()).unwrap_or("");
        let prompt = params.get("prompt").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing 'prompt'"))?;

        tracing::info!(provider, model, "AgentTool: delegating to platform action");

        let executor = context.executor.ok_or_else(|| {
            anyhow!("ActionExecutor not available in ToolContext. Cannot delegate Agent call.")
        })?;

        let (action_id, sub_params) = match provider {
            "openai" | "azure-openai" => (
                "openai.chat.completions",
                json!({
                    "user_prompt": prompt,
                    "system_prompt": system,
                    "model": model,
                })
            ),
            "anthropic" => (
                "anthropic.messages.create",
                json!({
                    "messages": [{"role": "user", "content": prompt}],
                    "system": system,
                    "model": model,
                })
            ),
            "gemini" => (
                "gemini.ai.chat",
                json!({
                    "model": model,
                    "message": prompt,
                    "history": [{"role": "user", "parts": [{"text": prompt}]}],
                })
            ),
            "mistral" => (
                "mistral.chat.completion",
                json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": system},
                        {"role": "user", "content": prompt}
                    ],
                })
            ),
            "ollama_cloud" | "ollama" => (
                "ollama_cloud.ai.chat",
                json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": system},
                        {"role": "user", "content": prompt}
                    ],
                })
            ),
            _ => return Err(anyhow!("Unsupported agent provider: {}", provider)),
        };

        let tenant_id = ferroflux_types::tenant::TenantId::from(context.tenant_id.as_str());
        let result = executor.execute(&tenant_id, action_id, sub_params, context)?;

        // Standardize the output – most actions return the text in a 'response' or 'text' field
        let final_text = if let Some(resp) = result.get("response").and_then(|v| v.as_str()) {
            resp.to_string()
        } else if let Some(txt) = result.get("text").and_then(|v| v.as_str()) {
            txt.to_string()
        } else if result.is_string() {
            result.as_str().unwrap().to_string()
        } else {
            // Fallback to debug string if we can't find a clear text field
            serde_json::to_string(&result)?
        };

        Ok(json!({ "result": final_text }))
    }
}
