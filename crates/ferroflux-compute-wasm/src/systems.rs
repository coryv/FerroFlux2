use crate::components::ComputeConfig;
use bevy_ecs::prelude::*;
use ferroflux_types::{
    merge_result, BlobStore, Inbox, NodeConfig, Outbox, SystemEvent, SystemEventBus,
};
use serde_json::json;
use std::sync::Arc;
use wasmtime::*;

#[derive(Resource, Clone)]
pub struct WasmRuntime {
    pub engine: Engine,
}

impl Default for WasmRuntime {
    fn default() -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(false);

        let engine = Engine::new(&config).expect("failed to create wasmtime engine");
        Self { engine }
    }
}

/// System: WASM Compute Worker
#[tracing::instrument(skip(query, store, runtime, event_bus))]
pub fn wasm_worker(
    mut query: Query<(&ComputeConfig, &NodeConfig, &mut Inbox, &mut Outbox)>,
    store: Res<BlobStore>,
    runtime: Res<WasmRuntime>,
    event_bus: Res<SystemEventBus>,
) {
    let event_tx = event_bus.0.clone();

    for (config, node_config, mut inbox, mut outbox) in query.iter_mut() {
        while let Some(ticket) = inbox.queue.pop_front() {
            let start = std::time::Instant::now();
            let trace_id = ticket
                .metadata
                .get("trace_id")
                .cloned()
                .unwrap_or("unknown".into());

            let payload_bytes = match store.claim(&ticket) {
                Ok(b) => b,
                Err(_) => continue,
            };

            let input_val: serde_json::Value =
                serde_json::from_slice(&payload_bytes).unwrap_or(serde_json::Value::Null);

            // In a real implementation, we'd compile the WASM or use a pre-compiled component
            // For now, this is a mock implementation of the WASM execution flow.
            let script_result = if config.runtime == "js-quickjs" {
                // Mock JS execution
                format!("{{\"processed\": true, \"original_len\": {}}}", payload_bytes.len())
            } else {
                "{\"error\": \"Unsupported runtime\"}".to_string()
            };

            let final_json = merge_result(&input_val, &script_result, None);

            if let Ok(bytes) = serde_json::to_vec(
                &serde_json::from_str::<serde_json::Value>(&final_json).unwrap(),
            ) && let Ok(mut new_ticket) = store.check_in(&bytes)
            {
                new_ticket.metadata = ticket.metadata.clone();
                outbox.queue.push_back((None, new_ticket));
            }

            let _ = event_tx.send(SystemEvent::NodeTelemetry {
                node_id: node_config.id,
                node_type: "Compute".into(),
                trace_id: trace_id.clone(),
                execution_ms: start.elapsed().as_millis() as u64,
                success: true,
                details: json!({"message": "WASM Executed"}),
            });
        }
    }
}
