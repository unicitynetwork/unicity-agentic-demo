//! Flow Based Programming system for transaction processing
//!
//! This module implements the neurosymbolic flow composition engine
//! that enables multi-step transaction execution through semantic discovery.

pub mod parser;
pub mod composer;
pub mod executor;

pub use parser::{TransactionFlow, FlowStep, parse_transaction_flow};
pub use composer::FlowComposer;
pub use executor::FlowExecutor;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub steps: Vec<StepResult>,
    pub final_state: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step: usize,
    pub method_name: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub success: bool,
    pub error: Option<String>,
}