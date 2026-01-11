use crate::store::SecureTicket;
use bevy_ecs::prelude::*;
use uuid::Uuid;

/// Represents a trigger event that initiates a workflow or node execution.
/// Ideally, this unifies Webhooks, Cron, Kafka, and other event sources.
#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub trigger_id: Uuid, // Usually the Node ID targeting the trigger
    pub payload: SecureTicket,
}

/// Resource for sending trigger events into the system.
/// This can be used by the API layer, Webhook bridges, or other input adapters.
#[derive(Resource, Clone)]
pub struct TriggerSender(pub async_channel::Sender<TriggerEvent>);

/// Resource for receiving trigger events within the ECS loop.
#[derive(Resource)]
pub struct TriggerReceiver(pub async_channel::Receiver<TriggerEvent>);

/// Trait for implementing custom trigger providers (e.g., Kafka, Cron, SQS).
/// Providers are initialized at app startup and can push events to the `TriggerSender`.
pub trait TriggerProvider: Send + Sync + 'static {
    /// Returns the unique name of the provider.
    fn name(&self) -> &str;

    /// Called when the app starts, providing the sender channel to push events to.
    fn on_enable(&self, sender: TriggerSender);

    /// Called when the app shuts down (best effort).
    fn on_disable(&self) {}
}
