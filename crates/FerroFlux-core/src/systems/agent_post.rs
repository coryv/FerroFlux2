use crate::components::pipeline::ExecutionResult;
use crate::components::{Outbox, WorkDone};
use crate::store::BlobStore;
use bevy_ecs::prelude::*;
use serde_json::{Value, json};

#[tracing::instrument(skip(commands, query, store, work_done, event_bus, outbox_query))]
pub fn agent_post(
    mut commands: Commands,
    query: Query<(Entity, &ExecutionResult)>,
    store: Res<BlobStore>,
    mut work_done: ResMut<WorkDone>,
    event_bus: Res<crate::api::events::SystemEventBus>,
    mut outbox_query: Query<&mut Outbox>,
) {
    for (entity, result) in query.iter() {
        work_done.0 = true;

        let mut success = false;
        let final_output_str;

        if result.status >= 200 && result.status < 300 {
            if let Ok(_json_data) = serde_json::from_str::<Value>(&result.raw_body) {
                if let Some(transform_text) = &result.context.output_transform {
                    match jmespath::compile(transform_text) {
                        Ok(expr) => match jmespath::Variable::from_json(&result.raw_body) {
                            Ok(data) => match expr.search(&data) {
                                Ok(res) => {
                                    if let Some(s) = res.as_string() {
                                        final_output_str = s.to_string();
                                        success = true;
                                    } else {
                                        final_output_str =
                                            serde_json::to_string(&res).unwrap_or_default();
                                        success = true;
                                    }
                                }
                                Err(e) => {
                                    final_output_str = format!("JMESPath Search Error: {}", e)
                                }
                            },
                            Err(e) => final_output_str = format!("JMESPath Parse Error: {}", e),
                        },
                        Err(e) => final_output_str = format!("JMESPath Compile Error: {}", e),
                    }
                } else {
                    final_output_str = result.raw_body.clone();
                    success = true;
                }
            } else {
                final_output_str = result.raw_body.clone();
                success = true;
            }
        } else {
            final_output_str = format!("HTTP Error {}: {}", result.status, result.raw_body);
        }

        // Merge Result
        let output = crate::systems::utils::merge_result(
            &result.context.input_json,
            &final_output_str,
            result.context.result_key.as_ref(),
        );

        // Telemetry
        let elapsed = (chrono::Utc::now().timestamp_millis() as u64)
            .saturating_sub(result.context.start_time);
        let _ = event_bus
            .0
            .send(crate::api::events::SystemEvent::NodeTelemetry {
                trace_id: result.trace_id.clone(),
                node_id: result.context.node_id,
                node_type: "Agent".to_string(),
                execution_ms: elapsed,
                success,
                details: json!({
                    "provider": result.context.provider_name,
                    "model": result.context.model_name,
                    "status": result.status,
                }),
            });

        // Store result and push to Outbox
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("trace_id".to_string(), result.trace_id.clone());

        if let Ok(ticket) = store.check_in_with_metadata(output.as_bytes(), metadata)
            && let Ok(mut outbox) = outbox_query.get_mut(entity) {
                outbox.queue.push_back(ticket);
                tracing::info!(
                    node_id = %result.context.node_id,
                    trace_id = %result.trace_id,
                    elapsed_ms = elapsed,
                    success = success,
                    "Agent execution finished"
                );
            }

        // Cleanup
        commands.entity(entity).remove::<ExecutionResult>();
    }
}
