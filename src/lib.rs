//! Library interface for unicity-agentic-demo
//!
//! This module provides the public API for the unicity-agentic-demo library,
//! allowing it to be used as a dependency in the Tauri UI.

pub mod agents;
pub mod app;
pub mod decimal;
pub mod embedding;
pub mod executor;
pub mod flow;
pub mod hnsw;
pub mod ledger;
pub mod llm;
pub mod models;
pub mod queries;

// Re-export public API
pub use app::App;
pub use decimal::{
    DECIMAL_FACTOR, DECIMAL_PLACES, format_amount_for_display, format_amount_for_llm,
    format_decimal_amount, parse_amount_from_llm, parse_decimal_amount,
};
pub use embedding::Embedding;
pub use flow::{
    ExecutionResult, FlowComposer, FlowExecutor, FlowStep, TransactionFlow, parse_transaction_flow,
};
pub use hnsw::HnswMemoryIndex;
pub use ledger::Ledger;
pub use llm::LlmClient;
pub use queries::Queries;
