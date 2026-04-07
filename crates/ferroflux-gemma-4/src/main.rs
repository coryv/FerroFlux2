use clap::Parser;
use ferroflux_gemma_4::{AgentConfig, ChatMessage, GemmaAgent};
use std::time::Instant;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about = "Gemma 4 Agent CLI", long_about = None)]
struct Args {
    /// The user prompt to send to the agent
    prompt: String,

    /// Enable thinking/reasoning mode
    #[arg(short, long, default_value_t = false)]
    thinking: bool,

    /// Max tokens to generate
    #[arg(short, long, default_value_t = 1000)]
    max_tokens: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize pretty logging for the smart loader's output
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    println!("🚀 Initializing Gemma 4 Agent...");
    let start_load = Instant::now();
    
    let config = AgentConfig {
        enable_thinking: args.thinking,
        max_new_tokens: args.max_tokens,
        ..AgentConfig::default()
    };

    // The smart loader automatically assesses system RAM (8GB threshold)
    let mut agent = GemmaAgent::load(config)?;
    
    let profile = agent.model_profile();
    println!(
        "✅ Loaded model: {:?} ({}) in {:.2}s",
        profile.variant,
        profile.hf_repo,
        start_load.elapsed().as_secs_f32()
    );

    println!("\nThinking: {}", if args.thinking { "ENABLED" } else { "DISABLED" });
    println!("Prompt: {}\n", args.prompt);

    let messages = vec![ChatMessage::new_user(args.prompt)];
    
    println!("--- Model Output ---");
    let response = agent.chat(&messages, None)?;

    if let Some(thought) = response.thinking {
        println!("\n[THOUGHTS]\n{}\n", thought);
    }

    println!("\n[RESPONSE]\n{}\n", response.content);
    
    if !response.tool_calls.is_empty() {
        println!("\n[TOOL CALLS]\n{:?}\n", response.tool_calls);
    }

    println!("-------------------");
    println!(
        "Tokens: {} | Speed: {:.2} t/s",
        response.tokens_generated,
        response.tokens_per_second
    );

    Ok(())
}
