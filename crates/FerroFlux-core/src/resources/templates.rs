use crate::systems::io::templating::{cel_to_json, json_to_cel};
use bevy_ecs::prelude::*;
use cel_interpreter::{Context, Program, Value as CelValue};
use std::sync::Arc;

#[derive(Resource)]
pub struct CelEngine;

impl Default for CelEngine {
    fn default() -> Self {
        Self
    }
}

fn json_template_func(v: CelValue) -> Arc<String> {
    let json_val = cel_to_json(v);
    Arc::new(serde_json::to_string(&json_val).unwrap_or_else(|_| "null".to_string()))
}

impl CelEngine {
    pub fn render(
        &self,
        template: &str,
        data: &serde_json::Value,
    ) -> Result<String, String> {
        if template.trim().is_empty() {
            return Ok(template.to_string());
        }
        let program = Program::compile(template).map_err(|e| e.to_string())?;
        let mut context = Context::default();
        
        // Register custom functions
        context.add_function("json", json_template_func);

        if let Some(obj) = data.as_object() {
            for (k, v) in obj {
                let _ = context.add_variable(k, json_to_cel(v.clone()));
            }
        }

        let result = program.execute(&context).map_err(|e| e.to_string())?;
        
        match result {
            CelValue::String(s) => Ok(s.to_string()),
            _ => {
                let json_val = cel_to_json(result);
                if json_val.is_string() {
                    Ok(json_val.as_str().unwrap().to_string())
                } else {
                    Ok(json_val.to_string())
                }
            }
        }
    }
}
