use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::{RecordId, Surreal};
use crate::models::{CreateAgent, CreateMethod, Method, Port, Record};

#[derive(Debug)]
pub struct Queries {
    db: Arc<Surreal<Db>>,
}

impl Queries {
    pub fn new(db: Arc<Surreal<Db>>) -> Queries {
        Queries { db }
    }

    pub async fn create_method(&self, method: CreateMethod) -> Result<Record, anyhow::Error> {
        let created: Option<Record> = self
            .db
            .create("method")
            .content(method)
            .await?;

        created.ok_or_else(|| anyhow::anyhow!("Failed to create method"))
    }

    pub async fn create_agent(&self, agent: CreateAgent) -> Result<Record, anyhow::Error> {
        let created: Option<Record> = self
            .db
            .create("agent")
            .content(agent)
            .await?;

        created.ok_or_else(|| anyhow::anyhow!("Failed to create agent"))
    }

    pub async fn relate_port(&self, from: RecordId, to: RecordId, port: Port) -> Result<Record, anyhow::Error> {
        let created: Option<Record> = self
            .db
            .query("RELATE $in->port->$out CONTENT $port")
            .bind(("in", from))
            .bind(("out", to))
            .bind(("port", port))
            .await?
            .take(0)?;

        created.ok_or_else(|| anyhow::anyhow!("Failed to create port"))
    }

    pub async fn get_method(&self, id: RecordId) -> Result<Method, anyhow::Error> {
        let method: Option<Method> = self
            .db
            .select(id)
            .await?;

        method.ok_or_else(|| anyhow::anyhow!("Method not found"))
    }
}