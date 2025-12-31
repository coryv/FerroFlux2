use crate::components::pipeline::PipelineNode;
use crate::resources::registry::DefinitionRegistry;
use crate::tools::ToolContext;
use crate::tools::registry::ToolRegistry;
use anyhow::Result;
use bevy_ecs::prelude::*;
use handlebars::Handlebars;
use serde_json::Value;
use std::collections::HashMap;

/// System that executes PipelineNodes when triggered.
///
/// NOTE: In a real implementation, this would likely be an async system or spawned task.
/// For this MVP, we execute synchronously when an "Exec" signal is received (implied).
pub fn pipeline_execution_system(
    mut _query: Query<(Entity, &mut PipelineNode)>,
    _node_registry: Res<DefinitionRegistry>,
    _tool_registry: Res<ToolRegistry>,
) {
    // In a real FerroFlux engine, we'd check for an "Active" state or Input Event.
    // Here we iterate all for demonstration, but logic usually requires a trigger.

    // For the sake of this architectural MVP, we define the `execute_node` function
    // which would be called by the graph runner.
}

/// Helper function to execute a single pipeline node.
/// This would be called by the main graph traversal loop.
pub fn execute_pipeline_node(
    node: &mut PipelineNode,
    inputs: HashMap<String, Value>,
    definitions: &DefinitionRegistry,
    tools: &ToolRegistry,
    global_memory: &mut HashMap<String, Value>,
) -> Result<HashMap<String, Value>> {
    let def = definitions
        .definitions
        .get(&node.definition_id)
        .ok_or_else(|| anyhow::anyhow!("Definition not found: {}", node.definition_id))?;

    // 1. Initialize Context
    // Merge Inputs + Config into "execution_context"
    // In spec: context keys map to specific inputs/settings via handlebars or direct mapping.
    // For MVP: We dump inputs into `inputs` key and config into `settings` key.

    let mut ctx_map = HashMap::new();
    ctx_map.insert("inputs".to_string(), serde_json::to_value(&inputs)?);
    ctx_map.insert("settings".to_string(), serde_json::to_value(&node.config)?);

    // Inject Platform Config
    if let Some(platform_id) = &def.meta.platform {
        if let Some(platform) = definitions.platforms.get(platform_id) {
            ctx_map.insert(
                "platform".to_string(),
                serde_json::to_value(&platform.config)?,
            );
        } else {
            // Warn? Or Fail? For now, just log and continue (might use default)
            eprintln!("WARN: Platform definition not found: {}", platform_id);
        }
    }

    // Also support the explicit `context` variables from definition
    let handlebars = Handlebars::new();
    if let Some(ctx_defs) = &def.context {
        for (key, template) in ctx_defs {
            let rendered = handlebars
                .render_template(template, &ctx_map)
                .unwrap_or_else(|_| template.clone());
            // Attempt to parse as JSON if possible, else string
            let val = serde_json::from_str(&rendered).unwrap_or(Value::String(rendered));
            ctx_map.insert(key.clone(), val);
        }
    }

    // "steps" namespace for step outputs
    ctx_map.insert("steps".to_string(), serde_json::json!({}));

    // 2. Execute Steps
    for step in &def.execution {
        let tool = tools
            .get(&step.tool)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", step.tool))?;

        // Resolve Params (Templating)
        // Resolve Params (Templating) & Type Preservation
        let resolved_params = resolve_recursive(&step.params, &ctx_map, &handlebars)?;

        // Run Tool
        let mut tool_ctx = ToolContext {
            local: &mut ctx_map,
            memory: global_memory,
        };

        let result = tool.run(&mut tool_ctx, resolved_params)?;

        // Map Returns
        // "returns": { "status": "status_code" } -> context["status_code"] = result["status"]
        if let Some(steps_obj) = ctx_map.get_mut("steps").and_then(|v| v.as_object_mut()) {
            // Store raw result under step ID for easy access: steps.my_step.status
            steps_obj.insert(step.id.clone(), result.clone());
        }

        // Returns Mapping
        for (key, var_name) in &step.returns {
            if let Value::Object(res_obj) = &result {
                if let Some(val) = res_obj.get(key) {
                    ctx_map.insert(var_name.clone(), val.clone());
                }
            }
        }
    }

    // 3. Routing (Optional)
    if let Some(routing) = &def.routing {
        // Evaluate Match Expression
        let match_expr_str = &routing.match_expr;
        let resolved_match = handlebars.render_template(match_expr_str, &ctx_map)?;

        // Find matching case
        if let Some(actions) = routing.cases.get(&resolved_match) {
            for action in actions {
                let tool = tools
                    .get(&action.tool)
                    .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", action.tool))?;

                let resolved_params = resolve_recursive(&action.params, &ctx_map, &handlebars)?;

                let mut tool_ctx = ToolContext {
                    local: &mut ctx_map,
                    memory: global_memory,
                };
                let result = tool.run(&mut tool_ctx, resolved_params)?;

                // Returns Mapping (Routing)
                for (key, var_name) in &action.returns {
                    if let Value::Object(res_obj) = &result {
                        if let Some(val) = res_obj.get(key) {
                            ctx_map.insert(var_name.clone(), val.clone());
                        }
                    }
                }
            }
        }
    }

    // 4. Collect Outputs (Emit tool writes to _outputs)
    let outputs = ctx_map
        .get("_outputs")
        .cloned()
        .unwrap_or(serde_json::json!({}));
    let output_map: HashMap<String, Value> = serde_json::from_value(outputs).unwrap_or_default();

    Ok(output_map)
}

fn resolve_recursive(
    value: &Value,
    ctx: &HashMap<String, Value>,
    reg: &Handlebars,
) -> Result<Value> {
    match value {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
                let inner = &trimmed[2..trimmed.len() - 2].trim();
                // If simple variable ref without helpers/logic
                if !inner.contains(' ') {
                    if let Some(val) = lookup_path(ctx, inner) {
                        return Ok(val.clone());
                    }
                }
            }
            let rendered = reg.render_template(s, ctx)?;
            Ok(Value::String(rendered))
        }
        Value::Array(arr) => {
            let mut new_arr = Vec::new();
            for v in arr {
                new_arr.push(resolve_recursive(v, ctx, reg)?);
            }
            Ok(Value::Array(new_arr))
        }
        Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                new_obj.insert(k.clone(), resolve_recursive(v, ctx, reg)?);
            }
            Ok(Value::Object(new_obj))
        }
        _ => Ok(value.clone()),
    }
}

fn lookup_path<'a>(ctx: &'a HashMap<String, Value>, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return None;
    }

    let mut current = ctx.get(parts[0])?;
    for part in &parts[1..] {
        match current {
            Value::Object(map) => {
                current = map.get(*part)?;
            }
            Value::Array(arr) => {
                if let Ok(idx) = part.parse::<usize>() {
                    current = arr.get(idx)?;
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(current)
}
