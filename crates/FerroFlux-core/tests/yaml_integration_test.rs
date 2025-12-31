use ferroflux_core::components::pipeline::PipelineNode;
use ferroflux_core::resources::registry::DefinitionRegistry;
use ferroflux_core::systems::pipeline::execute_pipeline_node;
use ferroflux_core::tools::primitives::{EmitTool, HttpClientTool, JsonQueryTool, SwitchTool};
use ferroflux_core::tools::registry::ToolRegistry;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn test_openai_chat_completion_yaml() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    // 1. Setup Mock Server
    let mock_server = rt.block_on(MockServer::start());

    rt.block_on(
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "created": 1677652288,
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello! How can I help you today?"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 9,
                    "completion_tokens": 12,
                    "total_tokens": 21
                }
            })))
            .mount(&mock_server),
    );

    // 2. Load Registry
    let mut def_registry = DefinitionRegistry::default();
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let platforms_dir = root_dir.join("platforms");

    // Load OpenAI platform and nodes
    def_registry
        .load_from_dir(&platforms_dir)
        .expect("Failed to load platforms");

    // 3. Override Base URL to point to Mock Server
    if let Some(platform) = def_registry.platforms.get_mut("openai") {
        platform
            .config
            .insert("base_url".to_string(), json!(mock_server.uri()));
        // Disable auth for mock or keep it (mock ignores it)
    } else {
        panic!("OpenAI platform definition not found!");
    }

    // 4. Register Tools
    let mut tool_registry = ToolRegistry::default();
    tool_registry.register(HttpClientTool);
    tool_registry.register(SwitchTool);
    tool_registry.register(JsonQueryTool);
    tool_registry.register(EmitTool);

    // 5. Create Node Instance
    let mut node_config = HashMap::new();
    node_config.insert("model".to_string(), json!("gpt-3.5-turbo"));
    node_config.insert("system_prompt".to_string(), json!("You are a test bot."));

    let mut pipeline_node = PipelineNode::new("openai.chat.completions".to_string(), node_config);

    // 6. Execute
    let mut inputs = HashMap::new();
    inputs.insert("user_prompt".to_string(), json!("Hello"));

    let mut global_memory = HashMap::new();

    let result = execute_pipeline_node(
        &mut pipeline_node,
        inputs,
        &def_registry,
        &tool_registry,
        &mut global_memory,
    )
    .expect("Pipeline execution failed");

    // 7. Verify Output
    // Check if 'response' output is emitted
    // My execute_pipeline_node logic collects returns from context into map?
    // The `EmitTool` writes to `_outputs` in context, which `execute_pipeline_node` returns.

    println!("Pipeline Result: {:?}", result);

    assert!(result.contains_key("Success"));
    assert!(result.contains_key("response"));
    assert_eq!(result["response"], "Hello! How can I help you today?");
}
