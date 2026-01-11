use crate::components::execution_state::DataRef;
use crate::tools::{Tool, ToolContext};
use anyhow::{Result, anyhow};
use serde_json::Value;

pub struct RhaiTool {
    engine: rhai::Engine,
}

impl Default for RhaiTool {
    fn default() -> Self {
        Self {
            engine: rhai::Engine::new(),
        }
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

            if let Some(val) = val_opt {
                if let Ok(dynamic) = rhai::serde::to_dynamic(&val) {
                    scope.push_dynamic(k, dynamic);
                }
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
