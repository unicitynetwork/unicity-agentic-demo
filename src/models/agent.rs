use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use crate::models::method::MethodId;

pub type AgentId = RecordId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateAgent {
    pub label: String,
    pub version: String,
    pub methods: Vec<MethodId>,
    pub description: String,
}

/// An App is a collection of methods working together to achieve a goal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agent {
    pub id: AgentId,
    pub label: String,
    pub version: String,
    pub methods: Vec<MethodId>,
    pub description: String,
}
