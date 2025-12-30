CREATE TABLE IF NOT EXISTS analytics_events (
    id UUID,
    timestamp DateTime64(3),
    tenant_id String,
    node_id String,
    workflow_id String,
    event_type String,
    payload String,
    duration_ms UInt32,
    status String
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (tenant_id, timestamp, node_id);
