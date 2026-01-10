use ferroflux_sdk::protocol::{ClientRequest, ProtocolMessage, ServerEvent};
use flow_canvas::model::NodeData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MyData;
impl NodeData for MyData {
    fn node_type(&self) -> String {
        "MyData".to_string()
    }
}

fn main() -> anyhow::Result<()> {
    println!("--> Testing Protocol Serialization");

    // 1. Create a Connect Request
    let req = ProtocolMessage::<MyData>::Request(ClientRequest::Connect {
        client_id: "test-client".to_string(),
    });

    let json = serde_json::to_string(&req)?;
    println!("[Client -> Server]: {}", json);

    // Verify deserialization
    let parsed: ProtocolMessage<MyData> = serde_json::from_str(&json)?;
    if let ProtocolMessage::Request(ClientRequest::Connect { client_id }) = parsed {
        assert_eq!(client_id, "test-client");
    } else {
        panic!("Failed to parse Connect request");
    }

    // 2. Create a Telemetry Event
    let event = ProtocolMessage::<MyData>::Event(ServerEvent::Telemetry {
        node_id: uuid::Uuid::new_v4(),
        success: true,
        execution_ms: 42,
        trace_id: "abc-123".to_string(),
    });

    let json_event = serde_json::to_string(&event)?;
    println!("[Server -> Client]: {}", json_event);

    println!("--> Protocol Check Passed");
    Ok(())
}
