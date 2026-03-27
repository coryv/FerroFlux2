use crate::blob::SecureTicket;
use bevy_ecs::prelude::*;
use uuid::Uuid;

/// A trigger event that initiates a workflow or node execution.
///
/// Trigger events are enqueued by external sources (webhooks, cron, SSE, Kafka)
/// and dispatched into the ECS world via the `TriggerReceiver` resource.
#[derive(Debug, Clone)]
pub struct TriggerEvent {
    /// Usually the Node ID of the trigger node that owns this event.
    pub trigger_id: Uuid,
    /// The payload delivered to the triggered workflow.
    pub payload: SecureTicket,
}

/// Resource for sending trigger events into the system.
///
/// Used by the API layer, webhook bridges, SSE receivers, and custom providers.
#[derive(Resource, Clone)]
pub struct TriggerSender(pub async_channel::Sender<TriggerEvent>);

/// Resource for receiving trigger events within the ECS loop.
#[derive(Resource)]
pub struct TriggerReceiver(pub async_channel::Receiver<TriggerEvent>);

/// Trait for implementing custom trigger providers (e.g., Kafka, SQS, NATS).
///
/// Providers are registered with `AppBuilder::with_trigger_provider()` and
/// initialized at startup. They push `TriggerEvent`s via the `TriggerSender`.
pub trait TriggerProvider: Send + Sync + 'static {
    /// Returns the unique name of this provider.
    fn name(&self) -> &str;

    /// Called at app startup; the provider should begin emitting events.
    fn on_enable(&self, sender: TriggerSender);

    /// Called at shutdown (best effort — may not always be invoked).
    fn on_disable(&self) {}
}
