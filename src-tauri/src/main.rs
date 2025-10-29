// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod app_state;

use std::sync::Arc;
use surrealdb::Surreal;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use unicity_agentic_demo::App;
use app_state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init_tracing();

    // Initialize database
    info!("🚀 Initializing database");
    let db: Arc<Surreal<surrealdb::engine::local::Db>> = Arc::new(Surreal::new::<surrealdb::engine::local::Mem>(()).await?);
    db.use_ns("unicity").use_db("demo").await?;
    info!("✅ Database initialized");

    // Get API key from environment or use a default for demo
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "demo-key".to_string());

    // Initialize the app
    info!("🏗️ Initializing application");
    let mut app = App::new(db.clone(), api_key.clone())?;
    
    // Initialize ledger with 100,000 USDT
    app.set_balance("USDT", 100_000 * unicity_agentic_demo::DECIMAL_FACTOR);
    info!("✅ App initialized with 100,000 USDT");

    // Create shared app state
    let app_state = AppState::new(app, db, api_key).await?;

    // Initialize agents
    if let Err(e) = commands::initialize_agents(&app_state).await {
        error!("❌ Failed to initialize agents: {}", e);
    }

    // Run Tauri application
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::process_query,
            commands::get_balance,
            commands::get_all_balances,
            commands::get_agents,
            commands::get_transaction_history
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "unicity_agentic_demo=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
}