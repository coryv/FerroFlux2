use crate::api::events::{SystemEvent, SystemEventBus};
use crate::components::core::{Inbox, NodeConfig, Outbox};
use crate::components::manipulation::{AggregateConfig, BatchState, SplitConfig, TransformConfig};
use crate::store::BlobStore;
use crate::systems::utils::merge_result;
use bevy_ecs::prelude::*;
use evalexpr::{ContextWithMutableFunctions, ContextWithMutableVariables}; // Trait for set_value & set_function
use jmespath;
use serde_json::json;
use std::time::{Duration, Instant};

/// System: Splitter Worker (Fan-Out)
///
/// Takes a single Ticket containing an Array and produces N Tickets.
/// - Uses JMESPath to locate the array within the payload.
/// - **Zero-Copy Optimization**: While we deserialize the JSON to split it,
///   outgoing tickets get their own fresh entries in BlobStore to ensure independent ownership.
#[tracing::instrument(skip(query, store, event_bus))]
pub fn splitter_worker(
    mut query: Query<(&SplitConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
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
                Err(e) => {
                    tracing::error!(node_id = %node_config.id, error = %e, "Failed to claim ticket for splitter");
                    continue;
                }
            };

            let input_val: serde_json::Value = match serde_json::from_slice(&payload_bytes) {
                Ok(v) => v,
                Err(_) => serde_json::Value::Null,
            };

            // Determine array to split
            let path = config.path.as_deref().unwrap_or("@"); // @ is current node (root) in JMESPath
            let expr = match jmespath::compile(path) {
                Ok(e) => e,
                Err(e) => {
                    tracing::error!(node_id = %node_config.id, path = %path, error = %e, "Invalid JMESPath in splitter");
                    continue;
                }
            };

            // FIX: Using std::rc::Rc because jmespath::Expression::search returns Rc<Variable>
            let search_result = expr
                .search(&input_val)
                .unwrap_or_else(|_| std::rc::Rc::new(jmespath::Variable::Null));

            let array_val = serde_json::to_value(search_result).unwrap_or(serde_json::Value::Null);

            let array = if array_val.is_array() {
                array_val.as_array().unwrap().clone()
            } else if array_val.is_null() {
                vec![]
            } else {
                vec![array_val.clone()]
            };

            let count = array.len();

            for item in array {
                if let Ok(bytes) = serde_json::to_vec(&item)
                    && let Ok(new_ticket) = store.check_in(&bytes)
                {
                    let mut final_ticket = new_ticket;
                    final_ticket.metadata = ticket.metadata.clone();
                    outbox.queue.push_back((None, final_ticket));
                }
            }

            // Emit Telemetry
            // FIX: Added node_type, converted details to Value
            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Split".to_string(),
                trace_id: trace_id.clone(),
                execution_ms: start.elapsed().as_millis() as u64,
                success: true,
                details: json!({ "message": format!("Split into {} items", count) }),
            });
        }
    }
}

/// System: Aggregator Worker (Fan-In / Batching)
///
/// Buffers incoming items until a condition is met (Size or Time).
/// - **Stateful**: Uses `BatchState` buffer.
/// - Useful for creating batches for `StatsNode` or reducing API calls.
#[tracing::instrument(skip(query, store, event_bus))]
pub fn aggregator_worker(
    mut query: Query<(
        &AggregateConfig,
        &NodeConfig,
        &mut BatchState,
        &mut Inbox,
        &mut Outbox,
    )>,
    store: Res<BlobStore>,
    event_bus: Res<SystemEventBus>,
) {
    let event_tx = event_bus.0.clone();

    for (config, node_config, mut state, mut inbox, mut outbox) in query.iter_mut() {
        // 1. Ingest
        while let Some(ticket) = inbox.queue.pop_front() {
            if state.items.is_empty() {
                state.last_update = Some(Instant::now());
            }

            if let Ok(bytes) = store.claim(&ticket)
                && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes)
            {
                state.items.push(val);
            }
        }

        // 2. Check Conditions
        let count = state.items.len();
        let elapsed = state
            .last_update
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO);

        let batch_full = count >= config.batch_size && count > 0;
        let timed_out = elapsed >= Duration::from_secs(config.timeout_seconds) && count > 0;

        if batch_full || timed_out {
            let batch_json = serde_json::Value::Array(state.items.clone());

            if let Ok(bytes) = serde_json::to_vec(&batch_json)
                && let Ok(mut ticket) = store.check_in(&bytes)
            {
                ticket
                    .metadata
                    .insert("trace_id".to_string(), uuid::Uuid::new_v4().to_string());
                outbox.queue.push_back((None, ticket));
            }

            // Emit Telemetry
            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Aggregate".to_string(),
                trace_id: "batch_event".to_string(),
                execution_ms: 0,
                success: true,
                details: json!({ "message": format!("Aggregated {} items", count) }),
            });

            // Reset
            state.items.clear();
            state.last_update = None;
        }
    }
}

#[tracing::instrument(skip(query, store, event_bus))]
pub fn transform_worker(
    mut query: Query<(&TransformConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
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

            let input_val: serde_json::Value =
                serde_json::from_slice(&payload_bytes).unwrap_or(serde_json::Value::Null);

            let expr = match jmespath::compile(&config.expression) {
                Ok(e) => e,
                Err(e) => {
                    tracing::error!(node_id = %node_config.id, error = %e, "Invalid expression in transform");
                    continue;
                }
            };

            // FIX: Using Rc
            let search_result = expr
                .search(&input_val)
                .unwrap_or_else(|_| std::rc::Rc::new(jmespath::Variable::Null));
            let result_json =
                serde_json::to_value(search_result).unwrap_or(serde_json::Value::Null);
            let result_str = result_json.to_string();

            // FIX: merge_result args
            let final_output = merge_result(&input_val, &result_str, config.result_key.as_ref());

            // Re-parse to validate/store as JSON bytes
            let final_output_bytes = final_output.into_bytes();

            if let Ok(mut new_ticket) = store.check_in(&final_output_bytes) {
                new_ticket.metadata = ticket.metadata.clone();
                outbox.queue.push_back((None, new_ticket));
            }

            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Transform".to_string(),
                trace_id,
                execution_ms: start.elapsed().as_millis() as u64,
                success: true,
                details: json!({ "message": "Transformation complete" }),
            });
        }
    }
}
/// System: Statistics Worker (Aggregation Analysis)
///
/// ## Architecture: Vector-First Processing
/// Instead of updating statistics incrementally per item (which is complex for Z-Score/StdDev),
/// this system processes entire batches (Tickets containing Arrays).
/// 1. **Pass 1**: Iterate array to calculate Mean and Variance.
/// 2. **Pass 2**: Iterate array again to calculate Z-Score and flag Outliers.
/// This ensures global context for the batch is available.
#[tracing::instrument(skip(query, store, event_bus))]
pub fn stats_worker(
    mut query: Query<(
        &crate::components::manipulation::StatsConfig,
        &NodeConfig,
        &mut Inbox,
        &mut Outbox,
    )>,
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

            let input_val: serde_json::Value =
                serde_json::from_slice(&payload_bytes).unwrap_or(serde_json::Value::Null);

            // Ensure input is Array
            let array = match input_val.as_array() {
                Some(arr) => arr,
                None => {
                    tracing::warn!(node_id = %node_config.id, "Input to stats is not an array");
                    continue; // Skip or Error
                }
            };

            // Pass 1: Collect Values & Calculate Stats
            let mut values: Vec<f64> = Vec::new();
            for item in array {
                if let Some(val) = item.get(&config.target_field).and_then(|v| v.as_f64()) {
                    values.push(val);
                }
            }

            let count = values.len() as f64;
            if count == 0.0 {
                // No values to analyze, pass through or error?
                // Let's pass through unmodified for safety, maybe logging warning.
                if let Ok(new_ticket) = store.check_in(&payload_bytes) {
                    let mut final_ticket = new_ticket;
                    final_ticket.metadata = ticket.metadata;
                    outbox.queue.push_back((None, final_ticket));
                }
                continue;
            }

            let sum: f64 = values.iter().sum();
            let mean = sum / count;

            let variance: f64 = values
                .iter()
                .map(|v| {
                    let diff = mean - *v;
                    diff * diff
                })
                .sum::<f64>()
                / count;
            let std_dev = variance.sqrt();

            // Pass 2: Enrich
            let mut enriched_array = array.clone();
            let mut outlier_count = 0;

            // Correct Pass 2: Iterate array, if field exists, calc Z-Score
            for item in enriched_array.iter_mut() {
                if let Some(obj) = item.as_object_mut()
                    && let Some(val) = obj.get(&config.target_field).and_then(|v| v.as_f64())
                {
                    let z_score = if std_dev == 0.0 {
                        0.0
                    } else {
                        (val - mean) / std_dev
                    };
                    let is_outlier = config.detect_outliers && z_score.abs() > config.threshold;

                    if is_outlier {
                        outlier_count += 1;
                    }

                    let stats_obj = json!({
                        "mean": mean,
                        "std_dev": std_dev,
                        "z_score": z_score,
                        "is_outlier": is_outlier
                    });

                    obj.insert(config.enrichment_key.clone(), stats_obj);
                }
            }

            let final_json = serde_json::Value::Array(enriched_array);
            let final_bytes = serde_json::to_vec(&final_json).unwrap(); // Should safeguard

            if let Ok(mut new_ticket) = store.check_in(&final_bytes) {
                new_ticket.metadata = ticket.metadata.clone();
                outbox.queue.push_back((None, new_ticket));
            }

            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Stats".to_string(),
                trace_id,
                execution_ms: start.elapsed().as_millis() as u64,
                success: true,
                details: json!({
                    "count": count,
                    "mean": mean,
                    "std_dev": std_dev,
                    "outliers": outlier_count
                }),
            });
        }
    }
}

/// System: Window Worker (Rolling Analysis)
///
/// ## Architecture: Stateful Nodes
/// Unlike most nodes which are stateless (f(input) -> output), this node maintains `WindowState`.
/// - `WindowState` contains a generic `VecDeque` buffer.
/// - State persists across ticks in the ECS `Query`.
/// - Allows calculating rolling averages/sums over a stream of individual events.
#[tracing::instrument(skip(query, store, event_bus))]
pub fn window_worker(
    mut query: Query<(
        &crate::components::manipulation::WindowConfig,
        &NodeConfig,
        &mut crate::components::manipulation::WindowState,
        &mut Inbox,
        &mut Outbox,
    )>,
    store: Res<BlobStore>,
    event_bus: Res<SystemEventBus>,
) {
    let event_tx = event_bus.0.clone();

    for (config, node_config, mut state, mut inbox, mut outbox) in query.iter_mut() {
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

            // 1. Extract Value
            let val = if let Some(obj) = json_val.as_object() {
                obj.get(&config.target_field).and_then(|v| v.as_f64())
            } else {
                None
            };

            // 2. Update Window State
            if let Some(v) = val {
                state.buffer.push_back(v);
                if state.buffer.len() > config.window_size {
                    state.buffer.pop_front();
                }
            }

            // 3. Calculate
            let count = state.buffer.len() as f64;
            let result = if count == 0.0 {
                0.0
            } else {
                match config.operation {
                    crate::components::manipulation::WindowOp::Mean => {
                        let sum: f64 = state.buffer.iter().sum();
                        sum / count
                    }
                    crate::components::manipulation::WindowOp::Sum => state.buffer.iter().sum(),
                    crate::components::manipulation::WindowOp::Min => {
                        state.buffer.iter().cloned().fold(f64::INFINITY, f64::min)
                    }
                    crate::components::manipulation::WindowOp::Max => state
                        .buffer
                        .iter()
                        .cloned()
                        .fold(f64::NEG_INFINITY, f64::max),
                    crate::components::manipulation::WindowOp::Variance => {
                        let sum: f64 = state.buffer.iter().sum();
                        let mean = sum / count;
                        state
                            .buffer
                            .iter()
                            .map(|v| {
                                let diff = mean - *v;
                                diff * diff
                            })
                            .sum::<f64>()
                            / count
                    }
                }
            };

            // 4. Enrich
            if let Some(obj) = json_val.as_object_mut() {
                obj.insert(config.result_key.clone(), serde_json::json!(result));
            }

            // 5. Output
            if let Ok(bytes) = serde_json::to_vec(&json_val)
                && let Ok(mut new_ticket) = store.check_in(&bytes)
            {
                new_ticket.metadata = ticket.metadata;
                outbox.queue.push_back((None, new_ticket));
            }

            // Telemetry
            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Window".to_string(),
                trace_id,
                execution_ms: start.elapsed().as_millis() as u64,
                success: true,
                details: json!({
                    "window_size": state.buffer.len(),
                    "result": result
                }),
            });
        }
    }
}

#[tracing::instrument(skip(query, store, event_bus))]
pub fn expression_worker(
    mut query: Query<(
        &crate::components::manipulation::ExpressionConfig,
        &NodeConfig,
        &mut Inbox,
        &mut Outbox,
    )>,
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
