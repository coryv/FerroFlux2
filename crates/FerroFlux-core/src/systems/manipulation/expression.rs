use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::components::manipulation::ExpressionConfig;
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use evalexpr::{ContextWithMutableFunctions, ContextWithMutableVariables};
use serde_json::json;
use std::time::Instant;

#[tracing::instrument(skip(query, store, event_bus))]
pub fn expression_worker(
    mut query: Query<(&ExpressionConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
    store: Res<BlobStore>,
    event_bus: Res<SystemEventBus>,
) {
    let event_tx = event_bus.0.clone();

    for (config, node_config, mut inbox, mut outbox) in query.iter_mut() {
        while let Some(ticket) = inbox.queue.pop_front() {
            let start = Instant::now();
            let trace_id = ticket
                .metadata
                .get("trace_id")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let payload_bytes = match store.claim(&ticket) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };

            let mut json_val: serde_json::Value =
                serde_json::from_slice(&payload_bytes).unwrap_or(serde_json::Value::Null);

            let mut context = evalexpr::HashMapContext::new();
            let mut bound_count = 0;

            // Bind top-level numeric fields
            if let Some(obj) = json_val.as_object() {
                for (k, v) in obj {
                    if let Some(num) = v.as_f64() {
                        // evalexpr expects Value
                        let _ = context.set_value(k.into(), num.into());
                        bound_count += 1;
                    }
                }
            }

            // Bind useful math constants/functions
            // evalexpr default context doesn't include functions when using eval_with_context on an empty HashMapContext
            let _ = context.set_function(
                "sqrt".into(),
                evalexpr::Function::new(|argument| {
                    let num = argument.as_float()?;
                    Ok(evalexpr::Value::Float(num.sqrt()))
                }),
            );
            let _ = context.set_function(
                "abs".into(),
                evalexpr::Function::new(|argument| {
                    let num = argument.as_float()?;
                    Ok(evalexpr::Value::Float(num.abs()))
                }),
            );
            let _ = context.set_function(
                "floor".into(),
                evalexpr::Function::new(|argument| {
                    let num = argument.as_float()?;
                    Ok(evalexpr::Value::Float(num.floor()))
                }),
            );
            let _ = context.set_function(
                "ceil".into(),
                evalexpr::Function::new(|argument| {
                    let num = argument.as_float()?;
                    Ok(evalexpr::Value::Float(num.ceil()))
                }),
            );
            let _ = context.set_function(
                "max".into(),
                evalexpr::Function::new(|argument| {
                    let args = argument.as_tuple()?;
                    let a = args[0].as_float()?;
                    let b = args[1].as_float()?;
                    Ok(evalexpr::Value::Float(a.max(b)))
                }),
            );
            let _ = context.set_function(
                "min".into(),
                evalexpr::Function::new(|argument| {
                    let args = argument.as_tuple()?;
                    let a = args[0].as_float()?;
                    let b = args[1].as_float()?;
                    Ok(evalexpr::Value::Float(a.min(b)))
                }),
            );

            let result = match evalexpr::eval_with_context(&config.expression, &context) {
                Ok(val) => {
                    // Extract float from Value
                    let float_val = match val {
                        evalexpr::Value::Float(f) => f,
                        evalexpr::Value::Int(i) => i as f64,
                        _ => 0.0, // Or handle error for non-numeric return
                    };

                    // Enrich
                    if let Some(obj) = json_val.as_object_mut() {
                        obj.insert(config.result_key.clone(), serde_json::json!(float_val));
                    }
                    Some(float_val)
                }
                Err(e) => {
                    tracing::error!(node_id = %node_config.id, expression = %config.expression, error = %e, "Expression evaluation error");
                    None
                }
            };

            if let Ok(bytes) = serde_json::to_vec(&json_val)
                && let Ok(mut new_ticket) = store.check_in(&bytes)
            {
                new_ticket.metadata = ticket.metadata;
                outbox.queue.push_back((None, new_ticket));
            }

            let success = result.is_some();
            let details = if let Some(val) = result {
                json!({ "result": val, "bound_vars": bound_count })
            } else {
                json!({ "error": "Evaluation Failed" })
            };

            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Expression".to_string(),
                trace_id,
                execution_ms: start.elapsed().as_millis() as u64,
                success,
                details,
            });
        }
    }
}
