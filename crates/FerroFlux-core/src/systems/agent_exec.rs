use crate::components::pipeline::{ExecutionResult, ReadyToExecute};
use crate::resources::{GlobalHttpClient, PipelineResultChannel, WorkDone};
use bevy_ecs::prelude::*;

#[tracing::instrument(skip(commands, query, http_client, runtime, channel, work_done))]
pub fn agent_exec(
    mut commands: Commands,
    query: Query<(Entity, &ReadyToExecute), Without<ExecutionResult>>,
    http_client: Res<GlobalHttpClient>,
    runtime: Res<crate::resources::TokioRuntime>,
    channel: Res<PipelineResultChannel>,
    mut work_done: ResMut<WorkDone>,
) {
    let (tx, rx) = (&channel.tx, &channel.rx);

    // 1. Poll completed tasks
    while let Ok((entity, result)) = rx.try_recv() {
        commands.entity(entity).insert(result);
        work_done.0 = true;
    }

    // 2. Spawn new tasks
    for (entity, ready) in query.iter() {
        let client = http_client.client.clone();
        let tx_clone = tx.clone();
        let entity_id = entity;
        let ready_clone = ready.clone();

        commands.entity(entity).remove::<ReadyToExecute>();
        work_done.0 = true;

        runtime.0.spawn(async move {
            let span = tracing::info_span!("agent_request", node_id = %ready_clone.context.node_id, trace_id = %ready_clone.trace_id);
            let _enter = span.enter();

            let method = match ready_clone.method.as_str() {
                "GET" => reqwest::Method::GET,
                "POST" => reqwest::Method::POST,
                "PUT" => reqwest::Method::PUT,
                "DELETE" => reqwest::Method::DELETE,
                _ => reqwest::Method::POST,
            };

            tracing::debug!(method = %method, url = %ready_clone.url, "Sending HTTP request");

            let mut request_builder = client.request(method, &ready_clone.url);

            for (k, v) in &ready_clone.headers {
                request_builder = request_builder.header(k, v);
            }

            request_builder = request_builder.body(ready_clone.body.clone());

            let response_result = request_builder.send().await;

            let (status, raw_body) = match response_result {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    tracing::debug!(status = %status, "Received HTTP response");
                    (status, resp.text().await.unwrap_or_default())
                }
                Err(e) => {
                    tracing::error!(error = %e, "HTTP request failed");
                    (500, format!("Request Failed: {}", e))
                }
            };

            let result = ExecutionResult {
                status,
                raw_body,
                trace_id: ready_clone.trace_id,
                context: ready_clone.context,
            };

            let _ = tx_clone.send((entity_id, result)).await;
        });
    }
}
