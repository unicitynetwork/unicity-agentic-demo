use crate::embedding::Embedding;
use crate::hnsw::HnswMemoryIndex;
use crate::ledger::Ledger;
use crate::models::{
    Channel, CreateAgent, CreateMethod, CreatePort, ExecKind, ProgramAbi, ProgramRef, Visibility,
};
use crate::queries::Queries;
use fluent_uri::Uri;
use serde_json::Value;
use std::sync::Arc;

pub struct Ping;

impl Ping {
    pub async fn create_agent(
        queries: &Queries,
        embedding: Arc<Embedding>,
        hnsw: &mut HnswMemoryIndex<'_>,
    ) -> Result<(), anyhow::Error> {
        let create_agent = CreateAgent {
            label: "Ping Agent".to_string(),
            version: "1.0.0".to_string(),
            methods: vec![],
            description: "An agent that responds to ping with pong.".to_string(),
        };

        let description =
            "A ping pong agent to test if a system is working by pinging and getting back a pong."
                .to_string();
        let agent_handle = queries.create_agent(create_agent).await?;
        let embedding = embedding.embed(&description).await?;
        let string_type: Uri<String> = "type://string".parse()?;

        let method = CreateMethod {
            agent: agent_handle.id,
            label: "Ping".to_string(),
            version: "1.0.0".to_string(),
            visibility: Visibility::Public,
            exec_kind: ExecKind::Local,
            description,
            in_port: CreatePort {
                label: "in.ping".to_string(),
                description: "Returns a pong from a ping.".to_string(),
                channel: Channel::Call,
                type_uri: None,
                end_point: Uri::parse("agent:://Ping/ping#in")?.to_owned(),
            },
            out_port: CreatePort {
                label: "out.ping".to_string(),
                description: "Returns a pong.".to_string(),
                channel: Channel::Call,
                type_uri: string_type.into(),
                end_point: Uri::parse("agent:://Ping/ping#out")?.to_owned(),
            },
            program: ProgramRef {
                module_uri: Uri::parse("local://Ping/ping")?.to_owned(),
                export: "ping".to_string(),
                abi: ProgramAbi::LocalFn,
                checksum: "n/a".to_string(),
            },
            embedding: embedding.clone(),
        };

        let method_handle = queries.create_method(method).await?;
        hnsw.add(method_handle.id, &embedding)?;

        Ok(())
    }
}

pub fn ping(_ledger: &mut Ledger, input: Value) -> Value {
    let response = format!("pong: {}", input);
    Value::String(response)
}
