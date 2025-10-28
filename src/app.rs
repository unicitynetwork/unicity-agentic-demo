use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use crate::embedding::Embedding;
use crate::ledger::Ledger;
use crate::models::Agent;
use crate::queries::Queries;

pub struct App {
    pub ledger: Ledger,
    pub query: Queries,
    pub embedding: Arc<Embedding>,
}

impl App {
    pub fn new(db: Arc<Surreal<Db>>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            ledger: Ledger::new(),
            query: Queries::new(db),
            embedding: Arc::new(Embedding::new()?),
        })
    }

    pub fn register_agent(&self, agent: Agent) {
        // self.query.
    }

    pub fn get_balance(&self, asset_id: &str) -> u128 {
        self.ledger.get_balance(&asset_id.to_string())
    }

    pub fn set_balance(&mut self, asset_id: &str, amount: u128) {
        self.ledger.set_balance(asset_id.to_string(), amount);
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        todo!()
    }
}