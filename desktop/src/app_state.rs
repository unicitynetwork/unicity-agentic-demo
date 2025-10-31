use crate::constants::INITIAL_USDT_BALANCE;
use std::sync::{Arc, Mutex};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use unicity_agentic_demo::{App, Embedding, HnswMemoryIndex, LlmClient, Queries};

/// Shared application state for Tauri commands
pub struct AppState {
    pub app: Arc<Mutex<App>>,
    pub queries: Arc<Queries>,
    pub embedding: Arc<Embedding>,
    pub llm: Arc<LlmClient>,
    pub agent_index: Arc<Mutex<HnswMemoryIndex<'static>>>,
}

impl AppState {
    pub async fn new(app: App, db: Arc<Surreal<Db>>, api_key: String) -> Self {
        let queries = Arc::new(Queries::new(db));
        let embedding = app.embedding.clone();
        let llm = Arc::new(LlmClient::new(api_key));
        let agent_index = Arc::new(Mutex::new(HnswMemoryIndex::new(
            INITIAL_USDT_BALANCE as usize,
            1024,
        )));

        Self {
            app: Arc::new(Mutex::new(app)),
            queries,
            embedding,
            llm,
            agent_index,
        }
    }
}

// Safety: AppState is Send + Sync because all its fields are Send + Sync
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}
