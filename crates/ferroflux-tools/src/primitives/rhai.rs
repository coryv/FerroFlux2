use ferroflux_types::DataRef;
use ferroflux_types::tool::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::Value;

use base64::{Engine as _, engine::general_purpose};

pub struct RhaiTool {
    engine: rhai::Engine,
}

impl Default for RhaiTool {
    fn default() -> Self {
        let mut engine = rhai::Engine::new();

        // Register Base64 Helpers
        engine.register_fn("base64_encode", |s: String| {
            general_purpose::STANDARD.encode(s)
        });
        engine.register_fn("base64_url_encode", |s: String| {
            general_purpose::URL_SAFE_NO_PAD.encode(s)
        });

        Self { engine }
    }
}

impl Tool for RhaiTool {
    fn id(&self) -> &'static str {
        "rhai"
    }

    fn run(&self, context: &mut ToolContext, params: Value) -> Result<Value> {
        let script = params
            .get("script")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing 'script'"))?;

        // Input binding
        // Using "input" param if present
        let input_val = params.get("input").unwrap_or(&Value::Null);

        let mut scope = rhai::Scope::new();

        // Inject entire context as variables (resolving DataRefs)
        for (k, v) in context.local.iter() {
            let val_opt = match v {
                DataRef::Inline(val) => Some(val.clone()),
                DataRef::Blob(ticket) => {
                    if let Some(store) = context.store {
                        if let Ok(bytes) = store.claim(ticket) {
                            serde_json::from_slice(&bytes).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            };

            if let Some(val) = val_opt
                && let Ok(dynamic) = rhai::serde::to_dynamic(&val)
            {
                scope.push_dynamic(k, dynamic);
            }
        }

        // Optional specific binding (overrides context if name collision, or explicit input)
        if !input_val.is_null() {
            let dynamic_input = rhai::serde::to_dynamic(input_val)?;
            scope.push_dynamic("input", dynamic_input);
        }

        // Eval
        let result = self
            .engine
            .eval_with_scope::<rhai::Dynamic>(&mut scope, script)?;

        let json_result: Value = rhai::serde::from_dynamic(&result)?;
        Ok(serde_json::json!({ "result": json_result }))
    }
}
