use std::sync::{Arc, Mutex};
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use unicity_agentic_demo::{App, Queries, Embedding, HnswMemoryIndex, LlmClient};

/// Shared application state for Tauri commands
pub struct AppState {
    pub app: Arc<Mutex<App>>,
    pub queries: Arc<Queries>,
    pub embedding: Arc<Embedding>,
    pub llm: Arc<LlmClient>,
    pub agent_index: Arc<Mutex<HnswMemoryIndex<'static>>>,
}

impl AppState {
    pub async fn new(
        app: App,
        db: Arc<Surreal<Db>>,
        api_key: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let queries = Arc::new(Queries::new(db));
        let embedding = app.embedding.clone();
        let llm = Arc::new(LlmClient::new(api_key));
        let agent_index = Arc::new(Mutex::new(HnswMemoryIndex::new(100_000, 1024)));

        Ok(Self {
            app: Arc::new(Mutex::new(app)),
            queries,
            embedding,
            llm,
            agent_index,
        })
    }
}

// Safety: AppState is Send + Sync because all its fields are Send + Sync
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}