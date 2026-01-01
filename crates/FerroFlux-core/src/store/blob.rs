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

/// Trait for pluggable blob storage backends.
pub trait BlobProvider: Send + Sync + std::fmt::Debug {
    fn store(&self, id: Uuid, data: Vec<u8>, metadata: HashMap<String, String>) -> anyhow::Result<()>;
    fn retrieve(&self, id: &Uuid) -> anyhow::Result<Option<(Vec<u8>, HashMap<String, String>)>>;
    fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;
    fn update_metadata(&self, id: &Uuid, metadata: HashMap<String, String>) -> anyhow::Result<()>;
    fn list_expired(&self, ttl: std::time::Duration) -> Vec<Uuid>;
}

/// In-memory implementation of BlobProvider.
#[derive(Debug, Default)]
pub struct MemoryProvider {
    storage: RwLock<HashMap<Uuid, BlobEntry>>,
}

impl BlobProvider for MemoryProvider {
    fn store(&self, id: Uuid, data: Vec<u8>, metadata: HashMap<String, String>) -> anyhow::Result<()> {
        let mut guard = self.storage.write().unwrap();
        guard.insert(
            id,
            BlobEntry {
                data,
                metadata,
                created_at: std::time::Instant::now(),
            },
        );
        Ok(())
    }

    fn retrieve(&self, id: &Uuid) -> anyhow::Result<Option<(Vec<u8>, HashMap<String, String>)>> {
        let guard = self.storage.read().unwrap();
        Ok(guard.get(id).map(|e| (e.data.clone(), e.metadata.clone())))
    }

    fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let mut guard = self.storage.write().unwrap();
        Ok(guard.remove(id).is_some())
    }

    fn update_metadata(&self, id: &Uuid, metadata: HashMap<String, String>) -> anyhow::Result<()> {
        let mut guard = self.storage.write().unwrap();
        if let Some(entry) = guard.get_mut(id) {
            entry.metadata.extend(metadata);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Ticket not found"))
        }
    }

    fn list_expired(&self, ttl: std::time::Duration) -> Vec<Uuid> {
        let guard = self.storage.read().unwrap();
        let now = std::time::Instant::now();
        guard
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.created_at) >= ttl)
            .map(|(id, _)| *id)
            .collect()
    }
}

#[derive(Clone, Debug, Resource)]
pub struct BlobStore {
    provider: Arc<dyn BlobProvider>,
}

impl Default for BlobStore {
    fn default() -> Self {
        Self {
            provider: Arc::new(MemoryProvider::default()),
        }
    }
}

impl BlobStore {
    pub fn new(provider: Arc<dyn BlobProvider>) -> Self {
        Self { provider }
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

        self.provider.store(id, data.to_vec(), metadata)?;
        Ok(ticket)
    }

    pub fn claim(&self, ticket: &SecureTicket) -> anyhow::Result<Vec<u8>> {
        match self.provider.retrieve(&ticket.id)? {
            Some((data, _)) => Ok(data),
            None => Err(anyhow::anyhow!("Ticket not found")),
        }
    }

    pub fn recover_ticket(&self, id: &Uuid) -> Option<SecureTicket> {
        match self.provider.retrieve(id).ok()? {
            Some((_, metadata)) => Some(SecureTicket { id: *id, metadata }),
            None => None,
        }
    }

    pub fn update_metadata(
        &self,
        id: &Uuid,
        new_metadata: HashMap<String, String>,
    ) -> anyhow::Result<()> {
        self.provider.update_metadata(id, new_metadata)
    }

    pub fn run_garbage_collection(&self) -> usize {
        let ttl = std::time::Duration::from_secs(60 * 15); // 15 Minutes TTL
        let expired = self.provider.list_expired(ttl);
        let count = expired.len();
        for id in expired {
            let _ = self.provider.delete(&id);
        }
        count
    }
}
