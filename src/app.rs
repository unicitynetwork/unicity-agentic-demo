use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tracing::{info, debug, trace, warn};
use crate::embedding::Embedding;
use crate::hnsw::HnswMemoryIndex;
use crate::ledger::Ledger;
use crate::llm::LlmClient;
use crate::queries::Queries;
use crate::decimal::{format_amount_for_display};

pub struct App {
    pub ledger: Ledger,
    pub query: Queries,
    pub embedding: Arc<Embedding>,
    pub llm: LlmClient,
    pub agent_index: HnswMemoryIndex<'static>,
}

impl App {
    pub fn new(db: Arc<Surreal<Db>>, api_key: String) -> Result<Self, anyhow::Error> {
        info!("🏗️  Creating new App instance");
        debug!("🔧 Initializing components");
        
        let ledger = Ledger::new();
        trace!("💰 Ledger initialized");
        
        let query = Queries::new(db);
        trace!("📊 Query service initialized");
        
        let embedding = Arc::new(Embedding::new()?);
        trace!("🧠 Embedding service initialized");
        
        let llm = LlmClient::new(api_key);
        trace!("🤖 LLM client initialized");
        
        let agent_index = HnswMemoryIndex::new(100_000, 1024);
        trace!("🗺️  HNSW index initialized");
        
        info!("✅ App instance created successfully");
        
        Ok(Self {
            ledger,
            query,
            embedding,
            llm,
            agent_index,
        })
    }

    pub fn get_balance(&self, asset_id: &str) -> u128 {
        trace!("💰 Getting balance for asset: {}", asset_id);
        let balance = self.ledger.get_balance(&asset_id.to_string());
        debug!("💰 Balance for {}: {} (internal: {})", asset_id, format_amount_for_display(balance), balance);
        balance
    }

    pub fn set_balance(&mut self, asset_id: &str, amount: u128) {
        info!("💰 Setting balance for {}: {} (internal: {})", asset_id, format_amount_for_display(amount), amount);
        debug!("💰 Previous balance: {} (internal: {})", format_amount_for_display(self.get_balance(asset_id)), self.get_balance(asset_id));
        self.ledger.set_balance(asset_id.to_string(), amount);
        debug!("💰 New balance: {} (internal: {})", format_amount_for_display(self.get_balance(asset_id)), self.get_balance(asset_id));
        trace!("✅ Balance updated successfully");
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        warn!("⚠️  App::run() not implemented yet");
        todo!()
    }
}