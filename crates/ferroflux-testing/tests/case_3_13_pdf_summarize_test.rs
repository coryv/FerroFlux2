use ferroflux_testing::harness::TestHarness;
use ferroflux_types::tool::{Tool, ToolContext};
use serde_json::{json, Value};
use anyhow::Result;

pub struct MockAgentTool;
impl Tool for MockAgentTool {
    fn id(&self) -> &'static str { "agent" }
    fn run(&self, _ctx: &mut ToolContext, params: Value) -> Result<Value> {
        let prompt = params.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
        let response = format!("MOCKED_SUMMARY: {}", prompt);
        Ok(json!({ "result": response }))
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_case_3_13_pdf_summarize() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("ferroflux_core=debug,ferroflux_testing=debug,ferroflux_tools=debug")
        .with_test_writer()
        .try_init();

    let mut harness = TestHarness::new().await;
    
    // Manually register our mock agent tool
    {
        let mut registry = harness.app.world.resource_mut::<ferroflux_types::tool::ToolRegistry>();
        registry.register(MockAgentTool);
    }
    
    harness.load_platforms().ok();

    // Smallest possible valid PDF (approx)
    let pdf_content = b"%PDF-1.1\n1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj\n3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >> endobj\n4 0 obj << /Length 20 >> stream\nBT /F1 12 Tf (Hello PDF) Tj ET\nendstream endobj\ntrailer << /Root 1 0 R >>\n%%EOF";
    use base64::{Engine as _, engine::general_purpose};
    let pdf_base64 = general_purpose::STANDARD.encode(pdf_content);

    // Load and trigger workflow
    let workflow_yaml = std::fs::read_to_string("fixtures/pdf_summarize.yaml")?
        .replace("=pdf_base64", &pdf_base64); // Simple inject for this test
    
    harness.load_waml(&workflow_yaml)?;

    let trigger_id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, b"webhook_in");
    harness.trigger_node(trigger_id, json!({}))?;

    harness.run_until_idle(100).await;

    Ok(())
}
