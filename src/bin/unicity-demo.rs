use clap::Parser;
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tracing::{info, error, warn, debug, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use unicity_agentic_demo::{
    App, LlmClient, Queries, Embedding, HnswMemoryIndex,
    FlowComposer, FlowExecutor, parse_transaction_flow,
    format_amount_for_display, format_amount_for_llm, DECIMAL_FACTOR
};

/// Format amount with proper decimal places for display
pub fn format_amount(amount: u128, decimals: u32) -> String {
    if decimals == 0 {
        return amount.to_string();
    }
    let divisor = 10u128.pow(decimals);
    let whole = amount / divisor;
    let frac  = amount % divisor;
    // pad the fractional part with leading zeros up to `decimals` width
    format!("{whole}.{frac:0>width$}", width = decimals as usize)
}

#[derive(Parser)]
#[command(name = "unicity-demo")]
#[command(about = "Unicity Agentic Demo - Neurosymbolic Flow Based Programming System")]
struct Args {
    /// OpenAI-compatible API key for LLM
    #[arg(short, long, env = "API_KEY")]
    api_key: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    // Initialize tracing with emoji prefixes
    init_tracing(&args.log_level)?;

    // 🚀 Initialize database
    info!("🚀 Initializing Unicity Agentic Demo");
    let db = Arc::new(Surreal::new::<surrealdb::engine::local::Mem>(()).await?);
    db.use_ns("unicity").use_db("demo").await?;
    info!("✅ Database initialized successfully");

    // 🤖 Initialize LLM client
    info!("🤖 Initializing LLM client");
    let llm_client = LlmClient::new(args.api_key);
    info!("✅ LLM client initialized");

    // 🧠 Initialize embedding service
    info!("🧠 Initializing embedding service");
    let embedding = Arc::new(Embedding::new()?);
    info!("✅ Embedding service initialized");

    // 🗺️ Initialize HNSW index
    info!("🗺️ Initializing HNSW memory index");
    let mut hnsw_index = HnswMemoryIndex::new(100_000, 1024);
    info!("✅ HNSW index initialized");

    // 📊 Initialize queries
    info!("📊 Initializing query service");
    let queries = Queries::new(db.clone());
    info!("✅ Query service initialized");

    // 🏗️ Initialize app
    info!("🏗️ Initializing application");
    let mut app = App::new(db, llm_client.api_key.clone())?;
    
    // 💰 Initialize ledger with 100,000 USDT (8 decimal places)
    info!("💰 Initializing ledger with 100,000 USDT");
    app.set_balance("USDT", 100_000 * DECIMAL_FACTOR); // 8 decimal places: 100,000.00000000 USDT
    info!("✅ Ledger initialized: {} USDT", format_amount_for_display(app.get_balance("USDT")));

    // 🤖 Register agents
    info!("🤖 Registering agents in knowledge graph");
    register_agents(&queries, embedding.clone(), &mut hnsw_index).await?;
    info!("✅ All agents registered successfully");

    // 🎯 Start interactive demo
    info!("🎯 Starting interactive demo");
    start_interactive_demo(&mut app, Arc::new(queries), &mut hnsw_index).await?;

    Ok(())
}

fn init_tracing(log_level: &str) -> Result<(), anyhow::Error> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer()
            .with_target(false)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true))
        .init();

    Ok(())
}

async fn register_agents(
    queries: &Queries,
    embedding: Arc<Embedding>,
    hnsw_index: &mut HnswMemoryIndex<'_>,
) -> Result<(), anyhow::Error> {
    // 🏓 Register Ping Agent
    info!("🏓 Registering Ping Agent");
    unicity_agentic_demo::agents::ping::Ping::create_agent(
        queries,
        embedding.clone(),
        hnsw_index,
    ).await?;
    info!("✅ Ping Agent registered");

    // 💱 Register Swap Agent
    info!("💱 Registering Swap Agent");
    unicity_agentic_demo::agents::swap::Swap::create_agent(
        queries,
        embedding.clone(),
        hnsw_index,
    ).await?;
    info!("✅ Swap Agent registered");

    Ok(())
}

async fn start_interactive_demo(
    app: &mut App,
    queries: Arc<Queries>,
    hnsw_index: &mut HnswMemoryIndex<'_>,
) -> Result<(), anyhow::Error> {
    println!("\n🎉 Unicity Agentic Demo Ready!");
    println!("💰 Available balance: {} USDT", format_amount_for_display(app.get_balance("USDT")));
    println!("\n📝 Enter your query (or 'quit' to exit):");
    
    loop {
        print!("\n❓ > ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            info!("👋 Shutting down demo");
            break;
        }
        
        if input.is_empty() {
            continue;
        }
        
        // Process query
        match process_query(input, app, queries.clone(), hnsw_index).await {
            Ok(response) => {
                println!("\n🤖 {}", response);
            }
            Err(e) => {
                error!("❌ Error processing query: {}", e);
                println!("\n❌ Sorry, something went wrong: {}", e);
            }
        }
    }
    
    Ok(())
}

async fn process_query(
    query: &str,
    app: &mut App,
    queries: Arc<Queries>,
    hnsw_index: &mut HnswMemoryIndex<'_>,
) -> Result<String, anyhow::Error> {
    info!("🔍 Processing query: {}", query);
    
    // 1. Parse query with LLM to get transaction flow
    debug!("🧠 Parsing query with LLM");
    let transaction_flow = parse_transaction_flow(query, &app.llm).await?;
    info!("✅ Parsed transaction flow with {} steps", transaction_flow.pipeline.len());

    // 2. Create flow composer and executor
    let composer = FlowComposer::new(
        queries.clone(),
        app.embedding.clone(),
        app.llm.clone(),
    );
    
    let mut executor = FlowExecutor::new(app.ledger.clone());
    
    // 3. Compose flow using semantic search and graph traversal
    debug!("🎼 Composing flow");
    let composed_flow = composer.compose_flow(&transaction_flow, hnsw_index).await?;
    info!("✅ Flow composed successfully");

    // 4. Execute complete flow
    debug!("⚡ Executing flow");
    let execution_result = executor.execute_flow(&composed_flow).await;
    
    // Update app ledger with execution results
    app.ledger = executor.get_ledger();
    
    match execution_result.success {
        true => {
            info!("✅ Flow executed successfully");
            
            // 5. Summarize results with LLM
            debug!("🤖 Generating execution summary");
            let summary = generate_execution_summary(
                query,
                &execution_result,
                &app.llm
            ).await?;
            
            Ok(summary)
        }
        false => {
            error!("❌ Flow execution failed: {:?}", execution_result.error);
            Ok(format!("❌ Execution failed: {:?}", execution_result.error))
        }
    }
}

/// Generate natural language summary of execution results
async fn generate_execution_summary(
    original_query: &str,
    execution_result: &unicity_agentic_demo::ExecutionResult,
    llm_client: &LlmClient,
) -> Result<String, anyhow::Error> {
    info!("🤖 Generating execution summary");
    
    let execution_json = serde_json::to_string_pretty(execution_result)?;
    
    let prompt = format!(r#"
You are explaining transaction execution results to a user.
Provide a natural, clear explanation of what happened.

IMPORTANT: All amounts in this system use 8 decimal places internally. When explaining amounts to users, use human-readable decimal format (e.g., "100" instead of "100.00000000", "0.5" instead of "0.50000000").

EXECUTION RESULTS: {}
ORIGINAL QUERY: "{}"

Explain in natural tone what was accomplished, including any intermediate steps and final outcomes.
When mentioning amounts, format them in a user-friendly way:
- Use "100" instead of "100.00000000"
- Use "0.5" instead of "0.50000000"
- Use "1.23456789" for amounts with non-zero fractional parts

Be concise but comprehensive."#, execution_json, original_query);

    let response = llm_client.generate_response(&prompt).await;
    
    match response.success {
        true => {
            info!("✅ Execution summary generated");
            Ok(response.response)
        }
        false => {
            error!("❌ Failed to generate execution summary: {:?}", response.error);
            Ok("✅ Transaction completed successfully, but summary generation failed.".to_string())
        }
    }
}