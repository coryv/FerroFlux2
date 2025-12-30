CREATE TABLE IF NOT EXISTS analytics_events (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMP,
    tenant_id VARCHAR,
    node_id VARCHAR,
    workflow_id VARCHAR,
    event_type VARCHAR,
    payload JSON,
    duration_ms INTEGER,
    status VARCHAR
);

CREATE INDEX IF NOT EXISTS idx_analytics_tenant ON analytics_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_analytics_node ON analytics_events(node_id);
CREATE INDEX IF NOT EXISTS idx_analytics_timestamp ON analytics_events(timestamp);
