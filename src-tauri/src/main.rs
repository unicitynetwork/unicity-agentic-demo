// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod app_state;
mod stt;
mod audio_capture;
mod audio_processor;
mod whisper_model;
mod error;
mod constants;

use std::sync::Arc;
use surrealdb::Surreal;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use unicity_agentic_demo::App;
use app_state::AppState;
use error::{WhisperError, WhisperResult};
use constants::INITIAL_USDT_BALANCE;

#[tokio::main]
async fn main() -> WhisperResult<()> {
    // Initialize tracing
    init_tracing();

    // Initialize database
    info!("🚀 Initializing database");
    let db: Arc<Surreal<surrealdb::engine::local::Db>> = Arc::new(Surreal::new::<surrealdb::engine::local::Mem>(()).await?);
    db.use_ns("unicity").use_db("demo").await?;
    info!("✅ Database initialized");

    // Get API key from environment - require it to be set
    let api_key = std::env::var("API_KEY")?;

    // Initialize the app
    info!("🏗️ Initializing application");
    let mut app = App::new(db.clone(), api_key.clone())?;
    
    // Initialize ledger with initial USDT balance
    app.set_balance("USDT", (INITIAL_USDT_BALANCE as u128) * unicity_agentic_demo::DECIMAL_FACTOR);
    info!("✅ App initialized with {} USDT", INITIAL_USDT_BALANCE);

    // Create shared app state
    let app_state = AppState::new(app, db, api_key).await?;

    // Initialize agents
    if let Err(e) = commands::initialize_agents(&app_state).await {
        error!("❌ Failed to initialize agents: {}", e);
        return Err(e);
    }

    // Run Tauri application
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::process_query,
            commands::get_balance,
            commands::get_all_balances,
            commands::get_agents,
            commands::get_transaction_history,
            commands::stt_start,
            commands::stt_stop
        ])
        .run(tauri::generate_context!())?;

    // Cleanup STT resources on shutdown
    // #[cfg(target_os = "macos")]
    // stt::cleanup();
    
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