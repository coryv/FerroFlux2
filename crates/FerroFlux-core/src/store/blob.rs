use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecureTicket {
    pub id: Uuid,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
struct BlobEntry {
    data: Vec<u8>,
    metadata: HashMap<String, String>,
    created_at: std::time::Instant,
}

#[derive(Clone, Debug, Resource)]
pub struct BlobStore {
    // Simple in-memory store for now.
    // Map uuid -> BlobEntry
    storage: Arc<RwLock<HashMap<Uuid, BlobEntry>>>,
}

impl Default for BlobStore {
    fn default() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl BlobStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_in(&self, data: &[u8]) -> anyhow::Result<SecureTicket> {
        self.check_in_with_metadata(data, HashMap::new())
    }

    pub fn check_in_with_metadata(
        &self,
        data: &[u8],
        metadata: HashMap<String, String>,
    ) -> anyhow::Result<SecureTicket> {
        let id = Uuid::new_v4();
        let ticket = SecureTicket {
            id,
            metadata: metadata.clone(),
        };

        self.storage.write().unwrap().insert(
            id,
            BlobEntry {
                data: data.to_vec(),
                metadata,
                created_at: std::time::Instant::now(),
            },
        );
        Ok(ticket)
    }

    pub fn claim(&self, ticket: &SecureTicket) -> anyhow::Result<Vec<u8>> {
        if let Some(entry) = self.storage.read().unwrap().get(&ticket.id) {
            Ok(entry.data.clone())
        } else {
            Err(anyhow::anyhow!("Ticket not found"))
        }
    }

    pub fn recover_ticket(&self, id: &Uuid) -> Option<SecureTicket> {
        let guard = self.storage.read().unwrap();
        guard.get(id).map(|entry| SecureTicket {
            id: *id,
            metadata: entry.metadata.clone(),
        })
    }

    pub fn update_metadata(
        &self,
        id: &Uuid,
        new_metadata: HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let mut guard = self.storage.write().unwrap();
        if let Some(entry) = guard.get_mut(id) {
            entry.metadata.extend(new_metadata);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Ticket not found"))
        }
    }

    pub fn run_garbage_collection(&self) -> usize {
        let mut guard = self.storage.write().unwrap();
        let now = std::time::Instant::now();
        let ttl = std::time::Duration::from_secs(60 * 15); // 15 Minutes TTL
        let initial_len = guard.len();

        guard.retain(|_, entry| now.duration_since(entry.created_at) < ttl);

        initial_len - guard.len()
    }
}
